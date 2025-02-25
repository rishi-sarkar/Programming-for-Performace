use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};

use itertools::Itertools;

use rpkg::debversion;
use rpkg::debversion::{DebianVersionNum,VersionRelation};

mod deps_available;
mod solvers;
mod parsers;
mod async_fns;

use crate::packages::async_fns::AsyncState;

static PACKAGE_COUNTER: AtomicI32 = AtomicI32::new(0);

pub struct Packages {
    dependencies : HashMap<i32,Vec<Dependency>>,
    md5sums : HashMap<i32,String>,
    available_debvers : HashMap<i32,DebianVersionNum>,
    installed_debvers : HashMap<i32,DebianVersionNum>,
    package_name_to_num : HashMap<String, i32>,
    package_num_to_name : HashMap<i32, String>,
    async_state : AsyncState,
}

// Dependency([X, Y, Z]) means X|Y|Z
pub struct RelVersionedPackageNum {
    package_num : i32,
    rel_version : Option<(VersionRelation, String)>
}
pub type Dependency = Vec<RelVersionedPackageNum>;

impl Packages {
    pub fn new() -> Packages {
        Packages { 
            dependencies : HashMap::new(), 
            md5sums : HashMap::new(),
            available_debvers : HashMap::new(),
            installed_debvers : HashMap::new(),
            package_name_to_num : HashMap::new(), 
            package_num_to_name : HashMap::new(),
            async_state : AsyncState::new(),
        }
    }

    // next few functions manipulate the list of packages and the name/number interface
    pub fn get_package_names(&self) -> Vec<&str> {
        self.package_name_to_num.keys().map(|x| &x[..]).collect()
    }

    fn get_package_name(&self, package_num: i32) -> &str {
        return self.package_num_to_name.get(&package_num).unwrap();
    }

    // panics if package_name doesn't already exist
    fn get_package_num(&self, package_name: &str) -> &i32 {
        return self.package_name_to_num.get(package_name).unwrap();
    }

    // inserts package_name into package_name_to_num if it doesn't already exist
    fn get_package_num_inserting(&mut self, package_name: &str) -> i32 {
        if !self.package_name_to_num.contains_key(package_name) {
            let pnum = PACKAGE_COUNTER.load(Ordering::SeqCst);
            self.package_name_to_num.insert(String::from(package_name), pnum);
            self.package_num_to_name.insert(pnum, String::from(package_name));
            self.dependencies.insert(pnum, vec![]);
            PACKAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
            return pnum;
        } else {
            return *self.package_name_to_num.get(package_name).unwrap();
        }
    }

    pub fn package_exists(&self, package_name: &str) -> bool {
        return self.package_name_to_num.contains_key(package_name);
    }

    // accessor methods for various maps
    pub fn get_available_debver(&self, package_name: &str) -> Option<&DebianVersionNum> {
        let package_num = self.package_name_to_num.get(package_name);
        return match package_num {
            None => None,
            Some(x) => match self.available_debvers.get(x) {
                None => None,
                Some(y) => Some(y)
            }
        }
    }

    pub fn get_installed_debver(&self, package_name: &str) -> Option<&DebianVersionNum> {
        let package_num = self.package_name_to_num.get(package_name);
        return match package_num {
            None => None,
            Some(x) => match self.installed_debvers.get(x) {
                None => None,
                Some(y) => Some(y)
            }
        }
    }

    pub fn get_md5sum(&self, package_name: &str) -> Option<&str> {
        let package_num = self.package_name_to_num.get(package_name);
        return match package_num {
            None => None,
            Some(x) => match self.md5sums.get(x) {
                None => None,
                Some(y) => Some(y)
            }
        }
    }

    // helper functions; these aren't structs so I can't make them implement Fmt::Display.
    fn deps2str(&self, deps: &Vec<Dependency>) -> String {
        return deps.iter().map(|dep| self.dep2str(dep)).format(", ").to_string();
    }

    fn dep2str(&self, dep: &Dependency) -> String {
        return dep.iter().map(|d| {
            let pn = self.get_package_name(d.package_num);
            match &d.rel_version {
                None => String::from(pn),
                Some((rel, ver)) => format!("{} ({} {})", pn, rel.to_string(), ver)
            }
        }).format(" | ").to_string();
    }

    // output commands
    pub fn print_deps(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        let deps : &Vec<Dependency> = &*self.dependencies.get(self.get_package_num(package_name)).unwrap();
        println!("{:?} depends on {:?}", package_name, self.deps2str(deps));
    }

    pub fn print_transitive_dep_solution(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        let dep_solution : Vec<i32> = self.transitive_dep_solution(package_name);
        println!("{:?} transitive dependency solution: {:?}", package_name, dep_solution.iter().map(|dep| self.get_package_name(*dep)).format(", ").to_string());
    }

    pub fn print_how_to_install(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        println!("Package {}:", package_name);
        let pkgs_to_install:Vec<i32> = self.compute_how_to_install(package_name);
        println!("{:?} to install: {:?}", package_name, pkgs_to_install.iter().map(|dep| self.get_package_name(*dep)).format(", ").to_string());
    }

    pub fn print_info(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        println!("Package: {}", package_name);
        let a = self.get_available_debver(package_name);
        let i = self.get_installed_debver(package_name);
        match a {
            None => (),
            Some(a) => {
                println!("Version: {}", a.to_string());
                println!("MD5Sum: {}", self.get_md5sum(package_name).unwrap().to_string());
                println!("Depends: {}", self.deps2str(&*self.dependencies.get(self.get_package_num(package_name)).unwrap()));
            }
        }
        match i {
            None => (),
            Some(i) => { println!("Installed-Version: {}", i.to_string()) }
        }
        match (a, i) {
            (Some(aa), Some(ii)) =>
                { println!("Newer-Available: {:?}", aa > ii); }
            _ => ()
        }
    }

    // generate output for package-verifier
    pub fn output_md5s(&self, fname: &str) {
        let path = Path::new(fname);
        let mut md5s : String = "name,version,hash\n".to_owned();
        for pn in self.get_package_names() {
            match (self.get_available_debver(pn), self.get_md5sum(pn)) {
                (Some(v), Some(m)) => {
                    let row = format!("{},{},{}\n",pn,v.to_string(),m);
                    md5s.push_str(&row)
                }
                (_, _) => ()
            }
        }
        std::fs::write(path, md5s).unwrap();
    }

    // provided parse function to let students do the async io part independently
    pub fn parse_csv(&mut self, filename: &str) {
        let mut rdr = csv::Reader::from_path(filename).unwrap();
        for line in rdr.records() {
            let line = line.unwrap();
            let package_name = String::from(line.get(0).unwrap());
            let debver = String::from(line.get(1).unwrap()).parse::<debversion::DebianVersionNum>().unwrap();
            let md5sum = String::from(line.get(2).unwrap());

            let package_num = self.get_package_num_inserting(&package_name);
            self.available_debvers.insert(package_num, debver);
            self.md5sums.insert(package_num, md5sum);

        }

        println!("Packages available: {}", self.available_debvers.keys().len());
    }
}
