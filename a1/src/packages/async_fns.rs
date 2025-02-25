use urlencoding::encode;

use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use std::collections::HashMap;
use std::str;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use crate::Packages;

struct Collector(Box<String>);
impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        (*self.0).push_str(str::from_utf8(&data.to_vec()).unwrap());
        Ok(data.len())
    }
}

const DEFAULT_SERVER: &str = "ece459.patricklam.ca:4590";
impl Drop for Packages {
    fn drop(&mut self) {
        self.execute()
    }
}

static EASYKEY_COUNTER: AtomicI32 = AtomicI32::new(0);

pub struct AsyncState {
    server: String,
    easy_vec: Vec<Easy2Handle<Collector>>,
    combined_data: HashMap<i32, (String, String, i32)>,
    multi: Multi,
}

impl AsyncState {
    pub fn new() -> AsyncState {
        AsyncState {
            server: String::from(DEFAULT_SERVER),
            easy_vec: Vec::new(),
            combined_data: HashMap::new(),
            multi: Multi::new(),
        }
    }
}

impl Packages {
    pub fn set_server(&mut self, new_server: &str) {
        self.async_state.server = String::from(new_server);
    }

    /// Retrieves the version number of pkg and calls enq_verify_with_version with that version number.
    pub fn enq_verify(&mut self, pkg: &str) {
        let version = self.get_available_debver(pkg);
        match version {
            None => {
                println!("Error: package {} not defined.", pkg);
                return;
            }
            Some(v) => {
                let vs = &v.to_string();
                self.enq_verify_with_version(pkg, vs);
            }
        };
    }

    /// Enqueues a request for the provided version/package information. Stores any needed state to async_state so that execute() can handle the results and print out needed output.
    pub fn enq_verify_with_version(&mut self, pkg: &str, version: &str) {
        let url = format!(
            "http://{}/rest/v1/checksums/{}/{}",
            self.async_state.server,
            pkg,
            encode(version)
        );

        let key = EASYKEY_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut easy: Easy2<Collector> = Easy2::new(Collector(Box::new(String::new())));
        easy.url(&url).unwrap();
        let easyhandle = self.async_state.multi.add2(easy).unwrap();
        
        let pkg_num: i32 = self.get_package_num_inserting(pkg);

        self.async_state.easy_vec.push(easyhandle);
        self.async_state
            .combined_data
            .insert(key, (pkg.to_string(), version.to_string(), pkg_num));

        println!("queueing request {}", url);
    }

    /// Asks curl to perform all enqueued requests. For requests that succeed with response code 200, compares received MD5sum with local MD5sum (perhaps stored earlier). For requests that fail with 400+, prints error message.
    pub fn execute(&mut self) {
        EASYKEY_COUNTER.store(0, Ordering::SeqCst);

        self.async_state.multi.pipelining(true, true).unwrap();

        while self.async_state.multi.perform().unwrap() > 0 {
            self.async_state
                .multi
                .wait(&mut [], Duration::from_secs(30))
                .unwrap();
        }

        for easy in self.async_state.easy_vec.drain(..) {
            let easy_key = EASYKEY_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut easy_after = self.async_state.multi.remove2(easy).unwrap();
            let response_code = easy_after.response_code().unwrap();
            let (pkg, version, pkg_num) = self.async_state.combined_data.get(&easy_key).unwrap();

            if response_code == 200 {
                let md5sum_local: &str = self.md5sums.get(&pkg_num).unwrap();
                let md5sum_response: &str = easy_after.get_ref().0.as_str();
                let matches = md5sum_local == md5sum_response;
                println!("verifying {}, matches: {:?}", pkg, matches);
            } else {
                println!(
                    "got error {} on request for package {} version {}",
                    response_code, pkg, version
                );
            }
        }

        EASYKEY_COUNTER.store(0, Ordering::Relaxed);
        self.async_state.easy_vec.clear();
        self.async_state.combined_data.clear();
    }
}
