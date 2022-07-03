#![feature(assert_matches)]
#![feature(test)]
#![feature(derive_default_enum)]

extern crate test;

use std::{env, io, fs, process};

use wasmtools::lexer::lex;

macro_rules! error {
    ( $( $x:expr ),+ ) => {
        {
            eprintln!( $( $x, )+ );
            process::exit(1);
        }
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let opts: StringMode = match &args[1..] {
        [] => error!("Please provide a file name or some text to compile"),
        [string] => StringMode::File(&string[..]),
        [string, flag] => {
            match &flag[..] {
                "-f" => StringMode::File(&string[..]),
                "-s" => StringMode::String(&string[..]),
                _ => error!("unknown option: {}", flag)
            }
        },
        [_, _, ..] => error!("Too many arguments provided!"),
    };
    let contents = match opts {
        StringMode::File(filename) => match fs::read_to_string(filename) {
            Ok(s) => s,
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => error!("couldn't read file {}: not found", filename),
                io::ErrorKind::PermissionDenied => error!("couldn't read file {}: permission denied", filename),
                _ => error!("couldn't read file {}: unknown error", filename)
            }
        },
        StringMode::String(string) => string.to_string()
    };
    let tokens = lex(&contents);
    if let Some(tokens) = tokens {
        println!("{:?}", tokens);
    }
}

enum StringMode<'a> {
    String(&'a str),
    File(&'a str)
}