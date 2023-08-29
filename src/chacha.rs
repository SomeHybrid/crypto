// A Rust implementation of XChaCha20-Poly1305
// This implementation defaults to 20 rounds
use crate::poly1305::Poly1305;
use crate::util::randbytes;
use pyo3::exceptions::PyAssertionError;
use pyo3::prelude::*;
use std::borrow::Cow;

const ROUNDS: usize = 20;

fn from_le_bytes(x: &[u8]) -> u32 {
    u32::from_le_bytes([x[0], x[1], x[2], x[3]])
}

fn quarter_round(a: usize, b: usize, c: usize, d: usize, block: &mut [u32; 16]) {
    block[a] = block[a].wrapping_add(block[b]);
    block[d] ^= block[a];
    block[d] = block[d].rotate_left(16);

    block[c] = block[c].wrapping_add(block[d]);
    block[b] ^= block[c];
    block[b] = block[b].rotate_left(12);

    block[a] = block[a].wrapping_add(block[b]);
    block[d] ^= block[a];
    block[d] = block[d].rotate_left(8);

    block[c] = block[c].wrapping_add(block[d]);
    block[b] ^= block[c];
    block[b] = block[b].rotate_left(7);
}

fn double_round(mut block: [u32; 16]) -> [u32; 16] {
    quarter_round(0, 4, 8, 12, &mut block);
    quarter_round(1, 5, 9, 13, &mut block);
    quarter_round(2, 6, 10, 14, &mut block);
    quarter_round(3, 7, 11, 15, &mut block);

    quarter_round(0, 5, 10, 15, &mut block);
    quarter_round(1, 6, 11, 12, &mut block);
    quarter_round(2, 7, 8, 13, &mut block);
    quarter_round(3, 4, 9, 14, &mut block);

    block
}

pub struct ChaCha20 {
    key: Vec<u8>,
}

// An implementation of IETF ChaCha
impl ChaCha20 {
    pub fn new(key: Vec<u8>) -> ChaCha20 {
        ChaCha20 { key }
    }

    fn keystream(&self, nonce: &[u8], counter: u32) -> Vec<u8> {
        let mut state = [
            0x61707865,
            0x3320646e,
            0x79622d32,
            0x6b206574,
            from_le_bytes(&self.key[0..4]),
            from_le_bytes(&self.key[4..8]),
            from_le_bytes(&self.key[8..12]),
            from_le_bytes(&self.key[12..16]),
            from_le_bytes(&self.key[16..20]),
            from_le_bytes(&self.key[20..24]),
            from_le_bytes(&self.key[24..28]),
            from_le_bytes(&self.key[28..]),
            counter,
            from_le_bytes(&nonce[0..4]),
            from_le_bytes(&nonce[4..8]),
            from_le_bytes(&nonce[8..12]),
        ];

        let mut working_state = state.clone();
        for _ in 0..(ROUNDS / 2) {
            working_state = double_round(working_state);
        }

        for (chunk, working_chunk) in state.iter_mut().zip(working_state) {
            *chunk = chunk.wrapping_add(working_chunk);
        }

        let mut result: Vec<u8> = Vec::new();

        for chunk in state {
            result.extend_from_slice(&chunk.to_le_bytes());
        }

        result
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8], counter: u32) -> Vec<u8> {
        let mut ciphertext: Vec<u8> = Vec::new();

        for (index, block) in plaintext.chunks(64).enumerate() {
            let keystream = self.keystream(nonce, counter + index as u32);

            for (key, chunk) in block.iter().zip(keystream) {
                ciphertext.push(chunk ^ key);
            }
        }

        ciphertext
    }
}

// ChaCha20-Poly1305 implementation
#[pyclass]
pub struct ChaCha20Poly1305 {
    key: Vec<u8>,
}

#[pymethods]
impl ChaCha20Poly1305 {
    #[new]
    pub fn new(key: Vec<u8>) -> ChaCha20Poly1305 {
        ChaCha20Poly1305 { key }
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8], aead: &[u8], counter: u32) -> Vec<u8> {
        let chacha = ChaCha20::new(self.key.clone());

        let otk = &chacha.keystream(nonce, 0);
        let poly1305_key = otk[..32].to_vec();

        let mut poly1305 = Poly1305::new(poly1305_key);
        let ciphertext = chacha.encrypt(plaintext, nonce, counter);

        poly1305.update(&aead, true);
        poly1305.update(&ciphertext, true);

        let aead_len = aead.len() as u64;
        let ciphertext_len = ciphertext.len() as u64;

        poly1305.update(&aead_len.to_le_bytes(), false);
        poly1305.update(&ciphertext_len.to_le_bytes(), false);

        [ciphertext, poly1305.tag()].concat().into()
    }

    pub fn decrypt(
        &self,
        text: &[u8],
        nonce: &[u8],
        aead: &[u8],
        counter: u32,
    ) -> PyResult<Vec<u8>> {
        if text.len() < 17 {
            return Err(PyAssertionError::new_err("Invalid ciphertext"));
        }

        let ciphertext = &text[..text.len() - 16];
        let tag = &text[text.len() - 16..];
        let chacha = ChaCha20::new(self.key.clone());

        let otk = &chacha.keystream(nonce, 0);
        let poly1305_key = otk[..32].to_vec();

        let mut poly1305 = Poly1305::new(poly1305_key);
        let plaintext = chacha.encrypt(ciphertext, nonce, counter);

        poly1305.update(&aead, true);
        poly1305.update(&ciphertext, true);

        let aead_len = aead.len() as u64;
        let ciphertext_len = ciphertext.len() as u64;

        poly1305.update(&aead_len.to_le_bytes(), false);
        poly1305.update(&ciphertext_len.to_le_bytes(), false);

        if poly1305.verify(tag) {
            return Ok(plaintext.to_vec());
        }

        Err(PyAssertionError::new_err("Invalid MAC"))
    }
}

pub fn hchacha20(key: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut state = [
        0x61707865,
        0x3320646e,
        0x79622d32,
        0x6b206574,
        from_le_bytes(&key[0..4]),
        from_le_bytes(&key[4..8]),
        from_le_bytes(&key[8..12]),
        from_le_bytes(&key[12..16]),
        from_le_bytes(&key[16..20]),
        from_le_bytes(&key[20..24]),
        from_le_bytes(&key[24..28]),
        from_le_bytes(&key[28..]),
        from_le_bytes(&nonce[0..4]),
        from_le_bytes(&nonce[4..8]),
        from_le_bytes(&nonce[8..12]),
        from_le_bytes(&nonce[12..]),
    ];

    for _ in 0..(ROUNDS / 2) {
        state = double_round(state);
    }

    let mut result: Vec<u8> = Vec::new();

    for chunk in state[0..4].iter().chain(state[12..16].iter()) {
        result.extend_from_slice(&chunk.to_le_bytes());
    }

    result
}

#[pyclass]
pub struct XChaCha20Poly1305 {
    key: Vec<u8>,
}

#[pymethods]
impl XChaCha20Poly1305 {
    #[new]
    pub fn new(key: Vec<u8>) -> XChaCha20Poly1305 {
        XChaCha20Poly1305 { key }
    }

    fn key(&self, nonce: &[u8]) -> (Vec<u8>, [u8; 12]) {
        let mut chacha_nonce = [0u8; 12];
        chacha_nonce[4..].copy_from_slice(&nonce[16..24]);

        let subkey = hchacha20(&self.key, &nonce[..16]);

        (subkey, chacha_nonce)
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8], aead: &[u8], counter: u32) -> Cow<[u8]> {
        let (subkey, chacha_nonce) = self.key(nonce);

        let chacha = ChaCha20Poly1305::new(subkey);

        chacha
            .encrypt(plaintext, &chacha_nonce, aead, counter)
            .into()
    }

    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &[u8],
        aead: &[u8],
        counter: u32,
    ) -> PyResult<Cow<[u8]>> {
        let (subkey, chacha_nonce) = self.key(nonce);

        let chacha = ChaCha20Poly1305::new(subkey);

        let output = chacha.decrypt(ciphertext, &chacha_nonce, aead, counter);

        match output {
            Ok(output) => Ok(output.into()),
            Err(e) => Err(e),
        }
    }
}

#[pyfunction]
fn keygen() -> Vec<u8> {
    randbytes::<32>().to_vec()
}

#[pymodule]
pub fn chacha(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(keygen, m)?)?;
    m.add_class::<ChaCha20Poly1305>()?;
    m.add_class::<XChaCha20Poly1305>()?;
    Ok(())
}

