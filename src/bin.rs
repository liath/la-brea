extern crate base64;

mod reader;

use glob::glob;
use gumdrop::Options;
use reader::Reader;
use std::env::args;
use std::fs::{File, OpenOptions};
use std::io;

#[derive(Debug, Options)]
pub struct CLIOptions {
    #[options(help = "Print this message")]
    help: bool,

    #[options(free)]
    target: String,

    #[options(free)]
    inputs: Vec<String>,
}

fn main() {
    let args = args().collect::<Vec<_>>();
    let options = CLIOptions::parse_args_default(&args[1..]).expect("failed to parse CLI args");
    if options.help || options.target.is_empty() || options.inputs.is_empty() {
        eprintln!("Usage: la_brea [target] [input [input [...]]]");
        return;
    }

    let mut reader = Reader::new();

    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(options.target)
        .expect("Failed to open output for read/write");

    for input in options.inputs {
        let input_s = input.as_str();
        for entry in glob(input_s).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let path_s = path.display().to_string();
                    let f = File::open(path_s.as_str())
                        .unwrap_or_else(|_| panic!("Failed to open input: {}", path_s));
                    reader.append_entry(path_s.to_string(), f)
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }

    io::copy(&mut reader, &mut output).expect("Encoding failed?");
}
