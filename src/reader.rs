use super::*;
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct DecodingReader<T> {
    // underlying encoded content, must be able to read/seek
    source: T,
    // the key maps source chars to plaintext chars
    pk: PolymorphicKey,
    // how many chars are encoded together
    group_size: u64,
    pub len: u64,
}

impl<T> DecodingReader<T>
where
    T: Read + Seek,
{
    pub fn new(mut source: T, pk: PolymorphicKey, group_size: u64) -> DecodingReader<T> {
        let len = source
            .seek(SeekFrom::End(0))
            .expect("Couldn't get length of source");
        source
            .seek(SeekFrom::Start(0))
            .expect("Couldn't return source to its starting position");

        let mut gs = group_size;
        if group_size <= 0 {
            gs = len;
        }

        let dr = DecodingReader {
            source,
            pk,
            group_size: gs,
            len,
        };

        return dr;
    }

    fn read_one_inner(&mut self, at: u64) -> u8 {
        self.source
            .seek(SeekFrom::Start(at))
            .expect("Failed to seek input to beginning");

        let mut buf: [u8; 1] = [0];
        // not sure we need to do this
        loop {
            match self.source.read(&mut buf) {
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

    pub fn read_one(&mut self) -> u8 {
        let dimensionality = self.pk.dimensionality() as u64;
        let index = self
            .source
            .seek(SeekFrom::Current(0))
            .expect("Unable to get current cursor from source, perhaps the file is closed?");
        let mut coords = Vec::new();

        for dimension in 0..dimensionality {
            let at = index + (dimension * self.group_size);
            let col = at / dimensionality;
            let row = at % dimensionality;

            let c = self.read_one_inner(col) as char;
            if !self.pk.is_symbol_encodable(c) {
                println!("got unencodable symbol? {} [{:#04x}]", c, c as u8);
                continue;
            }

            println!("index: {}, col: {}, row: {}", index, col, row);
            let bit = self.pk.get_coords_for_symbol(c)[row as usize];
            println!(" -> {}", bit);
            coords.push(bit);
        }

        // return source reader to where we found it
        self.source
            .seek(SeekFrom::Start(index + 1))
            .expect("failed to reset source cursor");

        return self.pk.get_symbol_for_coords(coords) as u8;
    }
}

impl<T> Read for DecodingReader<T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        println!("reading {} bytes", self.len);
        for i in 0..self.len as usize {
            buf[i] = self.read_one();
            println!("buf: {:x?}", buf);
        }
        Ok(buf.len())
    }
}

impl<T> Seek for DecodingReader<T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        return self.source.seek(pos);
    }
}

#[cfg(test)]
mod tests;
