use super::checksum::Checksum;
use super::Event;
use crossbeam::channel::Sender;
use std::fs;
use std::sync::{Arc, Mutex};

pub struct Package {
    pub name: String,
}

pub struct PackageDownloader {
    pkg_start_idx: usize,
    num_pkgs: usize,
    event_sender: Sender<Event>,
}

impl PackageDownloader {
    pub fn new(pkg_start_idx: usize, num_pkgs: usize, event_sender: Sender<Event>) -> Self {
        Self {
            pkg_start_idx,
            num_pkgs,
            event_sender,
        }
    }

    pub fn run(&self, pkg_checksum: Arc<Mutex<Checksum>>) {
        let file_contents = fs::read_to_string("data/packages.txt")
            .expect("Failed to read packages file");
        let lines: Vec<&str> = file_contents.lines().collect();
        let total_lines = lines.len();
    
        // Accumulate checksum locally instead of locking every iteration.
        let mut local_checksum = Checksum::default();
    
        for i in 0..self.num_pkgs {
            let index = (self.pkg_start_idx + i) % total_lines;
            let name = lines[index].to_owned();
    
            // Update local checksum.
            local_checksum.update(Checksum::with_sha256(&name));
    
            // Send event without any extra locking overhead.
            self.event_sender
                .send(Event::DownloadComplete(Package { name }))
                .unwrap();
        }
    
        // Once done, update the global checksum in one lock.
        let mut global_checksum = pkg_checksum.lock().unwrap();
        global_checksum.update(local_checksum);
    }
    
}
