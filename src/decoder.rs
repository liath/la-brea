extern crate base64;

use super::*;
use base64::{engine::general_purpose, Engine as _};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

#[derive(Debug)]
pub struct Decoder {
    input: String,
    output: String,
    extract_name: String,
    pk: PolymorphicKey,
    group_size: u64,
}

impl Decoder {
    pub fn new(
        input: String,
        output: String,
        pk: PolymorphicKey,
        group_size: u64,
        extract_name: String,
    ) -> Decoder {
        let decoder = Decoder {
            input: input.clone(),
            output: output.clone(),
            extract_name: extract_name.clone(),
            pk,
            group_size,
        };

        fs::metadata(input).expect("Failed to stat input file");

        return decoder;
    }

    pub fn decode(&self) {
        let mut input = File::open(self.input.clone()).expect("Failed to open input for read");

        let mut output = File::open("/dev/null").expect("Couldn't open /dev/null???");
        let list_mode = !(self.output.len() > 0);
        let mut extract_mode = false;
        if list_mode {
            println!("running in list mode");
        } else {
            output = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(self.output.clone())
                .expect("Failed to open output for read/write");

            if self.extract_name.len() > 0 {
                extract_mode = true;
                println!("running in extract mode");
            } else {
                println!("running in decode mode");
            }
        }

        // tar record size
        let tar_record_size = 512 * 20;

        let dimensionality = self.pk.dimensionality() as u64;
        let meta = input.metadata().expect("Failed to stat input");
        let length = meta.len();

        let mut header_buf = Vec::new();

        let mut index = 0;
        let mut extracting = false;
        let mut extract_size = 0;
        let mut header_start = 0;
        let mut skip = 0;
        while index < length {
            let mut coords = Vec::new();
            for dimension in 0..dimensionality {
                let at = index + (dimension * length);
                let col = at / dimensionality;
                let row = at % dimensionality;

                let c = Self::read_at(&mut input, col) as char;
                if !self.pk.is_symbol_encodable(c) {
                    println!("got unencodable symbol? {}", c);
                    continue;
                }

                let bit = self.pk.get_coords_for_symbol(c)[row as usize];
                // println!("index: {}, col: {}, row: {} -> {}", index, col, row, bit);
                coords.push(bit);
            }

            let sym = self.pk.get_symbol_for_coords(coords);
            //println!("out: {}", sym);

            // TODO: cccccyyyyycccccllloooooommmmaaaatttiiiiiiccccccccc C O M P L E X I T Y (complicity?)
            // TODO: we could just write chunks of input into the output as we which would spare
            //       holding it all in memory
            if list_mode || extract_mode {
                header_buf.push(sym as u8);

                // only check at base64 chunk size
                if header_buf.len() % 4 == 0 {
                    // println!("header b64: {:x?}", header_buf);
                    match general_purpose::STANDARD_NO_PAD.decode(header_buf.clone()) {
                        Ok(deb64) => {
                            let trimmed = (&deb64[skip..]).to_vec();
                            if extracting {
                                if trimmed.len() >= extract_size {
                                    if let Err(_) = output.write_all(&trimmed[0..extract_size]) {
                                        println!("Failed to write extracted file")
                                    }
                                    println!("extracted {} bytes to output", trimmed.len());

                                    // TODO: instead of extracting nulls we could fill the end of the
                                    // record with zeros for basically free
                                    // if let Err(_) = output.write_all(&[0; 0x2200]) {
                                    //     // append tar required waste
                                    //     println!("Failed to write tar waste to extracted file")
                                    // }
                                    break;
                                }
                            } else if trimmed.len() >= 136 {
                                println!(
                                    "header start: {}, end: {}, trimmed: {:x?}",
                                    header_start, index, trimmed
                                );

                                let (filename, size) = Self::parse_header(trimmed.clone());
                                println!("name: {}, size: {}, index: {}", filename, size, index);

                                let tar_length =
                                    (((512 + size) as f32 / tar_record_size as f32).ceil() as u64)
                                        * tar_record_size;

                                if extract_mode && filename == self.extract_name {
                                    extracting = true;
                                    extract_size = tar_length as usize;
                                    println!(
                                        "Found target file, extracting {} byte tar now",
                                        extract_size
                                    );
                                } else {
                                    // skip to next file
                                    let new_index =
                                        header_start + Self::base64_ratio(tar_length) - 1; // counts from zero

                                    // the chunk sizing of base64 and the record sizes of tar
                                    // guarantee that we'll always need to skip the first byte of
                                    // subsequent records. I think, I left this as a whole variable
                                    // instead of just doing a fixed offset in case I'm wrong
                                    skip = 1;

                                    println!("index jumped {} -> {}", index, new_index);

                                    // tar adds 0x2200 zero bytes for mysterious reasons
                                    // the -4 pulls us back one chunk of base64, which encodes 3
                                    // input bytes into 4 output ascii chars. This may cause the
                                    // header_buf to accumulate some null bytes from the waste at
                                    // the end of the previous file but we can handle that
                                    index = new_index;

                                    header_buf = Vec::new();
                                    header_start = index;
                                }
                            }
                        }
                        Err(_error) => {
                            // continue pulling bytes until we resolve the parse error
                            // println!("error: {}", _error);
                        }
                    }
                }
            } else {
                Self::write_at(&mut output, index, sym as u8);
            }

            index += 1;
        }
    }

    fn read_at(f: &mut File, at: u64) -> u8 {
        f.seek(SeekFrom::Start(at))
            .expect("Failed to seek input to beginning");

        let mut buf: [u8; 1] = [0];
        // not sure we need to do this
        loop {
            match f.read(&mut buf) {
                Ok(_) => {
                    break;
                }
                Err(error) => {
                    panic!("Got error reading input byte: {:?}", error)
                }
            }
        }

        return buf[0];
    }

    fn write_at(f: &mut File, at: u64, val: u8) {
        f.seek(SeekFrom::Start(at))
            .expect("Failed to seek input to beginning");

        // println!("write {} at {}", val, at);
        let buf: [u8; 1] = [val];
        loop {
            match f.write(&buf) {
                Ok(bytes) => {
                    if bytes > 0 {
                        break;
                    }
                }
                Err(error) => {
                    panic!("Got error pushing output byte: {:?}", error)
                }
            }
        }
    }

    fn parse_header(bytes: Vec<u8>) -> (String, u64) {
        let mut tmp = Vec::new();
        for i in 0..100 {
            if bytes[i] == 0 {
                break;
            }
            tmp.push(bytes[i] as char);
        }

        let filename: String = tmp.into_iter().collect();

        tmp = Vec::new();
        for i in 124..136 {
            if bytes[i] == 0 {
                break;
            }
            tmp.push(bytes[i] as char);
        }

        let size: String = tmp.into_iter().collect();

        println!("header | name: {}, size: {}", filename, size);
        return (
            filename,
            u64::from_str_radix(&size, 8).expect("Failed to parse tar entry size"),
        );
    }

    fn base64_ratio(i: u64) -> u64 {
        return (i / 3) as u64 * 4;
    }
}

#[cfg(test)]
mod tests;
