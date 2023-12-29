use crate::utils::from_le_bytes;

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

pub struct ChaCha {
    key: Vec<u8>,
    rounds: usize,
}

impl ChaCha {
    pub fn new(key: &[u8], rounds: Option<usize>) -> ChaCha {
        ChaCha {
            key: key.to_vec(),
            rounds: rounds.unwrap_or(20) / 2,
        }
    }

    pub fn keystream(&self, nonce: &[u8], counter: u32) -> [u8; 64] {
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
            from_le_bytes(&nonce[4..8]),
            from_le_bytes(&nonce[8..12]),
            from_le_bytes(&nonce[12..]),
        ];

        for _ in 0..self.rounds {
            state = double_round(state);
        }

        let mut result = [0u8; 64];

        for (index, chunk) in state.iter().enumerate() {
            result[index * 4..].copy_from_slice(&chunk.to_le_bytes());
        }

        result
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8]) -> Vec<u8> {
        let mut ciphertext: Vec<u8> = Vec::new();

        for block in plaintext.chunks(64) {
            let keystream = self.keystream(nonce, 1);

            for (key, chunk) in block.iter().zip(keystream) {
                ciphertext.push(chunk ^ key);
            }
        }

        ciphertext
    }
}

pub fn hchacha(key: &[u8], nonce: &[u8], rounds: usize) -> [u8; 32] {
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
        from_le_bytes(&key[28..32]),
        from_le_bytes(&nonce[0..4]),
        from_le_bytes(&nonce[4..8]),
        from_le_bytes(&nonce[8..12]),
        from_le_bytes(&nonce[12..16]),
    ];

    for _ in 0..(rounds / 2) {
        state = double_round(state);
    }

    let mut result = [0u8; 32];

    for (result_chunk, chunk) in result
        .chunks_exact_mut(4)
        .zip(state[0..4].iter().chain(state[12..16].iter()))
    {
        result_chunk.copy_from_slice(&chunk.to_le_bytes());
    }

    result
}
