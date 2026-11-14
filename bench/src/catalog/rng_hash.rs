//! Pseudo-random number generators & hashing kernels (`#171–196`).
use sha2::{Digest, Sha256};

#[inline]
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

pub struct Xoshiro256StarStar([u64; 4]);

impl Xoshiro256StarStar {
    pub fn from_seed(mut seed: u64) -> Self {
        Self([
            splitmix64(&mut seed),
            splitmix64(&mut seed),
            splitmix64(&mut seed),
            splitmix64(&mut seed),
        ])
    }

    pub fn next_u64(&mut self) -> u64 {
        let res = self.0[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.0[1] << 17;
        self.0[2] ^= self.0[0];
        self.0[3] ^= self.0[1];
        self.0[1] ^= self.0[2];
        self.0[0] ^= self.0[3];
        self.0[2] ^= t;
        self.0[3] = self.0[3].rotate_left(45);
        res
    }
}

pub struct Pcg64Dxsm {
    state: u128,
    inc: u128,
}

impl Pcg64Dxsm {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            state: u128::from(seed).wrapping_mul(0xda94_2042_e4dd_58b5) | 1,
            inc: 0xda94_2042_e4dd_58b5_da94_2042_e4dd_58b5 | 1,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        const MUL: u64 = 0xda94_2042_e4dd_58b5;
        let state = self.state;
        self.state = state.wrapping_mul(MUL as u128).wrapping_add(self.inc);
        let mut hi = (state >> 64) as u64;
        let lo = (state as u64) | 1;
        hi ^= hi >> 32;
        hi = hi.wrapping_mul(MUL);
        hi ^= hi >> 48;
        hi = hi.wrapping_mul(lo);
        hi
    }
}

#[inline]
pub fn wyrand(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_add(0xa076_1d64_78bd_642f);
    let t = u128::from(*seed) * u128::from(*seed ^ 0xe703_7ed1_a0b4_28db);
    ((t >> 64) ^ t) as u64
}

pub fn chacha8_block(key: &[u32; 8], counter: u32, nonce: &[u32; 3]) -> [u32; 16] {
    #[inline]
    fn qr(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = (x[d] ^ x[a]).rotate_left(16);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = (x[b] ^ x[c]).rotate_left(12);
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = (x[d] ^ x[a]).rotate_left(8);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = (x[b] ^ x[c]).rotate_left(7);
    }
    let init = [
        0x6170_7865,
        0x3320_646e,
        0x7962_2d32,
        0x6b20_6574,
        key[0],
        key[1],
        key[2],
        key[3],
        key[4],
        key[5],
        key[6],
        key[7],
        counter,
        nonce[0],
        nonce[1],
        nonce[2],
    ];
    let mut x = init;
    for _ in 0..4 {
        qr(&mut x, 0, 4, 8, 12);
        qr(&mut x, 1, 5, 9, 13);
        qr(&mut x, 2, 6, 10, 14);
        qr(&mut x, 3, 7, 11, 15);
        qr(&mut x, 0, 5, 10, 15);
        qr(&mut x, 1, 6, 11, 12);
        qr(&mut x, 2, 7, 8, 13);
        qr(&mut x, 3, 4, 9, 14);
    }
    for i in 0..16 {
        x[i] = x[i].wrapping_add(init[i]);
    }
    x
}

pub fn fnv1a_64(data: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

pub fn fxhash_64(data: &[u8]) -> u64 {
    const K: u64 = 0x517c_c1b7_2722_0a95;
    let mut h = 0u64;
    let (chunks, rem) = data.as_chunks::<8>();
    for &c in chunks {
        let w = u64::from_le_bytes(c);
        h = (h.rotate_left(5) ^ w).wrapping_mul(K);
    }
    for &b in rem {
        h = (h.rotate_left(5) ^ u64::from(b)).wrapping_mul(K);
    }
    h
}

pub fn siphash24(k0: u64, k1: u64, data: &[u8]) -> u64 {
    #[inline]
    fn sipround(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
        *v0 = v0.wrapping_add(*v1);
        *v1 = v1.rotate_left(13) ^ *v0;
        *v0 = v0.rotate_left(32);
        *v2 = v2.wrapping_add(*v3);
        *v3 = v3.rotate_left(16) ^ *v2;
        *v0 = v0.wrapping_add(*v3);
        *v3 = v3.rotate_left(21) ^ *v0;
        *v2 = v2.wrapping_add(*v1);
        *v1 = v1.rotate_left(17) ^ *v2;
        *v2 = v2.rotate_left(32);
    }
    let mut v0 = 0x736f_6d65_7073_6575 ^ k0;
    let mut v1 = 0x646f_7261_6e64_6f6d ^ k1;
    let mut v2 = 0x6c79_6765_6e65_7261 ^ k0;
    let mut v3 = 0x7465_6462_7974_6573 ^ k1;
    let (chunks, rem) = data.as_chunks::<8>();
    for &c in chunks {
        let m = u64::from_le_bytes(c);
        v3 ^= m;
        sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= m;
    }
    let mut tail = ((data.len() as u64) & 0xff) << 56;
    for (i, &b) in rem.iter().enumerate() {
        tail |= u64::from(b) << (8 * i);
    }
    v3 ^= tail;
    sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 ^= tail;
    v2 ^= 0xff;
    for _ in 0..4 {
        sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    }
    v0 ^ v1 ^ v2 ^ v3
}

pub fn crc32c_sw(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}

pub fn kernel_splitmix_xoshiro(iters: usize) -> u64 {
    let mut rng = Xoshiro256StarStar::from_seed(0xdead_beef_cafe_babe);
    let mut acc = 0u64;
    for _ in 0..iters.max(64) {
        acc ^= rng.next_u64();
    }
    acc
}

pub fn kernel_pcg64_wyrand(iters: usize) -> u64 {
    let mut pcg = Pcg64Dxsm::from_seed(0x1234_5678_9abc_def0);
    let mut wy = 0x5555_aaaa_u64;
    let mut acc = 0u64;
    for _ in 0..iters.max(64) {
        acc ^= pcg.next_u64() ^ wyrand(&mut wy);
    }
    acc
}

pub fn kernel_chacha8_stream(iters: usize) -> u64 {
    let key = [1, 2, 3, 4, 5, 6, 7, 8];
    let nonce = [9, 10, 11];
    let mut acc = 0u64;
    for ctr in 0..iters.max(8) {
        let block = chacha8_block(&key, ctr as u32, &nonce);
        acc = acc.wrapping_add(u64::from(block[0]) ^ u64::from(block[15]));
    }
    acc
}

pub fn kernel_noncrypto_hashes(iters: usize) -> u64 {
    let mut buf = [0u8; 256];
    for (i, b) in buf.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(31).wrapping_add(7);
    }
    let mut acc = 0u64;
    for r in 0..iters.max(16) {
        buf[0] = r as u8;
        acc ^= fnv1a_64(&buf) ^ fxhash_64(&buf) ^ siphash24(1, 2, &buf);
    }
    acc
}

pub fn kernel_crc32c(iters: usize) -> u64 {
    let mut buf = [0xaau8; 256];
    let mut acc = 0u64;
    for r in 0..iters.max(16) {
        buf[0] = r as u8;
        acc = acc.wrapping_add(u64::from(crc32c_sw(&buf)));
    }
    acc
}

pub fn kernel_sha256_digest(iters: usize) -> u64 {
    let mut buf = [0x3cu8; 512];
    let mut acc = 0u64;
    for r in 0..iters.max(8) {
        buf[0] = r as u8;
        let d: [u8; 32] = Sha256::digest(buf).into();
        acc ^= u64::from_le_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]]);
    }
    acc
}
