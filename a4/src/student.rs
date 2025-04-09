use super::{checksum::Checksum, idea::Idea, package::Package, Event};
use crossbeam::channel::{Receiver, Sender};
// use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};

pub struct Student {
    id: usize,
    idea: Option<Idea>,
    pkgs: Vec<Package>,
    skipped_idea: bool,
    event_sender: Sender<Event>,
    event_recv: Receiver<Event>,
}

impl Student {
    pub fn new(id: usize, event_sender: Sender<Event>, event_recv: Receiver<Event>) -> Self {
        Self {
            id,
            event_sender,
            event_recv,
            idea: None,
            pkgs: vec![],
            skipped_idea: false,
        }
    }

    fn build_idea(
        &mut self,
        local_idea_checksum: &mut Checksum,
        local_pkg_checksum: &mut Checksum,
    ) {
        if let Some(ref idea) = self.idea {
            let pkgs_required = idea.num_pkg_required;
            if pkgs_required <= self.pkgs.len() {
                local_idea_checksum.update(Checksum::with_sha256(&idea.name));
                let drained: Vec<Package> = self.pkgs.drain(0..pkgs_required).collect();
                for pkg in &drained {
                    local_pkg_checksum.update(Checksum::with_sha256(&pkg.name));
                }

                // // Build the output string using the local accumulators.
                // let mut output = String::with_capacity(256);
                // let idea_val = format!("{}", local_idea_checksum);
                // let pkg_val = format!("{}", local_pkg_checksum);
                // output.push_str(&format!(
                //     "\nStudent {} built {} using {} packages\nIdea checksum: {}\nPackage checksum: {}",
                //     self.id, idea.name, pkgs_required, idea_val, pkg_val
                // ));
                // for pkg in &drained {
                //     output.push_str(&format!("\n> {}", pkg.name));
                // }
                // let stdout = stdout();
                // let mut handle = stdout.lock();
                // writeln!(handle, "{}", output).unwrap();

                self.idea = None;
            }
        }
    }

    pub fn run(&mut self, idea_checksum: Arc<Mutex<Checksum>>, pkg_checksum: Arc<Mutex<Checksum>>) {
        let mut local_idea_checksum = Checksum::default();
        let mut local_pkg_checksum = Checksum::default();

        loop {
            let event = self.event_recv.recv().unwrap();
            match event {
                Event::NewIdea(idea) => {
                    if self.idea.is_none() {
                        self.idea = Some(idea);
                        self.build_idea(&mut local_idea_checksum, &mut local_pkg_checksum);
                    } else {
                        self.event_sender.send(Event::NewIdea(idea)).unwrap();
                        self.skipped_idea = true;
                    }
                }
                Event::DownloadComplete(pkg) => {
                    self.pkgs.push(pkg);
                    self.build_idea(&mut local_idea_checksum, &mut local_pkg_checksum);
                }
                Event::OutOfIdeas => {
                    if self.skipped_idea || self.idea.is_some() {
                        self.event_sender.send(Event::OutOfIdeas).unwrap();
                        self.skipped_idea = false;
                    } else {
                        for pkg in self.pkgs.drain(..) {
                            self.event_sender
                                .send(Event::DownloadComplete(pkg))
                                .unwrap();
                        }
                        {
                            let mut global_idea = idea_checksum.lock().unwrap();
                            global_idea.merge(local_idea_checksum);
                        }
                        {
                            let mut global_pkg = pkg_checksum.lock().unwrap();
                            global_pkg.merge(local_pkg_checksum);
                        }
                        return;
                    }
                }
            }
        }
    }
}
