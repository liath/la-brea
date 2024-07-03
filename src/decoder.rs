extern crate base64;

use super::*;
use base64::{engine::general_purpose, Engine as _};
use polyfid::Reader;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

#[derive(Debug)]
pub struct Decoder {
    source: Reader<File>,
    output: String,
    extract_name: String,
}

impl Decoder {
    pub fn new(
        input: String,
        output: String,
        pk: PolymorphicKey,
        group_size: u64,
        extract_name: String,
    ) -> Decoder {
        let source_f = File::open(input.clone()).expect("Failed to open input for read");
        let source = Reader::new(source_f, pk, group_size, String::from("decode"));

        Decoder {
            source,
            output: output.clone(),
            extract_name: extract_name.clone(),
        }
    }

    pub fn decode(&mut self) {
        let mut output = File::open("/dev/null").expect("Couldn't open /dev/null???");
        let list_mode = self.output.is_empty();
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

            if self.extract_name.is_empty() {
                println!("running in decode mode");
            } else {
                extract_mode = true;
                println!("running in extract mode");
            }
        }

        let tar_record_size = 512 * 20;
        let tar_header_size = Self::base64_ratio(136) as usize;

        let length = self.source.len;

        let mut buf = Vec::new();

        let mut index = 0;
        let mut extracting = false;
        let mut extract_size = 0;
        let mut header_start = 0;
        let mut skip = 0;
        while index < length {
            let sym = self.source.read_one();
            // println!("out: {}", sym as char);

            index += 1;

            if !list_mode && !extract_mode {
                Self::write_at(&mut output, index - 1, sym);
                continue;
            }

            buf.push(sym);

            // only check at base64 chunk size
            if buf.len() % 4 != 0 {
                continue;
            }

            if (!extracting && buf.len() <= tar_header_size)
                || (extracting && buf.len() < extract_size as usize)
            {
                continue;
            }

            let trimmed = match general_purpose::STANDARD_NO_PAD.decode(buf.clone()) {
                Ok(deb64) => deb64[skip..].to_vec(),
                Err(_error) => {
                    // continue pulling bytes until we resolve the parse error
                    println!("error: {}", _error);
                    continue;
                }
            };

            if extracting {
                // TODO: experiment with other write chunks sizes
                if trimmed.len() < 1024 && trimmed.len() < extract_size as usize {
                    continue;
                }
                skip = 0;

                if output.write_all(&trimmed).is_err() {
                    println!("Failed to write extracted file")
                }
                extract_size -= trimmed.len() as i64;
                println!(
                    "extracted {} bytes to output, {} remaining",
                    trimmed.len(),
                    extract_size
                );
                buf = Vec::new();

                // TODO: instead of extracting nulls we could fill the end of the
                // record with zeros for basically free
                // if let Err(_) = output.write_all(&[0; 0x2200]) {
                //     // append tar required waste
                //     println!("Failed to write tar waste to extracted file")
                // }

                if extract_size == 0 {
                    break;
                }
            } else if trimmed.len() >= 136 {
                println!(
                    "header start: {}, end: {}, trimmed: {:x?}",
                    header_start, index, trimmed
                );

                let (filename, size) = Self::parse_header(trimmed.clone());
                println!("name: {}, size: {}, index: {}", filename, size, index);

                let tar_length = (((512 + size) as f32 / tar_record_size as f32).ceil() as u64)
                    * tar_record_size;

                if extract_mode && filename == self.extract_name {
                    extracting = true;
                    extract_size = tar_length as i64;
                    println!(
                        "Found target file, extracting {} byte tar now",
                        extract_size
                    );
                } else {
                    // skip to next file
                    let new_index = header_start + Self::base64_ratio(tar_length);

                    // the chunk sizing of base64 and the record sizes of tar
                    // guarantee that we'll always need to skip the first byte of
                    // subsequent records. I think, I left this as a whole variable
                    // instead of just doing a fixed offset in case I'm wrong
                    skip = 1;

                    println!("index jumped {} -> {}", index, new_index);

                    index = new_index;
                    self.source
                        .seek(SeekFrom::Start(index))
                        .expect("failed to jump to next tar index");

                    buf = Vec::new();
                    header_start = index;
                }
            }
        }
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
        for byte in bytes.iter().take(100) {
            if *byte == 0 {
                break;
            }
            tmp.push(*byte as char);
        }

        let filename: String = tmp.into_iter().collect();

        tmp = Vec::new();
        for byte in bytes.iter().take(136).skip(124) {
            if *byte == 0 {
                break;
            }
            tmp.push(*byte as char);
        }

        let size: String = tmp.into_iter().collect();

        println!("header | name: {}, size: {}", filename, size);
        (
            filename,
            u64::from_str_radix(&size, 8).expect("Failed to parse tar entry size"),
        )
    }

    fn base64_ratio(i: u64) -> u64 {
        (i / 3) * 4
    }
}

#[cfg(test)]
mod tests;
