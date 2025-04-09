use super::checksum::Checksum;
use super::Event;
use crossbeam::channel::Sender;
use std::fs;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static PRODUCTS: Lazy<String> = Lazy::new(|| {
    fs::read_to_string("data/ideas-products.txt").expect("file not found: ideas-products.txt")
});
static CUSTOMERS: Lazy<String> = Lazy::new(|| {
    fs::read_to_string("data/ideas-customers.txt").expect("file not found: ideas-customers.txt")
});

static CROSS_PROD: Lazy<Vec<(String, String)>> = Lazy::new(|| {
    PRODUCTS
        .lines()
        .flat_map(|p| {
            CUSTOMERS.lines().map(move |c| (p.to_owned(), c.to_owned()))
        })
        .collect()
});

pub struct Idea {
    pub name: String,
    pub num_pkg_required: usize,
}

pub struct IdeaGenerator {
    idea_start_idx: usize,
    num_ideas: usize,
    num_students: usize,
    num_pkgs: usize,
    event_sender: Sender<Event>,
}

impl IdeaGenerator {
    pub fn new(
        idea_start_idx: usize,
        num_ideas: usize,
        num_students: usize,
        num_pkgs: usize,
        event_sender: Sender<Event>,
    ) -> Self {
        Self {
            idea_start_idx,
            num_ideas,
            num_students,
            num_pkgs,
            event_sender,
        }
    }

    // Idea names are generated from cross products between product names and customer names
    fn get_next_idea_name(idx: usize) -> String {
        let pair = &CROSS_PROD[idx % CROSS_PROD.len()];
        format!("{} for {}", pair.0, pair.1)
    }

    pub fn run(&self, idea_checksum: Arc<Mutex<Checksum>>) {
        // We'll compute the number of packages per idea
        let pkg_per_idea = self.num_pkgs / self.num_ideas;
        let extra_pkgs = self.num_pkgs % self.num_ideas;

        // 2) Use a local Checksum to reduce how often we lock the global one
        let mut local_checksum = Checksum::default();

        // Generate ideas & update local checksum
        for i in 0..self.num_ideas {
            let name = Self::get_next_idea_name(self.idea_start_idx + i);

            let extra = (i < extra_pkgs) as usize;
            let num_pkg_required = pkg_per_idea + extra;

            // We'll store the idea to send via event
            let idea = Idea {
                name,
                num_pkg_required,
            };

            // IMPORTANT: This call simulates "download time." Must keep it!
            // But we'll update our local checksum rather than locking each time.
            local_checksum.update(Checksum::with_sha256(&idea.name));

            // Send the new idea event
            self.event_sender.send(Event::NewIdea(idea)).unwrap();
        }

        // Now lock the global checksum once and merge our local work.
        {
            let mut global_checksum = idea_checksum.lock().unwrap();
            global_checksum.update(local_checksum);
        }

        // Finally, push student-termination events (one per student).
        for _ in 0..self.num_students {
            self.event_sender.send(Event::OutOfIdeas).unwrap();
        }
    }
}