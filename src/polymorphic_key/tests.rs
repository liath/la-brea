use super::*;

#[test]
fn basic() {
    let pk = PolymorphicKey::new(
        String::from("MEOW"),
        String::from("ABCDEFGHIKLMNOPQRSTUVWXYZ"),
        Vec::from([5, 5]),
    );

    assert_eq!(pk.get_coords_for_symbol('M'), [0, 0]);
    assert_eq!(pk.get_coords_for_symbol('W'), [0, 3]);
    assert_eq!(pk.get_coords_for_symbol('A'), [0, 4]);
    assert_eq!(pk.get_coords_for_symbol('Z'), [4, 4]);

    assert_eq!(pk.get_symbol_for_coords(Vec::from([0, 1])), 'E');
    assert_eq!(pk.get_symbol_for_coords(Vec::from([0, 2])), 'O');
    assert_eq!(pk.get_symbol_for_coords(Vec::from([1, 0])), 'B');
    assert_eq!(pk.get_symbol_for_coords(Vec::from([4, 3])), 'Y');
}

#[test]
#[should_panic]
fn thows_on_keyspace_shape_mismatch() {
    PolymorphicKey::new(
        String::from("abc"),
        String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        Vec::from([5, 5]),
    );
}
