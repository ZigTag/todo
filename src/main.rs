//TODO: Please document program
use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};

use std::path::Path;

use ignore::Walk;

fn main() {
    let matcher = RegexMatcher::new(r".*TODO:.*").unwrap();
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();
    let path = Path::new("./");
    let mut matches: Vec<(u64, String, String)> = vec![];

    for path in path {
        for result in Walk::new(path) {
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{}", err);
                    continue;
                }
            };
            if !dent.path().is_file() {
                continue;
            }
            println!("dir '{}'", dent.path().display());
            let result = searcher.search_path(
                &matcher,
                dent.path(),
                UTF8(|line_num, string| {
                    let my_match = matcher.find(string.as_bytes())?.unwrap();
                    matches.push((
                        line_num,
                        string[my_match].trim().to_string(),
                        dent.path().display().to_string(),
                    ));
                    Ok(true)
                }),
            );
            if let Err(err) = result {
                eprintln!("{}: {}", dent.path().display(), err);
            }
        }
    }

    for (line, text, path) in matches {
        println!("\nFile: '{}'\nLine: {}\nText: '{}'", path, line, text);
    }
}
