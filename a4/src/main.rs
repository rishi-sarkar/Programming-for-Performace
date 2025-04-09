#![warn(clippy::all)]
use lab4::{
    checksum::Checksum, idea::IdeaGenerator, package::PackageDownloader, student::Student, Event,
};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::env;
use std::error::Error;
use std::fs;
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
    let args: Vec<String> = env::args().collect();
    let num_ideas = args.get(1).map_or(Ok(80), |a| a.parse())?;
    let num_idea_gen = args.get(2).map_or(Ok(2), |a| a.parse())?;
    let num_pkgs = args.get(3).map_or(Ok(4000), |a| a.parse())?;
    let num_pkg_gen = args.get(4).map_or(Ok(6), |a| a.parse())?;
    let num_students = args.get(5).map_or(Ok(6), |a| a.parse())?;
    let config = Args {
        num_ideas,
        num_idea_gen,
        num_pkgs,
        num_pkg_gen,
        num_students,
    };

    run_hackathon(&config);
    Ok(())
}

// Returns the number of items allocated to a thread based on its index.
fn per_thread_amount(idx: usize, total: usize, count: usize) -> usize {
    let base = total / count;
    let extra = total % count;
    base + (idx < extra) as usize
}

fn run_hackathon(args: &Args) {
    // Create an event channel.
    let (send, recv): (Sender<Event>, Receiver<Event>) = unbounded();

    // Preload idea data (products and customers) and compute the cross product.
    let products = fs::read_to_string("data/ideas-products.txt")
        .expect("Failed to load data/ideas-products.txt");
    let customers = fs::read_to_string("data/ideas-customers.txt")
        .expect("Failed to load data/ideas-customers.txt");
    let cross: Vec<(String, String)> = products
        .lines()
        .flat_map(|p| customers.lines().map(move |c| (p.to_owned(), c.to_owned())))
        .collect();
    let _idea_cross = Arc::new(cross); // This shared data could be used by idea generators if needed.

    // Preload package data once.
    let packages = fs::read_to_string("data/packages.txt")
        .expect("Failed to load data/packages.txt");
    let pkg_lines: Vec<String> = packages.lines().map(|s| s.to_owned()).collect();
    let _pkg_shared = Arc::new(pkg_lines); // Likewise, this can be shared with package downloaders.

    // Global checksum containers (declared mutable to allow extraction via Arc::get_mut later).
    let mut idea_checksum = Arc::new(Mutex::new(Checksum::default()));
    let mut pkg_checksum = Arc::new(Mutex::new(Checksum::default()));
    let mut student_idea_checksum = Arc::new(Mutex::new(Checksum::default()));
    let mut student_pkg_checksum = Arc::new(Mutex::new(Checksum::default()));

    // Vector to hold thread handles.
    let mut threads = Vec::new();

    // Spawn student threads.
    for id in 0..args.num_students {
        let mut student = Student::new(id, send.clone(), recv.clone());
        let stud_idea = Arc::clone(&student_idea_checksum);
        let stud_pkg = Arc::clone(&student_pkg_checksum);
        let handle = spawn(move || {
            student.run(stud_idea, stud_pkg);
        });
        threads.push(handle);
    }

    // Spawn package downloader threads.
    let mut pkg_start_idx = 0;
    for i in 0..args.num_pkg_gen {
        let num = per_thread_amount(i, args.num_pkgs, args.num_pkg_gen);
        let downloader = PackageDownloader::new(pkg_start_idx, num, send.clone());
        pkg_start_idx += num;
        let pkg_sum = Arc::clone(&pkg_checksum);
        let handle = spawn(move || {
            downloader.run(pkg_sum);
        });
        threads.push(handle);
    }
    assert_eq!(pkg_start_idx, args.num_pkgs);

    // Spawn idea generator threads.
    let mut idea_start_idx = 0;
    for i in 0..args.num_idea_gen {
        let ideas_alloc = per_thread_amount(i, args.num_ideas, args.num_idea_gen);
        let pkgs_alloc = per_thread_amount(i, args.num_pkgs, args.num_idea_gen);
        let studs_alloc = per_thread_amount(i, args.num_students, args.num_idea_gen);
        let generator = IdeaGenerator::new(
            idea_start_idx,
            ideas_alloc,
            studs_alloc,
            pkgs_alloc,
            send.clone(),
        );
        idea_start_idx += ideas_alloc;
        let idea_sum = Arc::clone(&idea_checksum);
        let handle = spawn(move || {
            generator.run(idea_sum);
        });
        threads.push(handle);
    }
    assert_eq!(idea_start_idx, args.num_ideas);

    // Wait for all threads to complete.
    for th in threads {
        th.join().expect("A thread panicked");
    }

    // Extract final checksum values.
    // (We use Arc::get_mut because all other clones have been dropped by now.)
    let final_idea = Arc::get_mut(&mut idea_checksum).unwrap().get_mut().unwrap().to_string();
    let final_stud_idea = Arc::get_mut(&mut student_idea_checksum)
        .unwrap()
        .get_mut()
        .unwrap()
        .to_string();
    let final_pkg = Arc::get_mut(&mut pkg_checksum).unwrap().get_mut().unwrap().to_string();
    let final_stud_pkg = Arc::get_mut(&mut student_pkg_checksum)
        .unwrap()
        .get_mut()
        .unwrap()
        .to_string();

    println!("Global checksums:");
    println!("Idea Generator: {}", final_idea);
    println!("Student Idea: {}", final_stud_idea);
    println!("Package Downloader: {}", final_pkg);
    println!("Student Package: {}", final_stud_pkg);
}
