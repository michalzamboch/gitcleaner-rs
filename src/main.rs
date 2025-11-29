use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

use walkdir::WalkDir;

fn usage(cmd: &str) {
    eprintln!("Usage: {} [--apply] [start-path]", cmd);
    eprintln!("\nBy default the tool performs a dry-run and only lists repositories.");
    eprintln!("Use --apply (or -a) to actually run `git clean -dxf` in each repo.");
}

fn main() {
    let mut args = env::args().skip(1);
    let mut apply = false;
    let mut start_path = env::current_dir().expect("failed to get current dir");

    // Simple arg parsing: --help, --apply/-a, optional start-path
    while let Some(a) = args.next() {
        match a.as_str() {
            "--help" | "-h" => {
                usage(&env::args().next().unwrap_or_else(|| "gitcleaner-rs".into()));
                return;
            }
            "--apply" | "-a" => apply = true,
            other if start_path == env::current_dir().unwrap() && !other.starts_with('-') => {
                start_path = PathBuf::from(other);
            }
            _ => {
                eprintln!("Unknown option: {}", a);
                usage(&env::args().next().unwrap_or_else(|| "gitcleaner-rs".into()));
                std::process::exit(2);
            }
        }
    }

    eprintln!("Scanning '{}' for git repositories...", start_path.display());

    // Collect repo roots so we don't run the command multiple times for nested entries
    let mut repos: HashSet<PathBuf> = HashSet::new();

    for entry in WalkDir::new(&start_path).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        let file_name = entry.file_name();

        if file_name == OsStr::new(".git") {
            if let Some(parent) = entry.path().parent() {
                repos.insert(parent.to_path_buf());
            }
        }
    }

    if repos.is_empty() {
        eprintln!("No git repositories found under '{}'.", start_path.display());
        return;
    }

    let mut failed = Vec::new();

    let mut repos: Vec<_> = repos.into_iter().collect();
    repos.sort();

    for repo in repos {
        eprintln!("Found repo: {}", repo.display());
        if !apply {
            println!("DRY-RUN: would run `git clean -dxf` in: {}", repo.display());
            continue;
        }

        println!("Running `git clean -dxf` in: {}", repo.display());
        let status = Command::new("git")
            .arg("clean")
            .arg("-dxf")
            .current_dir(&repo)
            .status();

        match status {
            Ok(s) if s.success() => println!("Succeeded: {}", repo.display()),
            Ok(s) => {
                eprintln!("git returned non-zero ({}) for {}", s, repo.display());
                failed.push(repo);
            }
            Err(err) => {
                eprintln!("Failed to run git in {}: {}", repo.display(), err);
                failed.push(repo);
            }
        }
    }

    if !failed.is_empty() {
        eprintln!("Finished with failures for {} repos.", failed.len());
        std::process::exit(1);
    }

    eprintln!("Done.");
}
