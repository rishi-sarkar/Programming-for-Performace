#![warn(clippy::all)]
use crossbeam::channel::{unbounded, Receiver, Sender};
use lab4::{
    checksum::Checksum, idea::IdeaGenerator, package::PackageDownloader, student::Student, Event,
};
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

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
    
    let mut handles = vec![];

    let mut idea_checksum = Arc::new(Mutex::new(Checksum::default()));
    let mut pkg_checksum = Arc::new(Mutex::new(Checksum::default()));
    let mut student_idea_checksum = Arc::new(Mutex::new(Checksum::default()));
    let mut student_pkg_checksum = Arc::new(Mutex::new(Checksum::default()));

    for i in 0..args.num_students {
        let mut student = Student::new(i, Sender::clone(&sender), Receiver::clone(&receiver));
        let student_idea_checksum = Arc::clone(&student_idea_checksum);
        let student_pkg_checksum = Arc::clone(&student_pkg_checksum);
        handles.push(spawn(move || student.run(student_idea_checksum, student_pkg_checksum)));
    }

    let mut start_idx = 0;
    for i in 0..args.num_pkg_gen {
        let num_pkgs = per_thread_amount(i, args.num_pkgs, args.num_pkg_gen);
        let downloader = PackageDownloader::new(start_idx, num_pkgs, Sender::clone(&sender));
        start_idx += num_pkgs;
        let pkg_checksum_clone = Arc::clone(&pkg_checksum);
        handles.push(spawn(move || downloader.run(pkg_checksum_clone)));
    }
    assert_eq!(start_idx, args.num_pkgs);

    let mut start_idx = 0;
    for i in 0..args.num_idea_gen {
        let num_ideas = per_thread_amount(i, args.num_ideas, args.num_idea_gen);
        let num_pkgs = per_thread_amount(i, args.num_pkgs, args.num_idea_gen);
        let num_students = per_thread_amount(i, args.num_students, args.num_idea_gen);
        let generator = IdeaGenerator::new(
            start_idx,
            num_ideas,
            num_students,
            num_pkgs,
            Sender::clone(&sender),
        );
        let idea_checksum_clone = Arc::clone(&idea_checksum);
        start_idx += num_ideas;
        handles.push(spawn(move || generator.run(idea_checksum_clone)));
    }
    assert_eq!(start_idx, args.num_ideas);

    handles.into_iter().for_each(|t| t.join().unwrap());

    let idea = Arc::get_mut(&mut idea_checksum).unwrap().get_mut().unwrap();
    let student_idea = Arc::get_mut(&mut student_idea_checksum).unwrap().get_mut().unwrap();
    let pkg = Arc::get_mut(&mut pkg_checksum).unwrap().get_mut().unwrap();
    let student_pkg = Arc::get_mut(&mut student_pkg_checksum).unwrap().get_mut().unwrap();

    println!(
        "Global checksums:\nIdea Generator: {}\nStudent Idea: {}\nPackage Downloader: {}\nStudent Package: {}", 
        idea, student_idea, pkg, student_pkg);
}
