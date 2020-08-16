use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};

use std::path::Path;

use walkdir::WalkDir;

use git2::Repository;

fn main() {
    let matcher = RegexMatcher::new(r".*TODO:.*").unwrap();
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();
    let path = Path::new("./");
    let mut matches: Vec<(u64, String)> = vec![];

    let _is_git: bool;

    let git = match Repository::open(path) {
        Ok(repo) => Some(repo),
        Err(_) => None,
    };

    match git {
        Some(git) => {
            _is_git = true;

            for path in path {
                //TODO: Gitignore is not working properly

                for result in WalkDir::new(path).into_iter().filter(|e| {
                    git.status_should_ignore(e.as_ref().unwrap().path())
                        .unwrap()
                }) {
                    let dent = match result {
                        Ok(dent) => dent,
                        Err(err) => {
                            eprintln!("{}", err);
                            continue;
                        }
                    };
                    if !dent.file_type().is_file() {
                        continue;
                    }
                    println!("after '{}'", dent.path().display());
                    let result = searcher.search_path(
                        &matcher,
                        dent.path(),
                        UTF8(|line_num, string| {
                            let my_match = matcher.find(string.as_bytes())?.unwrap();
                            matches.push((line_num, string[my_match].trim().to_string()));
                            Ok(true)
                        }),
                    );
                    if let Err(err) = result {
                        eprintln!("{}: {}", dent.path().display(), err);
                    }
                }
            }
        }
        None => {
            _is_git = false;
        }
    }

    println!("{:?}", matches);
}
