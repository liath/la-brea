use nohash_hasher::BuildNoHashHasher;
use std::cmp::min;
use std::collections::HashMap;
use std::io::{self, Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Source<T> {
    name: String,
    size: u64,
    source: T,
}

impl<T> Read for Source<T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.source.read(buf)
    }
}

impl<T> Seek for Source<T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.source.seek(pos)
    }
}

#[derive(Debug)]
pub struct Reader<T> {
    // file ids in the order they should be returned
    index: Vec<(u64, u64)>,
    // length not accounting for blocking_factor
    len: u64,
    len_total: u64,
    // where are we within the index if this tar actually existed already
    pos: u64,
    // tars are grouped in `blocking_factor`*512 long "records"
    record_size: f64,
    // map file names to an id, this cuts down on memory usage when a tar has a
    // bunch of references to the same file (which is specifically why I wrote
    // this library lol)
    rolodex: HashMap<String, u64>,
    rolodex_next: u64,
    // map of file IDs to their underlying ReadSeeks
    sources: HashMap<u64, Source<T>, nohash_hasher::BuildNoHashHasher<u64>>,
}

impl<T> Default for Reader<T>
where
    T: Read + Seek,
{
    fn default() -> Self {
        Reader::new()
    }
}

impl<T> Reader<T>
where
    T: Read + Seek,
{
    pub fn new() -> Reader<T> {
        // default per https://www.gnu.org/software/tar/manual/html_node/Blocking-Factor.html
        // TODO: maybe we expose this to callers?
        let blocking_factor = 20;
        Reader {
            index: Vec::new(),
            len: 0,
            len_total: 0,
            pos: 0,
            record_size: (blocking_factor as f64) * 512.0,
            rolodex: HashMap::new(),
            // start at one so we know index_to_source_offset returning a 0 is
            // indicating we're in the trailing zeros (`end` does this too but
            // I wanna be extra clear and this costs us nothing)
            rolodex_next: 1,
            sources: HashMap::with_hasher(BuildNoHashHasher::default()),
        }
    }

    pub fn append_entry(&mut self, name: String, mut source: T) {
        // create a rolodex entry if needed
        let id = if self.rolodex.contains_key(&name) {
            *self
                .rolodex
                .get(&name)
                .expect("rolodex has entry but not a value?")
        } else {
            let i = self.rolodex_next;
            self.rolodex_next += 1;
            self.rolodex.insert(name.clone(), i);
            i
        };

        // create a sources entry if needed
        self.sources.entry(id).or_insert({
            let size = source
                .seek(SeekFrom::End(0))
                .expect("Couldn't get length of source");
            source
                .seek(SeekFrom::Start(0))
                .expect("Couldn't rewind source");

            Source {
                name: name.clone(),
                size,
                source,
            }
        });

        let size = self.sources.get(&id).expect("How did we get here?").size;
        // append another copy of this source to the output
        self.index.push((id, self.len));
        self.len += 512 + ((size as f64 / 512.0).ceil() as u64 * 512);
        // blocks are grouped in record
        self.len_total = ((self.len as f64 / self.record_size).ceil() * self.record_size) as u64;

        println!("la_brea: now {} bytes long", self.len);
    }

    fn header(&mut self, id: u64) -> [u8; 512] {
        // TODO: Stop hardcoding almost every header field
        // chksum is initialized to spaces per:
        // https://www.gnu.org/software/tar/manual/html_node/Standard.html
        let mut buf = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, // name
            0x30, 0x30, 0x30, 0x30, 0x36, 0x36, 0x34, 0x00, // mode
            0x30, 0x30, 0x30, 0x31, 0x37, 0x35, 0x30, 0x00, // uid
            0x30, 0x30, 0x30, 0x31, 0x37, 0x35, 0x30, 0x00, // gid
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x00, // size
            0x31, 0x34, 0x36, 0x32, 0x33, 0x32, 0x36, 0x30, 0x35, 0x33, 0x31, 0x00, // mtime
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, // chksum
            0x30, // typeflag
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, // linkname
            0x75, 0x73, 0x74, 0x61, 0x72, 0x20, // magic
            0x20, 0x00, // version
            0x6c, 0x69, 0x61, 0x74, 0x68, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, // uname
            0x6c, 0x69, 0x61, 0x74, 0x68, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, // gname
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // devmajor
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // devminor
            // prefix ->
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let f = self.sources.get(&id).expect("failed to retrieve source");
        for (i, c) in f.name.chars().enumerate() {
            if i == 100 {
                break;
            }

            buf[i] = c as u8;
        }

        let size = format!("{:0>11o}", f.size);
        // println!("size: {}", size);
        for (i, c) in size.chars().enumerate() {
            buf[124 + i] = c as u8;
        }

        let chksum = format!("{:0>6o}", buf.iter().fold(0u64, |sum, i| sum + (*i as u64)));
        // println!("chksum: {}", chksum);
        for (i, c) in chksum.chars().enumerate() {
            buf[148 + i] = c as u8;
        }
        buf[154] = 0;

        buf
    }

    fn index_to_source_offset(&mut self, at: u64) -> (u64, u64, u64, bool) {
        // println!("i2so -> {}", at);

        let mut lid = 0;
        let mut loff = 0;
        //      love?

        for (id, offset) in self.index.iter() {
            // println!("     -> id: {}, offset: {}", id, offset);
            if *offset > at {
                return (lid, at - loff, *offset, false);
            }
            lid = *id;
            loff = *offset;
        }
        if at < self.len {
            return (lid, at - loff, self.len_total, false);
        }
        (0, at - self.len, self.len_total, true)
    }
}

impl<T> Read for Reader<T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // nothing to output yet or EOF
        if self.len == 0 || self.pos == self.len_total {
            return Ok(0);
        }

        let want = buf.len() as u64;
        let (id, offset, boundary, end) = self.index_to_source_offset(self.pos);
        /* println!(
            "lb[{}] pos: {}, len: {}, want: {}, id: {}, offset: {}, end: {}",
            id, self.pos, self.len, want, id, offset, end
        ); */

        let mut wrote = 0;
        if offset < 512 && !end {
            let mut header = Cursor::new(self.header(id));
            header
                .seek(SeekFrom::Start(offset))
                .expect("Somehow failed to seek header, but whhhhyyyy?");
            let res = header
                .read(buf)
                .expect("Somehow failed to read header, how could that even happen???");

            wrote += res as u64;
            // println!("lb[{}] wrote {} bytes of header", id, res);
        }

        if wrote < want && !end {
            let source = self.sources.get_mut(&id).expect("Unable to fetch source");
            // position where we start emitting the padding between blocks
            let pad_start = 512 + source.size;

            // is there more file than we've written?
            if pad_start > offset + wrote {
                source
                    .seek(SeekFrom::Start(offset + wrote - 512))
                    .expect("Unable to seek source");
                let res = source
                    .read(&mut buf[wrote as usize..])
                    .expect("Failed to read source");

                wrote += res as u64;
                // println!("lb[{}->{}] wrote {} bytes of content", id, source.name, res);
            }

            // handle padding between files
            if wrote < want && pad_start <= offset + wrote && offset + wrote < boundary {
                let pad_len = min(want, boundary - offset) - wrote;
                let padding = vec![0; pad_len as usize];

                buf[wrote as usize..(wrote + min(want, pad_len)) as usize]
                    .copy_from_slice(&padding);

                /* println!(
                    "lb[{}->{}] wrote {} bytes of padding",
                    id, source.name, pad_len
                ); */
                wrote += pad_len;
            }
        }

        if end {
            // TODO: should we lock at this point? It would prevent our output
            //       requiring --ignore-zeros for files where an append happens
            //       after reading the trailer.
            let mut trailer = Cursor::new([0; 8192]);

            // how much trailer is there left to get to the record-size edge
            // in at most blocks of 8192, as that seems to be the chunk size
            // io::Read uses
            let size = min(self.len_total - self.pos, 8192);

            // adjust to remaining read size
            trailer
                .seek(SeekFrom::Start(8192 - size))
                .expect("The trailer seek position is weird");

            let res = trailer
                .read(&mut buf[wrote as usize..])
                .expect("Failed to write trailer");
            wrote += res as u64;
            /* println!(
                "lb[{}] wrote {}/{} bytes at {} to pad out trailer",
                id,
                res,
                self.len_total,
                self.pos
            ); */
        }

        self.pos += wrote;

        // if we aren't at the end of the archive and the output buffer isn't
        // full call ourselves again to start reading the next file
        if self.pos != self.len_total && wrote < want {
            let res = self.read(&mut buf[wrote as usize..]).unwrap();
            wrote += res as u64;
        }

        Ok(wrote as usize)
    }
}

impl<T> Seek for Reader<T>
where
    T: Read + Seek,
{
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                // println!("lb seeking to: {}", n);
                return Ok(n);
            }
            SeekFrom::End(n) => (self.len_total, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        match base_pos.checked_add_signed(offset) {
            Some(n) => {
                self.pos = n;
                // println!("lb seeking to: {}", n);
                Ok(self.pos)
            }
            None => Err(Error::new(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }
}

#[cfg(test)]
mod tests;
