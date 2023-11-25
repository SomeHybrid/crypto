use crate::utils::from_le_bytes;

pub struct Poly1305 {
    r: [u32; 5],
    h: [u32; 5],
    pad: [u32; 4],
    finished: bool,
    output: [u8; 16],
}

impl Poly1305 {
    fn block(&mut self, m: &[u8], partial: bool) {
        let hibit: u32 = if partial { 0 } else { 1 << 24 };
        let s = [self.r[1] * 5, self.r[2] * 5, self.r[3] * 5, self.r[4] * 5];

        self.h[0] += (from_le_bytes(&m[0..4])) & 0x3ffffff;
        self.h[1] += (from_le_bytes(&m[3..7]) >> 2) & 0x3ffffff;
        self.h[2] += (from_le_bytes(&m[6..10]) >> 4) & 0x3ffffff;
        self.h[3] += (from_le_bytes(&m[9..13]) >> 6) & 0x3ffffff;
        self.h[4] += (from_le_bytes(&m[12..16]) >> 8) | hibit;

        let mut d = [0u64; 5];

        for i in 0..5 {
            for j in 0..5 {
                let num = if j > i { s[4 - (j - i)] } else { self.r[i - j] };
                d[i] += self.h[j] as u64 * num as u64;
            }
        }

        let mut c: u32;

        for i in 0..4 {
            self.h[i] = d[i] as u32 & 0x3ffffff;
            c = (d[i] >> 26) as u32;
            d[i + 1] += c as u64;
        }

        c = (d[4] >> 26) as u32;
        self.h[4] = d[4] as u32 & 0x3ff_ffff;
        self.h[0] += c * 5;

        c = self.h[0] >> 26;
        self.h[0] &= 0x3ff_ffff;
        self.h[1] += c;
    }

    fn finish(&mut self) {
        let mut h = self.h.clone();

        let mut c: u32 = 0;

        for i in 1..5 {
            h[i] += c;
            c = h[i] >> 26;
            h[i] &= 0x3ffffff;
        }

        h[0] += c * 5;
        c = h[0] >> 26;
        h[0] = h[0] & 0x3ffffff;
        h[1] += c;

        let mut g = [0u32; 5];

        let mut c = 5u32;

        for i in 0..4 {
            g[i] = h[i].wrapping_add(c);
            c = g[i] >> 26;
            g[i] &= 0x3ffffff;
        }

        g[4] = h[4].wrapping_add(c).wrapping_sub(1 << 26);

        let mut mask = (g[4] >> (32 - 1)).wrapping_sub(1);

        for i in g.iter_mut() {
            *i &= mask;
        }

        mask = !mask;

        for i in 0..5 {
            h[i] = (h[i] & mask) | g[i];
        }

        h[0] |= h[1] << 26;
        h[1] = (h[1] >> 6) | (h[2] << 20);
        h[2] = (h[2] >> 12) | (h[3] << 14);
        h[3] = (h[3] >> 18) | (h[4] << 8);

        let mut f: u64;
        f = h[0] as u64 + self.pad[0] as u64;
        h[0] = f as u32;
        f = h[1] as u64 + self.pad[1] as u64 + (f >> 32);
        h[1] = f as u32;
        f = h[2] as u64 + self.pad[2] as u64 + (f >> 32);
        h[2] = f as u32;
        f = h[3] as u64 + self.pad[3] as u64 + (f >> 32);
        h[3] = f as u32;

        self.finished = true;

        let mut output = [0u8; 16];

        for i in 0..4 {
            output[i * 4..(i + 1) * 4].clone_from_slice(&h[i].to_le_bytes());
        }

        self.output = output;
    }
}

impl Poly1305 {
    pub fn new(key: &[u8]) -> Poly1305 {
        let mut r = [0u32; 5];
        r[0] = (from_le_bytes(&key[0..4])) & 0x3ffffff;
        r[1] = (from_le_bytes(&key[3..7]) >> 2) & 0x3ffff03;
        r[2] = (from_le_bytes(&key[6..10]) >> 4) & 0x3ffc0ff;
        r[3] = (from_le_bytes(&key[9..13]) >> 6) & 0x3f03fff;
        r[4] = (from_le_bytes(&key[12..16]) >> 8) & 0x00fffff;

        let mut pad = [0u32; 4];
        pad[0] = from_le_bytes(&key[16..20]);
        pad[1] = from_le_bytes(&key[20..24]);
        pad[2] = from_le_bytes(&key[24..28]);
        pad[3] = from_le_bytes(&key[28..32]);

        let h = [0u32; 5];

        let finished = false;
        let output = [0u8; 16];

        Poly1305 {
            r,
            h,
            pad,
            finished,
            output,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        for chunk in data.chunks(16) {
            let mut m = [0u8; 16];
            m[..chunk.len()].clone_from_slice(chunk);
            self.block(&m, false);
        }
    }

    pub fn update_unpadded(&mut self, data: &[u8]) {
        for chunk in data.chunks(16) {
            if chunk.len() == 16 {
                self.block(chunk, false);
            } else {
                let mut m = [0u8; 16];
                m[..chunk.len()].copy_from_slice(chunk);
                m[chunk.len()] = 1;
                self.block(&m, true);
            }
        }
    }

    pub fn tag(&mut self) -> Vec<u8> {
        if !self.finished {
            self.finish();
        }

        self.output.to_vec()
    }

    pub fn verify(&mut self, other: &[u8]) -> bool {
        let tag = self.tag();
        let mut dif = 0u32;
        for i in 0..16 {
            dif |= (tag[i] ^ other[i]) as u32;
        }

        dif = (dif.wrapping_sub(1)) >> 31;
        if (dif & 1) != 0 {
            return true;
        } else {
            return false;
        }
    }
}
