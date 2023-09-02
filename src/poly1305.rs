use std::cmp::min;

fn u8tou32(items: &[u8]) -> u32 {
    u32::from_le_bytes([items[0], items[1], items[2], items[3]])
}

fn pad16(items: &[u8]) -> Vec<u8> {
    vec![0u8; 16 - (items.len() % 16)]
}

pub struct Poly1305 {
    r: [u32; 5],
    h: [u32; 5],
    pad: [u32; 4],
    leftover: usize,
    buffer: [u8; 16],
    finalized: bool,
}

impl Poly1305 {
    pub fn new(key: Vec<u8>) -> Poly1305 {
        let mut r = [0u32; 5];
        r[0] = (u8tou32(&key[0..4])) & 0x3ffffff;
        r[1] = (u8tou32(&key[3..7]) >> 2) & 0x3ffff03;
        r[2] = (u8tou32(&key[6..10]) >> 4) & 0x3ffc0ff;
        r[3] = (u8tou32(&key[9..13]) >> 6) & 0x3f03fff;
        r[4] = (u8tou32(&key[12..16]) >> 8) & 0x00fffff;

        let mut pad = [0u32; 4];
        pad[0] = u8tou32(&key[16..20]);
        pad[1] = u8tou32(&key[20..24]);
        pad[2] = u8tou32(&key[24..28]);
        pad[3] = u8tou32(&key[28..32]);

        let h = [0u32; 5];

        let leftover = 0;

        let buffer = [0u8; 16];

        let finalized = false;

        Poly1305 {
            r,
            h,
            pad,
            leftover,
            buffer,
            finalized,
        }
    }

    fn block(&mut self, m: &[u8]) {
        let hibit: u32 = if self.finalized { 0 } else { 1 << 24 };

        let s1 = self.r[1] * 5;
        let s2 = self.r[2] * 5;
        let s3 = self.r[3] * 5;
        let s4 = self.r[4] * 5;

        self.h[0] += (u8tou32(&m[0..4])) & 0x3ffffff;
        self.h[1] += (u8tou32(&m[3..7]) >> 2) & 0x3ffffff;
        self.h[2] += (u8tou32(&m[6..10]) >> 4) & 0x3ffffff;
        self.h[3] += (u8tou32(&m[9..13]) >> 6) & 0x3ffffff;
        self.h[4] += (u8tou32(&m[12..16]) >> 8) | hibit;

        let mut d = [0u64; 5];

        d[0] = (self.h[0] as u64 * self.r[0] as u64)
            + (self.h[1] as u64 * s4 as u64)
            + (self.h[2] as u64 * s3 as u64)
            + (self.h[3] as u64 * s2 as u64)
            + (self.h[4] as u64 * s1 as u64);
        d[1] = (self.h[0] as u64 * self.r[1] as u64)
            + (self.h[1] as u64 * self.r[0] as u64)
            + (self.h[2] as u64 * s4 as u64)
            + (self.h[3] as u64 * s3 as u64)
            + (self.h[4] as u64 * s2 as u64);
        d[2] = (self.h[0] as u64 * self.r[2] as u64)
            + (self.h[1] as u64 * self.r[1] as u64)
            + (self.h[2] as u64 * self.r[0] as u64)
            + (self.h[3] as u64 * s4 as u64)
            + (self.h[4] as u64 * s3 as u64);
        d[3] = (self.h[0] as u64 * self.r[3] as u64)
            + (self.h[1] as u64 * self.r[2] as u64)
            + (self.h[2] as u64 * self.r[1] as u64)
            + (self.h[3] as u64 * self.r[0] as u64)
            + (self.h[4] as u64 * s4 as u64);
        d[4] = (self.h[0] as u64 * self.r[4] as u64)
            + (self.h[1] as u64 * self.r[3] as u64)
            + (self.h[2] as u64 * self.r[2] as u64)
            + (self.h[3] as u64 * self.r[1] as u64)
            + (self.h[4] as u64 * self.r[0] as u64);

        let mut c = 0u32;

        for i in 0..5 {
            d[i] += c as u64;
            c = (d[i] >> 26) as u32;
            self.h[i] = d[i] as u32 & 0x3ffffff;
        }

        self.h[0] += c * 5;
        c = self.h[0] >> 26;
        self.h[0] &= 0x3ffffff;
        self.h[1] += c;
    }

    fn finish(&mut self) -> [u8; 16] {
        if self.leftover > 0 {
            let mut i = self.leftover;
            self.buffer[i] = 1;

            i += 1;

            for j in i..16 {
                self.buffer[j] = 0;
            }

            self.finalized = true;

            let tmp = self.buffer;
            self.block(&tmp);
        }

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

        h[0] = ((h[0]) | (h[1] << 26)) & 0xffffffff;
        h[1] = ((h[1] >> 6) | (h[2] << 20)) & 0xffffffff;
        h[2] = ((h[2] >> 12) | (h[3] << 14)) & 0xffffffff;
        h[3] = ((h[3] >> 18) | (h[4] << 8)) & 0xffffffff;

        let mut f: u64;
        f = h[0] as u64 + self.pad[0] as u64;
        h[0] = f as u32;
        f = h[1] as u64 + self.pad[1] as u64 + (f >> 32);
        h[1] = f as u32;
        f = h[2] as u64 + self.pad[2] as u64 + (f >> 32);
        h[2] = f as u32;
        f = h[3] as u64 + self.pad[3] as u64 + (f >> 32);
        h[3] = f as u32;

        let mut output = [0u8; 16];
        output[0..4].clone_from_slice(&h[0].to_le_bytes());
        output[4..8].clone_from_slice(&h[1].to_le_bytes());
        output[8..12].clone_from_slice(&h[2].to_le_bytes());
        output[12..16].clone_from_slice(&h[3].to_le_bytes());

        output
    }

    fn _update(&mut self, data: &[u8]) {
        let mut m = data;

        if self.leftover > 0 {
            let want = min(16 - self.leftover, m.len());
            for i in 0..want {
                self.buffer[self.leftover + i] = m[i];
            }
            m = &m[want..];
            self.leftover += want;

            if self.leftover < 16 {
                return;
            }

            let tmp = self.buffer;
            self.block(&tmp);

            self.leftover = 0;
        }

        while m.len() >= 16 {
            self.block(&m[0..16]);
            m = &m[16..];
        }

        for i in 0..m.len() {
            self.buffer[i] = m[i];
        }
        self.leftover = m.len();
    }

    pub fn update(&mut self, data: &[u8], pad: bool) {
        self._update(data);

        if pad {
            self._update(&pad16(data));
        }
    }

    pub fn tag(&mut self) -> Vec<u8> {
        let output = self.finish();

        output.to_vec()
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
