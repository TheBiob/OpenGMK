mod action;
mod asset;
mod game;
mod gml;
mod handleman;
mod imgui;
mod input;
mod instance;
mod instancelist;
mod math;
mod render;
mod tile;
mod types;
mod util;

use game::replay::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

fn help(argv0: &str, opts: getopts::Options) {
    print!(
        "{}",
        opts.usage(&format!("Usage: {} FILE [options]", match Path::new(argv0).file_name() {
            Some(file) => file.to_str().unwrap_or(argv0),
            None => argv0,
        }))
    );
}

fn main() {
    process::exit(xmain());
}

fn xmain() -> i32 {
    let args: Vec<String> = env::args().collect();
    let process = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "prints this help message");
    opts.optopt("v", "version", "which version the input file is. Supported: v0, gmtas_any. gmtas_any for automatic detection.", "gmtas_any");
    opts.optopt("f", "replay-file", "path to gmtas file to convert", "save.gmtas");
    opts.optopt("o", "output-file", "output gmtas name", "save-v1.gmtas");

    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(fail) => {
            use getopts::Fail::*;
            match fail {
                ArgumentMissing(arg) => eprintln!("missing argument {}", arg),
                UnrecognizedOption(opt) => eprintln!("unrecognized option {}", opt),
                OptionMissing(opt) => eprintln!("missing option {}", opt),
                OptionDuplicated(opt) => eprintln!("duplicated option {}", opt),
                UnexpectedArgument(arg) => eprintln!("unexpected argument {}", arg),
            }
            return EXIT_FAILURE
        },
    };

    if args.len() < 2 || matches.opt_present("h") {
        help(&process, opts);
        return EXIT_SUCCESS
    }

    let version = matches.opt_str("v").unwrap();
    let input_file = &matches.opt_str("f").map(PathBuf::from).unwrap();
    let output_file = &matches.opt_str("o").map(PathBuf::from).unwrap();

    if version == "gmtas_any" {
        convert_any_replay(input_file, output_file);
    } else if version == "v0" {
        convert_v0_replay(input_file, output_file);
    } else {
        help(&process, opts);
    }

    EXIT_SUCCESS
}

fn convert_any_replay(input: &PathBuf, output: &PathBuf) {
    match Replay::from_file(input) {
        Ok(v) => {v.to_file(output);},
        Err(v) => {println!("Error readon v1, trying v0"); convert_v0_replay(input, output);},
    }
}
fn convert_v0_replay(input: &PathBuf, output: &PathBuf) {
    match ReplayV0::from_file(input) {
        Ok(v) => {Replay::from(v).to_file(output);},
        Err(v) => println!("Error reading v0 replay. Input file does not match known format."),
    }
}
