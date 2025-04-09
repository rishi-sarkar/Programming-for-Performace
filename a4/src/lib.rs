#![warn(clippy::all)]
pub mod checksum;
pub mod idea;
pub mod package;
pub mod student;

use idea::Idea;
use package::Package;

pub enum Event {
    // Newly generated idea for students to work on
    NewIdea(Idea),
    // Termination event for student threads
    OutOfIdeas,
    // Packages that students can take to work on their ideas
    DownloadComplete(Package),
}
