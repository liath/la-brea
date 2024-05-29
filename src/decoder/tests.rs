use super::*;

#[test]
fn decode_bifid() {
    let pk = PolymorphicKey::new(
        String::from("MEOW"),
        String::from("ABCDEFGHIKLMNOPQRSTUVWXYZ"),
        Vec::from([5, 5]),
    );

    let output = String::from("./tmp/test-decoder-bifid-out.txt");
    let decoder = Decoder::new(
        String::from("./fixtures/test-decoder-bifid-in.txt"),
        output.clone(),
        pk,
        0,
        String::from(""),
    );

    decoder.decode();

    let res = fs::read_to_string(output).expect("failed to read output");
    assert_eq!(res, "MEOW");
}

#[test]
fn decode_trifid() {
    let pk = PolymorphicKey::new(
        String::from("MEOW"),
        String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ+"),
        Vec::from([3, 3, 3]),
    );

    let output = String::from("./tmp/test-decoder-trifid-out.txt");
    let decoder = Decoder::new(
        String::from("./fixtures/test-decoder-trifid-in.txt"),
        output.clone(),
        pk,
        0,
        String::from(""),
    );

    decoder.decode();

    let res = fs::read_to_string(output).expect("failed to read output");
    assert_eq!(res, "MEOW");
}

#[test]
fn decode_big() {
    let pk = PolymorphicKey::new(
        String::from(""),
        String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"),
        Vec::from([4, 4, 4]),
    );

    let output = String::from("./tmp/test-decoder-big-out.txt");
    let decoder = Decoder::new(
        String::from("./fixtures/test-decoder-big-in.txt"),
        output.clone(),
        pk,
        0,
        String::from(""),
    );

    decoder.decode();

    let expected = fs::read_to_string("./fixtures/expected-decoder-big.txt")
        .expect("failed to load expected output");
    let res = fs::read_to_string(output).expect("failed to read output");
    assert_eq!(res, expected);
}

#[test]
fn decode_and_extract_name() {
    let pk = PolymorphicKey::new(
        String::from(""),
        String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"),
        Vec::from([4, 4, 4]),
    );

    let output = String::from("./tmp/test-decoder-goots.tar");
    let decoder = Decoder::new(
        String::from("./fixtures/goots.la-brea"),
        output.clone(),
        pk,
        0,
        String::from("goots.jpg"),
    );

    decoder.decode();

    let expected = fs::read("./fixtures/goots.tar.actual").expect("failed to load expected output");
    let res = fs::read(output).expect("failed to read output");
    assert_eq!(res, expected);
}
