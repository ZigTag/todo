//TODO: Please document program
use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};

use std::io::Write;
use std::path::Path;

use ignore::WalkBuilder;

use termcolor::{ColorChoice, StandardStream};

use clap::{App, Arg};

use git2::Repository;

fn main() {
    let args = App::new("todo")
        .version("0.1.0")
        .author("ZigTag <GitHub>")
        .about("Reads out your current TODOs")
        .arg(
            Arg::with_name("path")
                .short("d")
                .long("path")
                .value_name("DIR")
                .help("Sets the working directory. (optional)")
                .takes_value(true)
                .required(false)
                .default_value("./")
                .index(1),
        )
        .arg(
            Arg::with_name("color")
                .long("color")
                .value_name("bool")
                .help("Display text color.")
                .default_value("true"),
        )
        .arg(
            Arg::with_name("show_hidden")
                .short("h")
                .long("show-hidden")
                .help("Includes hidden files."),
        )
        .get_matches();

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let matcher = RegexMatcher::new(r".*TODO:.*").unwrap();
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();
    let path = Path::new(args.value_of("path").unwrap());
    let mut matches: Vec<(usize, String, String)> = vec![];
    let show_hidden = !args.is_present("show_hidden");

    let is_git: bool;

    let git = match Repository::open(path) {
        Ok(git) => Some(git),
        Err(_) => None,
    };

    for path in path {
        for result in WalkBuilder::new(path).hidden(show_hidden).build() {
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
            writeln!(&mut stdout, "dir '{}'", dent.path().display()).unwrap();
            let result = searcher.search_path(
                &matcher,
                dent.path(),
                UTF8(|line_num, string| {
                    let my_match = matcher.find(string.as_bytes())?.unwrap();
                    matches.push((
                        line_num as usize,
                        string[my_match].trim().to_string(),
                        dent.path().to_str().unwrap().to_string(),
                    ));
                    Ok(true)
                }),
            );
            if let Err(err) = result {
                eprintln!("{}: {}", dent.path().display(), err);
            }
        }
    }

    if let Some(git) = git {
        is_git = true;

        for (line, text, path) in matches {
            let blame = git
                .blame_file(Path::new(&path).strip_prefix("./").unwrap(), None)
                .unwrap();

            let hunk = blame.get_line(line as usize).unwrap();

            let commit = hunk.final_commit_id();

            writeln!(&mut stdout, "commit {}", commit).unwrap();
        }
    } else {
        is_git = false;
    }
}
