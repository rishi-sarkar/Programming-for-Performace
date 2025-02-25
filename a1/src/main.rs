use rustyline::error::ReadlineError;
use rustyline::Editor;

use rpkg::debversion;
use crate::packages::Packages;

mod packages;

fn check_syntax(n: usize, cmd_fragments:&Vec<&str>, arg: &str) -> bool {
    let cmd : &str = &cmd_fragments.get(0).unwrap();
    if cmd_fragments.len() != n {
        println!("syntax: {} {}", cmd, arg);
        return false
    }
    return true
}

fn process_command(state: &mut Packages, cmdline: &str) -> bool {
    let cmd_fragments: Vec<&str> = cmdline.split(" ").collect();
    if cmdline.is_empty() { return false }
    let cmd : &str = &cmd_fragments.get(0).unwrap();
    match cmd {
        "quit" => { 
            return true 
        },
        "load-csv" | "lc" => {
            if !check_syntax(2, &cmd_fragments, "<csvfile-name>") { return false; }
            let arg = cmd_fragments.get(1).unwrap();
            state.parse_csv(arg)
        }
        // parsers.rs
        "load-packages" | "lp" => {
            if !check_syntax(2, &cmd_fragments, "<pkgfile-name>") { return false; }
            let arg = cmd_fragments.get(1).unwrap();
            state.parse_packages(arg)
        }
        "load-installed" | "li" => {
            if !check_syntax(2, &cmd_fragments, "<pkgfile-name>") { return false; }
            let arg = cmd_fragments.get(1).unwrap();
            state.parse_installed(arg)
        }
        // convenience function, also depends on parsers.rs
        "load-defaults" | "ld" => {
            state.parse_packages("data/mirror.csclub.uwaterloo.ca_debian_dists_sid_main_binary-amd64_Packages");
            state.parse_installed("data/installed-packages")
        }

        "info" => {
            if !check_syntax(2, &cmd_fragments, "<pkg>") { return false; }
            let pkg = cmd_fragments.get(1).unwrap();
            state.print_info(pkg)
        }
        "deps" => {
            // test: deps 0ad
            if !check_syntax(2, &cmd_fragments, "<pkg>") { return false; }
            let pkg = cmd_fragments.get(1).unwrap();
            state.print_deps(pkg)
        }

        // deps-available.rs
        "deps-available" => {
            // test: deps-available 3depict
            if !check_syntax(2, &cmd_fragments, "<pkg>") { return false; }
            let pkg = cmd_fragments.get(1).unwrap();
            state.deps_available(pkg)
        }

        // solvers.rs, and deps-available.rs for how-to-install
        "transitive-dep-solution" => {
            // test: transitive-dep-solution 0ad
            if !check_syntax(2, &cmd_fragments, "<pkg>") { return false; }
            let pkg = cmd_fragments.get(1).unwrap();
            state.print_transitive_dep_solution(pkg)
        }
        "how-to-install" => {
            if !check_syntax(2, &cmd_fragments, "<pkg>") { return false; }
            let pkg = cmd_fragments.get(1).unwrap();
            state.print_how_to_install(pkg)
        }

        "set-server" => {
            if !check_syntax(2, &cmd_fragments, "<server>") { return false; }
            let server = cmd_fragments.get(1).unwrap();
            state.set_server(server)
        }
        "execute" => {
            state.execute();
        }
        "enq-verify" => {
            let cmd : &str = &cmd_fragments.get(0).unwrap();
            if cmd_fragments.len() < 2 || cmd_fragments.len() > 3 {
                println!("syntax: {} <pkg> [<version>]", cmd);
                return false
            }
            let pkg = cmd_fragments.get(1).unwrap();
            if cmd_fragments.len() == 2 {
                state.enq_verify(pkg);
            } else {
                let version = cmd_fragments.get(2).unwrap();
                state.enq_verify_with_version(pkg, version);
            }
        }

        "output-md5s" => {
            if !check_syntax(2, &cmd_fragments, "<output-file>") { return false; }
            let fname = cmd_fragments.get(1).unwrap();
            state.output_md5s(fname);
        }
        "test-version-compare" => {
            if !check_syntax(3, &cmd_fragments, "<version1> <version2>") { return false; }
            let v1 = cmd_fragments.get(1).unwrap().parse::<debversion::DebianVersionNum>().unwrap();
            let v2 = cmd_fragments.get(2).unwrap().parse::<debversion::DebianVersionNum>().unwrap();
            println!("{} and {}: {:?}", v1, v2, v1.cmp(&v2));
            // 1:0.4.5+cvs20030824-9 vs 1:0.4.5+cvs20030824-10
            // a vs b
            // a vs a
            // b vs a
            // 1-a vs 1-b
            // 2-a vs 1-b
            // a vs ~a
        }
        _ => {
            println!("couldn't understand cmd {:?}", cmd)
        }
    }
    return false;
}

fn main() {
    let mut state : Packages = Packages::new();

    // bonus (0 points): implement command completion!
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {}
    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if process_command(&mut state, &line) { break }
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
