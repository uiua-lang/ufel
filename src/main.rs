use std::{env, fmt::Display, fs, path::Path, process::exit};

use colored::Colorize;

use ufel::{lex, InputSrc};

fn fail<T>(e: impl Display) -> T {
    eprintln!("{e}");
    exit(1);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let args_strs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut args = args_strs.as_slice();
    let mut path: Option<&Path> = None;
    let mut help = false;
    loop {
        match args {
            [] => break,
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

    if help {
        show_help();
        return;
    }

    let path = if let Some(path) = path {
        path
    } else {
        let path = Path::new("main.fel");
        if !path.exists() && args_strs.is_empty() {
            show_help();
            return;
        }
        path
    };
    let text = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))
        .unwrap_or_else(fail);
    run(InputSrc::File(path.into()), &text);
}

fn run(src: InputSrc, text: &str) {
    let mut inputs = Vec::new();
    let tokens = lex(src, text, &mut inputs).unwrap();
    for token in tokens {
        println!("{token:?}");
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
