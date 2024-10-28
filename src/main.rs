use std::{env, fmt::Display, fs, path::Path, process::exit};

use colored::Colorize;

use ufel::{InputSrc, Ufel};

fn fail<T>(e: impl Display) -> T {
    eprintln!("{e}");
    exit(1);
}

enum Command {
    Run,
    Watch,
    Help,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let args_strs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut args = args_strs.as_slice();
    let mut path: Option<&Path> = None;
    let mut help = false;
    let mut command = None;
    loop {
        match args {
            [] => break,
            ["watch", rest @ ..] => {
                command = Some(Command::Watch);
                args = rest;
            }
            ["run", rest @ ..] => {
                command = Some(Command::Run);
                args = rest;
            }
            ["help", rest @ ..] => {
                command = Some(Command::Help);
                args = rest;
            }
            ["-h" | "--help", rest @ ..] => {
                help = true;
                args = rest;
            }
            &[pth, ref rest @ ..] if !pth.starts_with('-') && path.is_none() => {
                path = Some(Path::new(pth));
                args = rest;
            }
            args => {
                eprint!("Invalid argument{}", if args.len() > 1 { "s" } else { "" });
                for arg in args {
                    eprint!(" {arg:?}");
                }
                eprintln!();
                show_help();
                exit(1);
            }
        }
    }

    match command {
        Some(Command::Run) => run_maybe_path(path, false),
        Some(Command::Watch) => todo!(),
        Some(Command::Help) => show_help(),
        None if help => show_help(),
        None => run_maybe_path(path, args_strs.is_empty()),
    }
}

fn run_maybe_path(path: Option<&Path>, empty: bool) {
    let path = if let Some(path) = path {
        path
    } else {
        let path = Path::new("main.fel");
        if !path.exists() && empty {
            show_help();
            exit(0);
        }
        path
    };
    let text = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))
        .unwrap_or_else(fail);
    run(InputSrc::File(path.into()), &text);
}

fn run(src: InputSrc, text: &str) {
    let mut rt = Ufel::new();
    let res = rt.run(src, text);
    for val in rt.take_stack() {
        println!("{val}");
    }
    if let Err(e) = res {
        eprintln!("{e}");
    }
}

fn show_help() {
    println!(
        "{} - {}iua {}orm {}xperimentation {}anguage",
        "Ufel".bold(),
        "U".bold(),
        "f".bold(),
        "e".bold(),
        "l".bold()
    );
    println!();
    println!("Usage:");
    println!("  ufel [file]");
    println!("  Defaults to `main.fel` if no file is specified");
    println!();
    println!("Options:");
    println!("  -h | --help  Show this help message");
}
