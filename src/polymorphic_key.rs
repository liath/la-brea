#[derive(Debug)]
pub struct PolymorphicKey {
    pub key: String,
    pub shape: Vec<u8>,
}

impl PolymorphicKey {
    pub fn new(password: String, alphabet: String, shape: Vec<u8>) -> PolymorphicKey {
        let key = Self::password_to_key(password, alphabet);

        let key_len = key.len() as u8;
        let shape_size = shape.iter().product();
        if key_len != shape_size {
            panic!("Keyspace defined by shape is different than given password+alphabet ({} vs {}), check your parameters!", key_len, shape_size)
        }

        return PolymorphicKey { key, shape };
    }

    pub fn get_coords_for_symbol(&self, symbol: char) -> Vec<u8> {
        let mut index = 0;
        if symbol as u8 != 0 {
            index = self
                .key
                .find(symbol)
                .expect("Failed to find input symbol in key");
        }

        return Self::index_to_coords(&self.shape[..], index as u8, Vec::new());
    }

    pub fn get_symbol_for_coords(&self, coords: Vec<u8>) -> char {
        let index = Self::coords_to_index(&self.shape[..], 0, &coords[..]);

        return self
            .key
            .chars()
            .nth(index as usize)
            .expect("Failed to map coords to symbol");
    }

    pub fn dimensionality(&self) -> usize {
        return self.shape.len();
    }

    pub fn is_symbol_encodable(&self, symbol: char) -> bool {
        return self.key.contains(symbol);
    }

    fn password_to_key(password: String, alphabet: String) -> String {
        let mut state = String::from(password.trim());

        for c in alphabet.chars() {
            if state.contains(c) {
                continue;
            }

            state.push(c);
        }

        return state;
    }

    fn index_to_coords(shape: &[u8], index: u8, mut coords: Vec<u8>) -> Vec<u8> {
        // example iters
        // [4, 4, 4], 17, [] -> [4, 4], 1, [1] -> [4], 1, [1, 0] -> [], 0, [1, 0, 1]
        // [4, 4, 4], 53, [] -> [4, 4], 5, [3] -> [4], 1, [3, 1] -> [], 0, [3, 1, 1]
        // or in other words, (4*4*3)+(4*1)+1

        if shape.len() == 0 {
            return coords;
        }
        let chunk: u8 = shape[1..].iter().product();
        coords.push(index / chunk);
        return Self::index_to_coords(&shape[1..], index % chunk, coords);
    }

    fn coords_to_index(shape: &[u8], index: u8, coords: &[u8]) -> u8 {
        // [4, 4, 4], 0, [1, 0, 1] -> [4, 4], 16, [0, 1] -> [4], 16, [1] -> [], 17, []
        // [5, 5], 0, [0, 1] -> [5], 0, [1] -> [], 1, []

        if shape.len() == 0 {
            return index;
        }

        let chunk: u8 = shape[1..].iter().product();
        return Self::coords_to_index(&shape[1..], index + (chunk * coords[0]), &coords[1..]);
    }
}

#[cfg(test)]
mod tests;
