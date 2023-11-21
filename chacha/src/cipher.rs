// A Rust implementation of XChaCha-Poly1305
// This implementation defaults to 20 rounds
use crate::backends;
use crate::poly1305::Poly1305;
use pyo3::exceptions::PyAssertionError;
use pyo3::prelude::*;
use std::borrow::Cow;

#[pyclass]
pub struct ChaCha {
    backend: backends::Backend,
}

impl ChaCha {
    pub fn new(key: Vec<u8>, rounds: Option<usize>) -> PyResult<ChaCha> {
        let rounds = if rounds.is_some() {
            rounds.unwrap()
        } else {
            20
        };

        if key.len() != 32 {
            return Err(PyAssertionError::new_err("Key must be 32 bytes in length."));
        }

        if rounds < 1 {
            return Err(PyAssertionError::new_err("Rounds must be at least 1"));
        }

        Ok(ChaCha {
            backend: backends::Backend::new(key, rounds),
        })
    }

    pub fn keystream(&self, nonce: &[u8], counter: u32) -> [u8; 64] {
        self.backend.keystream(nonce, counter)
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8]) -> Vec<u8> {
        self.backend.encrypt(plaintext, nonce)
    }
}

// ChaCha-Poly1305 implementation
#[pyclass]
pub struct ChaChaPoly1305 {
    key: Vec<u8>,
    rounds: usize,
}

#[pymethods]
impl ChaChaPoly1305 {
    #[new]
    pub fn new(key: Vec<u8>, r: Option<usize>) -> PyResult<ChaChaPoly1305> {
        let rounds;

        if r.is_some() {
            rounds = r.unwrap();
        } else {
            rounds = 20;
        }

        if key.len() != 32 {
            return Err(PyAssertionError::new_err("Key must be 32 bytes in length."));
        }

        if rounds < 1 {
            return Err(PyAssertionError::new_err("Rounds must be at least 1"));
        }

        Ok(ChaChaPoly1305 { key, rounds })
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8], aad: &[u8]) -> PyResult<Vec<u8>> {
        let chacha = ChaCha::new(self.key.clone(), Some(self.rounds))?;

        let otk = &chacha.keystream(nonce, 0);
        let poly1305_key = &otk[..32];

        let mut poly1305 = Poly1305::new(poly1305_key);
        let ciphertext = chacha.encrypt(plaintext, nonce);

        poly1305.update(aad);
        poly1305.update(&ciphertext);
        let aad_len = aad.len() as u64;
        let ciphertext_len = ciphertext.len() as u64;
        let mut lens = Vec::new();

        lens.extend_from_slice(&aad_len.to_le_bytes());
        lens.extend_from_slice(&ciphertext_len.to_le_bytes());

        poly1305.update(&lens);

        Ok([ciphertext, poly1305.tag()].concat())
    }

    pub fn decrypt(&self, text: &[u8], nonce: &[u8], aad: &[u8]) -> PyResult<Vec<u8>> {
        if text.len() < 17 {
            return Err(PyAssertionError::new_err("Invalid ciphertext"));
        }

        let ciphertext = &text[..text.len() - 16];
        let tag = &text[text.len() - 16..];
        let chacha = ChaCha::new(self.key.clone(), Some(self.rounds))?;

        let otk = &chacha.keystream(nonce, 0);
        let poly1305_key = &otk[..32];

        let mut poly1305 = Poly1305::new(poly1305_key);
        let plaintext = chacha.encrypt(ciphertext, nonce);

        poly1305.update(&ciphertext);
        poly1305.update(&aad);

        let aad_len = aad.len() as u64;
        let ciphertext_len = ciphertext.len() as u64;
        let mut lens = Vec::new();

        lens.extend_from_slice(&aad_len.to_le_bytes());
        lens.extend_from_slice(&ciphertext_len.to_le_bytes());

        poly1305.update(&lens);

        if !poly1305.verify(tag) {
            return Err(PyAssertionError::new_err("Invalid MAC"));
        }

        Ok(plaintext.to_vec())
    }
}

#[pyclass]
pub struct XChaChaPoly1305 {
    key: Vec<u8>,
    rounds: usize,
}

#[pymethods]
impl XChaChaPoly1305 {
    #[new]
    pub fn new(key: Vec<u8>, r: Option<usize>) -> PyResult<XChaChaPoly1305> {
        let rounds;

        if r.is_some() {
            rounds = r.unwrap();
        } else {
            rounds = 20;
        }

        if key.len() != 32 {
            return Err(PyAssertionError::new_err("Key must be 32 bytes in length."));
        }

        if rounds < 1 {
            return Err(PyAssertionError::new_err("Rounds must be at least 1"));
        }

        Ok(XChaChaPoly1305 { key, rounds })
    }

    fn key(&self, nonce: &[u8]) -> (Vec<u8>, [u8; 12]) {
        let mut chacha_nonce = [0u8; 12];
        chacha_nonce[4..].copy_from_slice(&nonce[16..24]);

        let subkey = backends::hchacha(&self.key, nonce);

        (subkey.to_vec(), chacha_nonce)
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8], aad: &[u8]) -> PyResult<Vec<u8>> {
        let (subkey, chacha_nonce) = self.key(nonce);

        let chacha = ChaChaPoly1305::new(subkey, Some(self.rounds))?;

        chacha.encrypt(plaintext, &chacha_nonce, aad).into()
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8], aad: &[u8]) -> PyResult<Vec<u8>> {
        let (subkey, chacha_nonce) = self.key(nonce);

        let chacha = ChaChaPoly1305::new(subkey, Some(self.rounds))?;

        chacha.decrypt(ciphertext, &chacha_nonce, aad)
    }
}

#[pyfunction]
pub fn encrypt(
    key: Vec<u8>,
    plaintext: Vec<u8>,
    iv: Option<Vec<u8>>,
    data: Option<Vec<u8>>,
    rounds: Option<usize>,
) -> PyResult<Cow<'static, [u8]>> {
    let cipher = XChaChaPoly1305::new(key.clone(), rounds.clone())?;

    let nonce = iv.unwrap_or(vec![0u8; 24]);
    let aad = data.unwrap_or_default();

    let data = cipher.encrypt(&plaintext, &nonce, &aad)?;

    Ok(data.into())
}

#[pyfunction]
pub fn decrypt(
    key: Vec<u8>,
    plaintext: Vec<u8>,
    iv: Option<Vec<u8>>,
    data: Option<Vec<u8>>,
    rounds: Option<usize>,
) -> PyResult<Cow<'static, [u8]>> {
    let cipher = XChaChaPoly1305::new(key, rounds)?;

    let nonce = iv.unwrap_or(vec![0u8; 24]);
    let aad = data.unwrap_or_default();

    let data = cipher.decrypt(&plaintext, &nonce, &aad)?;

    Ok(data.into())
}
