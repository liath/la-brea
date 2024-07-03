extern crate base64;

mod cli_args;
mod decoder;

use decoder::Decoder;
use gumdrop::Options;
use polyfid::PolymorphicKey;

fn parse_shape(s: String) -> Vec<u8> {
    let mut res = Vec::new();
    for s in s.split('x') {
        res.push(s.parse()
                .expect("Unexpected value in shape string, shapes must be decimal integers separated by `x`. Example: `3x3x3` for trifid."),
        );
    }
    res
}

fn main() {
    let options = cli_args::DecoderOptions::parse_args_default_or_exit();
    println!("{:?}", options);

    // build key
    let key = PolymorphicKey::new(
        options.password.unwrap_or(String::from("")),
        options.alphabet.expect("alphabet must be set!"),
        parse_shape(options.shape.expect("shape must be set!")),
    );
    println!("{:?}", key);

    let input_s = options.input.expect("no input file given");
    let output_s = options.output.unwrap_or(String::from(""));
    let extract_name_s = options.extract_name.unwrap_or(String::from(""));
    let group_size = options.group_size.unwrap_or(0);
    let mut decoder = Decoder::new(input_s, output_s, key, group_size, extract_name_s);
    println!("{:?}", decoder);

    decoder.decode();
}
