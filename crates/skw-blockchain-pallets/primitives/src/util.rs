use sp_std::vec::Vec;

pub fn compress_hex_key(s: &Vec<u8>) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| s[i] * 16 + s[i + 1])
        .collect()
}
