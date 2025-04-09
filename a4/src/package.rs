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
        // Generate a set of packages and place them into the event queue
        // Update the package checksum with each package name
        for i in 0..self.num_pkgs {
            let name = fs::read_to_string("data/packages.txt")
                .unwrap()
                .lines()
                .cycle()
                .nth(self.pkg_start_idx + i)
                .unwrap()
                .to_owned();

            pkg_checksum
                .lock()
                .unwrap()
                .update(Checksum::with_sha256(&name));
            self.event_sender
                .send(Event::DownloadComplete(Package { name }))
                .unwrap();
        }
    }
}
