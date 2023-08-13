use super::*;

#[test]
fn encode_bifid() {
    let pk = PolymorphicKey::new(
        String::from("MEOW"),
        String::from("ABCDEFGHIKLMNOPQRSTUVWXYZ"),
        Vec::from([5, 5]),
    );

    let output = String::from("./tmp/test-encoder-bifid-out.txt");
    let encoder = Encoder::new(
        String::from("./fixtures/encoder-basic-in.txt"),
        output.clone(),
        pk,
        0,
    );

    encoder.encode();

    let res = fs::read_to_string(output).expect("failed to read output");
    assert_eq!(res, "MMEL");
}
