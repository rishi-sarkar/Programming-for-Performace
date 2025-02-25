use crate::packages::Dependency;
use crate::Packages;
use rpkg::debversion::DebianVersionNum;
use std::collections::VecDeque;

impl Packages {
    /// Computes a solution for the transitive dependencies of package_name; when there is a choice A | B | C,
    /// chooses the first option A. Returns a Vec<i32> of package numbers.
    ///
    /// Note: does not consider which packages are installed.
    pub fn transitive_dep_solution(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }

        let deps: &Vec<Dependency> = &*self
            .dependencies
            .get(self.get_package_num(package_name))
            .unwrap();
        let mut dependency_set = vec![];
        let mut worklist = Vec::new();

        for dependency in deps {
            match dependency.first() {
                None => {
                    continue;
                }
                Some(first_option) => {
                    worklist.push(first_option);
                }
            }
        }

        while let Some(first_option) = worklist.pop() {
            dependency_set.push(first_option.package_num);
            if let Some(dependencies) = self.dependencies.get(&first_option.package_num) {
                for dependency in dependencies {
                    match dependency.first() {
                        None => {
                            continue;
                        }
                        Some(first_option) => {
                            if !dependency_set.contains(&first_option.package_num) {
                                worklist.push(first_option);
                            }
                        }
                    }
                }
            }
        }
        return dependency_set;
    }

    /// Computes a set of packages that need to be installed to satisfy package_name's deps given the current installed packages.
    /// When a dependency A | B | C is unsatisfied, there are two possible cases:
    ///   (1) there are no versions of A, B, or C installed; pick the alternative with the highest version number (yes, compare apples and oranges).
    ///   (2) at least one of A, B, or C is installed (say A, B), but with the wrong version; of the installed packages (A, B), pick the one with the highest version number.
    pub fn compute_how_to_install(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }
        let mut dependencies_to_add: Vec<i32> = vec![];

        let mut worklist: VecDeque<i32> = VecDeque::new();

        worklist.push_back(*self.get_package_num(package_name));

        while let Some(current_package_num) = worklist.pop_front() {
            if dependencies_to_add.contains(&current_package_num) {
                continue;
            }

            dependencies_to_add.push(current_package_num);

            if let Some(dependencies) = self.dependencies.get(&current_package_num) {
                for dependency in dependencies {
                    if self.dep_is_satisfied(dependency).is_none() {
                        let selected_package = self.select_dependency(dependency);
                        if let Some(package_num) = selected_package {
                            worklist.push_back(package_num);
                        }
                    }
                }
            }

            if current_package_num == *self.get_package_num(package_name) {
                dependencies_to_add.pop();
            }
        }
        return dependencies_to_add;
    }

    fn select_dependency(&self, dep: &Dependency) -> Option<i32> {
        let mut best_package_num: Option<i32> = None;
        let mut highest_version: Option<&DebianVersionNum> = None;

        let wrong_version_packages: Vec<String> = self.dep_satisfied_by_wrong_version(dep)
            .into_iter()
            .map(|num| num.to_string())
            .collect();

        for alternative in dep {
            let package_num = alternative.package_num;

            let is_wrong_version_installed = wrong_version_packages.contains(&package_num.to_string());

            if is_wrong_version_installed || self.available_debvers.contains_key(&package_num) {
                if let Some(version) = self.available_debvers.get(&package_num) {
                    if highest_version.is_none() || version > highest_version.unwrap() {
                        best_package_num = Some(package_num);
                        highest_version = Some(version);
                    }
                }
            }
        }
        return best_package_num;
    }

}
