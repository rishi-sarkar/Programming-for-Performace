#![warn(clippy::all)]
pub mod checksum;
pub mod idea;
pub mod package;
pub mod student;

use idea::Idea;
use package::Package;

pub enum Event {
    NewIdea(Idea),
    OutOfIdeas,
    DownloadComplete(Package),
}
