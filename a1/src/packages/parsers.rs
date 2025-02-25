use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use regex::Regex;

use crate::packages::RelVersionedPackageNum;
use crate::Packages;

use rpkg::debversion;

const KEYVAL_REGEX: &str = r"^(?P<key>(\w|-)+): (?P<value>.+)";
const PKGNAME_AND_VERSION_REGEX: &str =
    r"(?P<pkg>(\w|\.|\+|-)+)( \((?P<op>(<|=|>)(<|=|>)?) (?P<ver>.*)\))?";

impl Packages {
    /// Loads packages and version numbers from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate value into the installed_debvers map with the parsed version number.
    pub fn parse_installed(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num: i32 = 0;
            for line in lines {
                if let Ok(ip) = line {
                    match kv_regexp.captures(&ip) {
                        None => (),
                        Some(caps) => {
                            let (key, value) = (
                                caps.name("key").unwrap().as_str(),
                                caps.name("value").unwrap().as_str(),
                            );
                            if key == "Package" {
                                current_package_num = self.get_package_num_inserting(&value);
                            }
                            // println!("package #:{}", current_package_num);
                            if key == "Version" {
                                let debver = value
                                    .trim()
                                    .parse::<debversion::DebianVersionNum>()
                                    .unwrap();
                                self.installed_debvers.insert(current_package_num, debver);
                            }
                        }
                    }
                }
            }
        }
        println!(
            "Packages installed: {}",
            self.installed_debvers.keys().len()
        );
    }

    /// Loads packages, version numbers, dependencies, and md5sums from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate values into the dependencies, md5sum, and available_debvers maps.
    pub fn parse_packages(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        let pkgver_regexp = Regex::new(PKGNAME_AND_VERSION_REGEX).unwrap();

        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                    match kv_regexp.captures(&ip) {
                        None => (),
                        Some(caps) => {
                            let (key, value) = (
                                caps.name("key").unwrap().as_str(),
                                caps.name("value").unwrap().as_str(),
                            );
                            if key == "Package" {
                                current_package_num = self.get_package_num_inserting(&value);
                            }
                            if key == "Version" {
                                let debver = value
                                    .trim()
                                    .parse::<debversion::DebianVersionNum>()
                                    .unwrap();
                                self.available_debvers.insert(current_package_num, debver);
                            }
                            if key == "MD5sum" {
                                self.md5sums
                                    .insert(current_package_num, String::from(value));
                            }
                            if key == "Depends" {
                                let mut dependencies = vec![];
                                for dependency in value.split(",") {
                                    let mut alternatives = vec![];
                                    for alternative in dependency.split("|") {
                                        match pkgver_regexp.captures(alternative) {
                                            None => {}
                                            Some(cap) => {
                                                let package = cap.name("pkg").unwrap().as_str();
                                                let package_num = self.get_package_num_inserting(package);
                                                let mut rel_version = Option::None;

                                                match cap.name("op") {
                                                    None => {}
                                                    Some(option) => {
                                                        let op: debversion::VersionRelation = option
                                                            .as_str()
                                                            .parse::<debversion::VersionRelation>()
                                                            .unwrap();
                                                        let ver: String = cap
                                                            .name("ver")
                                                            .unwrap()
                                                            .as_str()
                                                            .to_string();
                                                        rel_version = Option::Some((op, ver));
                                                    }
                                                }
                                                alternatives.push(RelVersionedPackageNum {package_num, rel_version});
                                            }
                                        }
                                    }
                                    dependencies.push(alternatives);
                                }
                                self.dependencies.insert(current_package_num, dependencies);
                            }
                        }
                    }
                }
            }
        }
        println!(
            "Packages available: {}",
            self.available_debvers.keys().len()
        );
    }
}

// standard template code downloaded from the Internet somewhere
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
