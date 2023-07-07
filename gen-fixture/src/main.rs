use std::path::Path;

use gen_fixture::units;
use path_dedot::ParseDot;

const DEFAULT_OUTPUT_DIR: &str = "fixtures";

fn print_help() {
    println!("cargo run <size> [output_dir]");
    println!();
    println!("  size:        The size of the fixture to generate. Unit can be specified as");
    println!("               'kib', 'mib', or 'gib'. Defaults to bytes.");
    println!("  output_dir:  The directory to place the file in, relative to the project");
    println!("               root. default={}", DEFAULT_OUTPUT_DIR);
}

fn main() {
    let command_dir = env!("CARGO_MANIFEST_DIR");
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.iter().find(is_help_flag).is_some() {
        print_help();
        return;
    }

    if args.len() < 1 {
        println!("Error: missing argument");
        println!();
        print_help();
        return;
    }

    if args.len() > 2 {
        println!("Error: too many arguments");
        println!();
        print_help();
        return;
    }

    let size = match args[0].parse::<units::Bytes>() {
        Ok(size) => size,
        Err(e) => {
            println!("{}", e);
            println!();
            return;
        }
    };

    let output_dir = if args.len() >= 2 {
        Path::new(command_dir).join("..").join(args[1].as_str())
    } else {
        Path::new(command_dir).join("..").join(DEFAULT_OUTPUT_DIR)
    };

    let output_dir = output_dir.parse_dot().unwrap();

    gen_fixture::gen_fixture(&output_dir, &size).unwrap();
}

fn is_help_flag(arg: &&String) -> bool {
    *arg == "-h" || *arg == "--help"
}
