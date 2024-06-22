use super::*;
use std::io::Cursor;

#[test]
fn basic() {
    let pk = PolymorphicKey::new(
        String::from("MEOW"),
        String::from("ABCDEFGHIKLMNOPQRSTUVWXYZ"),
        Vec::from([5, 5]),
    );

    let source = Cursor::new(b"MMEL");

    let mut out = [0, 0, 0, 0];
    DecodingReader::new(source, pk, 4)
        .read(&mut out)
        .expect("failed to read?");

    assert_eq!(std::str::from_utf8(&out).expect(""), "MEOW");
}
