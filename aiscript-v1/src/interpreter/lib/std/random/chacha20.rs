use super::Rng;

use chacha20::{ChaCha20Legacy, KeyIvInit, cipher::StreamCipher};

#[derive(Debug)]
pub struct ChaCha20Rng {
    cipher: ChaCha20Legacy,
}

impl ChaCha20Rng {
    pub fn new(seed: &[u8]) -> Self {
        let (key, nonce) = seed.split_at(32);
        let nonce = &nonce[..8];
        let cipher = ChaCha20Legacy::new_from_slices(key, nonce).unwrap();
        ChaCha20Rng { cipher }
    }
}

impl Rng for ChaCha20Rng {
    fn generate_int_by_bytes(&mut self, bytes: u8) -> u64 {
        let mut buf = [0_u8; 8];
        self.cipher.apply_keystream(&mut buf[..usize::from(bytes)]);
        u64::from_le_bytes(buf)
    }
}
