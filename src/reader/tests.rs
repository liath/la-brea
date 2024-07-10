use super::*;
use std::fs::File;
use std::io::Cursor;

#[test]
fn header() {
    let mut reader = Reader::new();
    reader.append_entry("le-garbage".to_string(), Cursor::new([0; 1024]));

    let mut expected: [u8; 512] = [0; 512];
    File::open("fixtures/test.tar")
        .expect("failed to open fixture")
        .read_exact(&mut expected)
        .expect("failed to read header from fixture");

    assert_eq!(reader.header(0), expected);
}
