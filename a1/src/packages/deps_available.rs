use crate::packages::Dependency;
use crate::Packages;
use rpkg::debversion;

impl Packages {
    /// Gets the dependencies of package_name, and prints out whether they are satisfied (and by which library/version) or not.
    pub fn deps_available(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        println!("Package {}:", package_name);

        let dependencies: &Vec<Dependency> = self
            .dependencies
            .get(self.get_package_num(package_name))
            .unwrap();
        for dependency in dependencies {
            println!("- dependency {:?}", self.dep2str(dependency));
            match self.dep_is_satisfied(dependency) {
                None => {
                    println!("-> not satisfied");
                }
                Some(dependency) => {
                    println!(
                        "+ {} satisfied by installed version {}",
                        dependency,
                        self.get_installed_debver(dependency).unwrap()
                    );
                }
            }
        }
    }

    /// Returns Some(package) which satisfies dependency dd, or None if not satisfied.
    pub fn dep_is_satisfied(&self, dd: &Dependency) -> Option<&str> {
        for alternative in dd {
            if self
                .installed_debvers
                .contains_key(&alternative.package_num)
            {
                match &alternative.rel_version {
                    None => {
                        continue;
                    }
                    Some((op, required_version)) => {
                        let iv = self
                            .installed_debvers
                            .get(&alternative.package_num)
                            .unwrap();
                        let v = required_version
                            .parse::<debversion::DebianVersionNum>()
                            .unwrap();
                        if debversion::cmp_debversion_with_op(op, iv, &v) {
                            return Some(self.get_package_name(alternative.package_num));
                        }
                    }
                }
            }
        }
        return None;
    }

    /// Returns a Vec of packages which would satisfy dependency dd but for the version.
    /// Used by the how-to-install command, which calls compute_how_to_install().
    pub fn dep_satisfied_by_wrong_version(&self, dd: &Dependency) -> Vec<&str> {
        assert!(self.dep_is_satisfied(dd).is_none());
        let mut result = vec![];
        for alternative in dd {
            match &alternative.rel_version {
                None => {
                    continue;
                }
                Some((op, required_version)) => {
                    let v = required_version
                        .parse::<debversion::DebianVersionNum>()
                        .unwrap();
                    let iv = self
                        .installed_debvers
                        .get(&alternative.package_num)
                        .unwrap();
                    if !debversion::cmp_debversion_with_op(op, iv, &v) {
                        result.push(self.get_package_name(alternative.package_num));
                    }
                }
            }
        }
        return result;
    }
}
