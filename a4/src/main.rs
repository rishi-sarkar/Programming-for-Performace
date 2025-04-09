#![warn(clippy::all)]
use lab4::{
    checksum::Checksum, idea::IdeaGenerator, package::PackageDownloader, student::Student, Event,
};
use crossbeam::channel::unbounded;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

struct Args {
    pub num_ideas: usize,
    pub num_idea_gen: usize,
    pub num_pkgs: usize,
    pub num_pkg_gen: usize,
    pub num_students: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    let num_ideas = args.get(1).map_or(Ok(80), |a| a.parse())?;
    let num_idea_gen = args.get(2).map_or(Ok(2), |a| a.parse())?;
    let num_pkgs = args.get(3).map_or(Ok(4000), |a| a.parse())?;
    let num_pkg_gen = args.get(4).map_or(Ok(6), |a| a.parse())?;
    let num_students = args.get(5).map_or(Ok(6), |a| a.parse())?;
    let args = Args {
        num_ideas,
        num_idea_gen,
        num_pkgs,
        num_pkg_gen,
        num_students,
    };

    hackathon(&args);
    Ok(())
}

fn per_thread_amount(thread_idx: usize, total: usize, threads: usize) -> usize {
    let per_thread = total / threads;
    let extras = total % threads;
    per_thread + (thread_idx < extras) as usize
}

fn hackathon(args: &Args) {
    let (sender, receiver) = unbounded::<Event>();

    let idea_checksum = Arc::new(Mutex::new(Checksum::default()));
    let pkg_checksum = Arc::new(Mutex::new(Checksum::default()));
    let student_idea_checksum = Arc::new(Mutex::new(Checksum::default()));
    let student_pkg_checksum = Arc::new(Mutex::new(Checksum::default()));

    let mut handles = Vec::new();

    for i in 0..args.num_students {
        let mut student = Student::new(i, sender.clone(), receiver.clone());
        let student_idea_checksum = Arc::clone(&student_idea_checksum);
        let student_pkg_checksum = Arc::clone(&student_pkg_checksum);
        handles.push(thread::spawn(move || {
            student.run(student_idea_checksum, student_pkg_checksum)
        }));
    }

    let mut pkg_start_idx = 0;
    for i in 0..args.num_pkg_gen {
        let num_pkgs = per_thread_amount(i, args.num_pkgs, args.num_pkg_gen);
        let downloader = PackageDownloader::new(pkg_start_idx, num_pkgs, sender.clone());
        pkg_start_idx += num_pkgs;
        let pkg_checksum = Arc::clone(&pkg_checksum);
        handles.push(thread::spawn(move || downloader.run(pkg_checksum)));
    }
    assert_eq!(pkg_start_idx, args.num_pkgs);

    let mut idea_start_idx = 0;
    for i in 0..args.num_idea_gen {
        let num_ideas = per_thread_amount(i, args.num_ideas, args.num_idea_gen);
        let num_pkgs = per_thread_amount(i, args.num_pkgs, args.num_idea_gen);
        let num_students = per_thread_amount(i, args.num_students, args.num_idea_gen);
        let generator = IdeaGenerator::new(
            idea_start_idx,
            num_ideas,
            num_students,
            num_pkgs,
            sender.clone(),
        );
        idea_start_idx += num_ideas;
        let idea_checksum = Arc::clone(&idea_checksum);
        handles.push(thread::spawn(move || generator.run(idea_checksum)));
    }
    assert_eq!(idea_start_idx, args.num_ideas);

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let idea = idea_checksum.lock().unwrap().to_string();
    let student_idea = student_idea_checksum.lock().unwrap().to_string();
    let pkg = pkg_checksum.lock().unwrap().to_string();
    let student_pkg = student_pkg_checksum.lock().unwrap().to_string();

    println!(
        "Global checksums:\nIdea Generator: {}\nStudent Idea: {}\nPackage Downloader: {}\nStudent Package: {}",
        idea, student_idea, pkg, student_pkg
    );
}
