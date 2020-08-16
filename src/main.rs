//TODO: Please document program
use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};

use std::path::Path;
use std::io::Write;

use ignore::WalkBuilder;

use termcolor::{StandardStream, ColorChoice};

use clap::{App, Arg};

fn main() {
    let args = App::new("todo")
        .version("0.1.0")
        .author("ZigTag <GitHub>")
        .about("Reads out your current TODOs")
        .arg(Arg::with_name("path")
            .short("d")
            .long("path")
            .value_name("DIR")
            .help("Sets the working directory. (optional)")
            .takes_value(true)
            .required(false)
            .default_value("./")
            .index(1))
        .arg(Arg::with_name("color")
            .long("color")
            .value_name("bool")
            .help("Display text color.")
            .default_value("true"))
        .arg(Arg::with_name("show_hidden")
            .short("h")
            .long("show-hidden")
            .help("Includes hidden files."))
        .get_matches();

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let matcher = RegexMatcher::new(r".*TODO:.*").unwrap();
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();
    let path = Path::new(args.value_of("path").unwrap());
    let mut matches: Vec<(u64, String, String)> = vec![];
    let show_hidden = !args.is_present("show_hidden");

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
        writeln!(&mut stdout, "\nFile: '{}'\nLine: {}\nText: '{}'", path, line, text).unwrap();
    }
}
