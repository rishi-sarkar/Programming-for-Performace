use super::checksum::Checksum;
use super::Event;
use crossbeam::channel::Sender;
use std::fs;
use std::sync::{Arc, Mutex};

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
        let products = fs::read_to_string("data/ideas-products.txt").expect("file not found");
        let customers = fs::read_to_string("data/ideas-customers.txt").expect("file not found");
        let ideas = Self::cross_product(products, customers);
        let pair = &ideas[idx % ideas.len()];
        format!("{} for {}", pair.0, pair.1)
    }

    fn cross_product(products: String, customers: String) -> Vec<(String, String)> {
        products
            .lines()
            .flat_map(|p| customers.lines().map(move |c| (p.to_owned(), c.to_owned())))
            .collect()
    }

    pub fn run(&self, idea_checksum: Arc<Mutex<Checksum>>) {
        let pkg_per_idea = self.num_pkgs / self.num_ideas;
        let extra_pkgs = self.num_pkgs % self.num_ideas;

        // Generate a set of new ideas and place them into the event-queue
        // Update the idea checksum with all generated idea names
        for i in 0..self.num_ideas {
            let name = Self::get_next_idea_name(self.idea_start_idx + i);
            let extra = (i < extra_pkgs) as usize;
            let num_pkg_required = pkg_per_idea + extra;
            let idea = Idea {
                name,
                num_pkg_required,
            };

            idea_checksum
                .lock()
                .unwrap()
                .update(Checksum::with_sha256(&idea.name));

            self.event_sender.send(Event::NewIdea(idea)).unwrap();
        }

        // Push student termination events into the event queue
        for _ in 0..self.num_students {
            self.event_sender.send(Event::OutOfIdeas).unwrap();
        }
    }
}
