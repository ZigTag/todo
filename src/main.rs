//TODO: Please document program
use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};

use std::path::Path;
use std::io::{self, Write};

use ignore::WalkBuilder;

use termcolor::{ColorChoice, StandardStream, WriteColor, ColorSpec, Color};

use clap::{App, Arg};

use git2::Repository;

use time::OffsetDateTime;

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

    let _is_git: bool;

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

    let mut input = String::new();
    writeln!(&mut stdout, "You have {} TODOs.\nWould you like to view them? (y/N)", matches.len()).unwrap();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    if !("yes".starts_with(&input)) {
        std::process::exit(0);
    }

    if let Some(git) = git {
        _is_git = true;

        for (line, text, path) in matches {
            let blame = git
                .blame_file(Path::new(&path).strip_prefix("./").unwrap(), None)
                .unwrap();

            let hunk = blame.get_line(line as usize).unwrap();

            let commit = git.find_commit(hunk.final_commit_id()).unwrap();

            let time = OffsetDateTime::from_unix_timestamp(commit.time().seconds());

            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(224, 131, 65)))).unwrap();
            writeln!(&mut stdout, "{}", text).unwrap();

            stdout.reset().unwrap();
            write!(&mut stdout, "In file ").unwrap();

            stdout.set_color(ColorSpec::new().set_bold(true)).unwrap();
            write!(&mut stdout, "{}", path).unwrap();

            stdout.reset().unwrap();
            write!(&mut stdout, " at line ").unwrap();

            stdout.set_color(ColorSpec::new().set_bold(true)).unwrap();
            write!(&mut stdout, "{}", line).unwrap();

            stdout.reset().unwrap();
            write!(&mut stdout, " and last updated at ").unwrap();

            stdout.set_color(ColorSpec::new().set_bold(true)).unwrap();
            writeln!(
                &mut stdout,
                "{}-{}-{} {} UTC\n",
                time.year(),
                time.month(),
                time.day(),
                time.time()
            )
            .unwrap();

            stdout.reset().unwrap();
        }
    } else {
        _is_git = false;
    }
}
