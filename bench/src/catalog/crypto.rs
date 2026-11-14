//! Post-quantum, lattice, hash-based signature, and constant-time kernels (`#892–900`).
use sha2::{Digest, Sha256};

/// Kyber / ML-KEM prime modulus `q = 3329 = 13 * 256 + 1`.
pub const KYBER_Q: i32 = 3329;
/// Kyber / ML-KEM polynomial degree `N = 256`.
pub const KYBER_N: usize = 256;
/// Primitive 256th root of unity mod 3329 (`17^128 ≡ -1 mod 3329`, `17^256 ≡ 1 mod 3329`).
const KYBER_ZETA: i32 = 17;
/// `128^(-1) mod 3329 = 3303`.
const INV_128: i32 = 3303;

fn sha256_multi(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

#[inline]
const fn mod_q(x: i32) -> i32 {
    let mut r = x % KYBER_Q;
    if r < 0 {
        r += KYBER_Q;
    }
    r
}

const fn pow_mod_q(mut base: i32, mut exp: usize) -> i32 {
    let mut res = 1i32;
    base = mod_q(base);
    while exp > 0 {
        if exp & 1 == 1 {
            res = (res * base) % KYBER_Q;
        }
        base = (base * base) % KYBER_Q;
        exp >>= 1;
    }
    res
}

const fn bit_rev_7(x: usize) -> usize {
    let mut v = x & 0x7f;
    let mut r = 0usize;
    let mut i = 0;
    while i < 7 {
        r = (r << 1) | (v & 1);
        v >>= 1;
        i += 1;
    }
    r
}

const ZETAS: [i32; 128] = {
    let mut table = [0i32; 128];
    let mut i = 0usize;
    while i < 128 {
        table[i] = pow_mod_q(KYBER_ZETA, bit_rev_7(i));
        i += 1;
    }
    table
};

/// `#892` Forward ML-KEM/Kyber Number-Theoretic Transform (`N = 256, q = 3329`).
pub fn ntt_forward(a: &mut [i32; KYBER_N]) {
    for v in a.iter_mut() {
        *v = mod_q(*v);
    }
    let mut k = 1usize;
    let mut len = 128usize;
    while len >= 2 {
        let mut start = 0usize;
        while start < KYBER_N {
            let zeta = ZETAS[k];
            k += 1;
            for j in start..start + len {
                let t = (zeta * a[j + len]) % KYBER_Q;
                a[j + len] = mod_q(a[j] - t);
                a[j] = mod_q(a[j] + t);
            }
            start += 2 * len;
        }
        len >>= 1;
    }
}

/// `#893` Inverse ML-KEM/Kyber Number-Theoretic Transform (`N = 256, q = 3329`).
pub fn ntt_inverse(a: &mut [i32; KYBER_N]) {
    let mut k = 127usize;
    let mut len = 2usize;
    while len <= 128 {
        let mut start = 0usize;
        while start < KYBER_N {
            let zeta = ZETAS[k];
            k -= 1;
            for j in start..start + len {
                let t = a[j];
                a[j] = mod_q(t + a[j + len]);
                let diff = mod_q(a[j + len] - t);
                a[j + len] = (zeta * diff) % KYBER_Q;
            }
            start += 2 * len;
        }
        len <<= 1;
    }
    for v in a.iter_mut() {
        *v = (*v * INV_128) % KYBER_Q;
    }
}

/// `#894` Pointwise degree-2 polynomial multiplication in NTT domain over `Z_q[X]/(X^256 + 1)`.
pub fn ntt_basemul(a: &[i32; KYBER_N], b: &[i32; KYBER_N]) -> [i32; KYBER_N] {
    let mut r = [0i32; KYBER_N];
    for i in 0..64 {
        let zeta = ZETAS[64 + i];
        let p = 4 * i;
        r[p] = mod_q(a[p] * b[p] + ((a[p + 1] * b[p + 1]) % KYBER_Q) * zeta);
        r[p + 1] = mod_q(a[p] * b[p + 1] + a[p + 1] * b[p]);
        r[p + 2] = mod_q(a[p + 2] * b[p + 2] - ((a[p + 3] * b[p + 3]) % KYBER_Q) * zeta);
        r[p + 3] = mod_q(a[p + 2] * b[p + 3] + a[p + 3] * b[p + 2]);
    }
    r
}

fn sample_uniform_poly(seed: &[u8; 32], domain: u8) -> [i32; KYBER_N] {
    let mut poly = [0i32; KYBER_N];
    let mut idx = 0usize;
    let mut ctr = 0u32;
    while idx < KYBER_N {
        let block = sha256_multi(&[seed, &[domain], &ctr.to_le_bytes()]);
        ctr += 1;
        for &chunk in block.as_chunks::<2>().0 {
            if idx >= KYBER_N {
                break;
            }
            let cand = (u16::from_le_bytes(chunk) & 0x0fff) as i32;
            if cand < KYBER_Q {
                poly[idx] = cand;
                idx += 1;
            }
        }
    }
    poly
}

fn sample_small_noise(seed: &[u8; 32], domain: u8) -> [i32; KYBER_N] {
    // Centered ternary/CBD-1 noise in [-1, 1] per coefficient gives huge decryption margin (> 5x).
    let mut poly = [0i32; KYBER_N];
    for c in 0..8 {
        let block = sha256_multi(&[seed, &[domain, c as u8]]);
        for b_idx in 0..32 {
            let byte = block[b_idx];
            let a = (byte & 1) as i32;
            let b = ((byte >> 1) & 1) as i32;
            poly[c * 32 + b_idx] = mod_q(a - b);
        }
    }
    poly
}

/// `#895` Public/secret keypair for Ring-LWE (`N = 256, q = 3329`).
#[derive(Clone, Debug)]
pub struct LwePublicKey {
    pub seed: [u8; 32],
    pub t_hat: [i32; KYBER_N],
}

#[derive(Clone, Debug)]
pub struct LweSecretKey {
    pub s_hat: [i32; KYBER_N],
}

#[derive(Clone, Debug)]
pub struct LweCiphertext {
    pub u: [i32; KYBER_N],
    pub v: [i32; KYBER_N],
}

pub fn lwe_keygen(seed: &[u8; 32]) -> (LwePublicKey, LweSecretKey) {
    let mut a_hat = sample_uniform_poly(seed, 0x01);
    let mut s_hat = sample_small_noise(seed, 0x02);
    let mut e_hat = sample_small_noise(seed, 0x03);
    ntt_forward(&mut a_hat);
    ntt_forward(&mut s_hat);
    ntt_forward(&mut e_hat);
    let prod = ntt_basemul(&a_hat, &s_hat);
    let mut t_hat = [0i32; KYBER_N];
    for i in 0..KYBER_N {
        t_hat[i] = mod_q(prod[i] + e_hat[i]);
    }
    (LwePublicKey { seed: *seed, t_hat }, LweSecretKey { s_hat })
}

pub fn lwe_encrypt(pk: &LwePublicKey, msg: &[u8; 32], coins: &[u8; 32]) -> LweCiphertext {
    let mut a_hat = sample_uniform_poly(&pk.seed, 0x01);
    let mut r_hat = sample_small_noise(coins, 0x10);
    let e1 = sample_small_noise(coins, 0x11);
    let e2 = sample_small_noise(coins, 0x12);
    ntt_forward(&mut a_hat);
    ntt_forward(&mut r_hat);

    let mut u = ntt_basemul(&a_hat, &r_hat);
    ntt_inverse(&mut u);
    let mut v = ntt_basemul(&pk.t_hat, &r_hat);
    ntt_inverse(&mut v);

    let half_q = (KYBER_Q + 1) / 2;
    for i in 0..KYBER_N {
        let bit = ((msg[i / 8] >> (i % 8)) & 1) as i32;
        u[i] = mod_q(u[i] + e1[i]);
        v[i] = mod_q(v[i] + e2[i] + bit * half_q);
    }
    LweCiphertext { u, v }
}

pub fn lwe_decrypt(sk: &LweSecretKey, ct: &LweCiphertext) -> [u8; 32] {
    let mut u_hat = ct.u;
    ntt_forward(&mut u_hat);
    let mut prod = ntt_basemul(&sk.s_hat, &u_hat);
    ntt_inverse(&mut prod);

    let mut out = [0u8; 32];
    let q = KYBER_Q;
    let quarter = q / 4;
    let three_quarter = (3 * q) / 4;
    for i in 0..KYBER_N {
        let diff = mod_q(ct.v[i] - prod[i]);
        // Closest to q/2 encodes 1; closest to 0 encodes 0. Constant-time range check:
        let is_one = ((diff >= quarter) & (diff <= three_quarter)) as u8;
        out[i / 8] |= is_one << (i % 8);
    }
    out
}

/// `#896` Lamport one-time signature scheme over SHA-256 (32-byte digest / message).
#[derive(Clone, Debug)]
pub struct LamportSecretKey {
    pub zero: [[u8; 32]; 256],
    pub one: [[u8; 32]; 256],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LamportPublicKey {
    pub zero: [[u8; 32]; 256],
    pub one: [[u8; 32]; 256],
}

pub fn lamport_keygen(seed: &[u8; 32]) -> (LamportPublicKey, LamportSecretKey) {
    let mut sk = LamportSecretKey {
        zero: [[0u8; 32]; 256],
        one: [[0u8; 32]; 256],
    };
    let mut pk = LamportPublicKey {
        zero: [[0u8; 32]; 256],
        one: [[0u8; 32]; 256],
    };
    for i in 0..256 {
        let idx = (i as u16).to_le_bytes();
        sk.zero[i] = sha256_multi(&[seed, b"Z", &idx]);
        sk.one[i] = sha256_multi(&[seed, b"O", &idx]);
        pk.zero[i] = sha256_multi(&[&sk.zero[i]]);
        pk.one[i] = sha256_multi(&[&sk.one[i]]);
    }
    (pk, sk)
}

pub fn lamport_sign(sk: &LamportSecretKey, msg: &[u8]) -> [[u8; 32]; 256] {
    let digest = sha256_multi(&[msg]);
    let mut sig = [[0u8; 32]; 256];
    for i in 0..256 {
        let bit = (digest[i / 8] >> (i % 8)) & 1;
        sig[i] = if bit == 0 { sk.zero[i] } else { sk.one[i] };
    }
    sig
}

pub fn lamport_verify(pk: &LamportPublicKey, msg: &[u8], sig: &[[u8; 32]; 256]) -> bool {
    let digest = sha256_multi(&[msg]);
    let mut ok = true;
    for i in 0..256 {
        let bit = (digest[i / 8] >> (i % 8)) & 1;
        let expected = if bit == 0 { &pk.zero[i] } else { &pk.one[i] };
        let got = sha256_multi(&[&sig[i]]);
        ok &= ct_eq(&got, expected);
    }
    ok
}

/// `#897` Winternitz WOTS+ (`w = 16`, 64 message nibbles + 3 checksum nibbles = 67 chains).
const WOTS_CHAINS: usize = 67;

fn wots_chain(
    start: &[u8; 32],
    start_step: usize,
    steps: usize,
    pub_seed: &[u8; 32],
    chain_idx: usize,
) -> [u8; 32] {
    let mut cur = *start;
    for step in start_step..start_step + steps {
        cur = sha256_multi(&[pub_seed, &[chain_idx as u8, step as u8], &cur]);
    }
    cur
}

fn wots_digits(msg: &[u8]) -> [usize; WOTS_CHAINS] {
    let digest = sha256_multi(&[msg]);
    let mut digits = [0usize; WOTS_CHAINS];
    let mut csum = 0usize;
    for (i, b) in digest.iter().enumerate() {
        let hi = (b >> 4) as usize;
        let lo = (b & 0x0f) as usize;
        digits[2 * i] = hi;
        digits[2 * i + 1] = lo;
        csum += (15 - hi) + (15 - lo);
    }
    digits[64] = (csum >> 8) & 0x0f;
    digits[65] = (csum >> 4) & 0x0f;
    digits[66] = csum & 0x0f;
    digits
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WotsPublicKey {
    pub pub_seed: [u8; 32],
    pub nodes: [[u8; 32]; WOTS_CHAINS],
}

#[derive(Clone, Debug)]
pub struct WotsSecretKey {
    pub pub_seed: [u8; 32],
    pub nodes: [[u8; 32]; WOTS_CHAINS],
}

pub fn wots_keygen(sk_seed: &[u8; 32], pub_seed: &[u8; 32]) -> (WotsPublicKey, WotsSecretKey) {
    let mut sk_nodes = [[0u8; 32]; WOTS_CHAINS];
    let mut pk_nodes = [[0u8; 32]; WOTS_CHAINS];
    for i in 0..WOTS_CHAINS {
        sk_nodes[i] = sha256_multi(&[sk_seed, b"WOTS", &(i as u8).to_le_bytes()]);
        pk_nodes[i] = wots_chain(&sk_nodes[i], 0, 15, pub_seed, i);
    }
    (
        WotsPublicKey {
            pub_seed: *pub_seed,
            nodes: pk_nodes,
        },
        WotsSecretKey {
            pub_seed: *pub_seed,
            nodes: sk_nodes,
        },
    )
}

pub fn wots_sign(sk: &WotsSecretKey, msg: &[u8]) -> [[u8; 32]; WOTS_CHAINS] {
    let digits = wots_digits(msg);
    let mut sig = [[0u8; 32]; WOTS_CHAINS];
    for i in 0..WOTS_CHAINS {
        sig[i] = wots_chain(&sk.nodes[i], 0, digits[i], &sk.pub_seed, i);
    }
    sig
}

pub fn wots_verify(pk: &WotsPublicKey, msg: &[u8], sig: &[[u8; 32]; WOTS_CHAINS]) -> bool {
    let digits = wots_digits(msg);
    let mut ok = true;
    for i in 0..WOTS_CHAINS {
        let rem = 15 - digits[i];
        let computed = wots_chain(&sig[i], digits[i], rem, &pk.pub_seed, i);
        ok &= ct_eq(&computed, &pk.nodes[i]);
    }
    ok
}

/// `#898` Merkle authentication tree root & proof over SHA-256 leaves.
pub fn merkle_root_and_proof(leaves: &[[u8; 32]], leaf_idx: usize) -> ([u8; 32], Vec<[u8; 32]>) {
    assert!(!leaves.is_empty() && leaves.len().is_power_of_two());
    assert!(leaf_idx < leaves.len());
    let mut level: Vec<[u8; 32]> = leaves.iter().map(|l| sha256_multi(&[b"\x00", l])).collect();
    let mut idx = leaf_idx;
    let mut proof = Vec::new();
    while level.len() > 1 {
        proof.push(level[idx ^ 1]);
        level = level
            .as_chunks::<2>()
            .0
            .iter()
            .map(|[left, right]| sha256_multi(&[b"\x01", left, right]))
            .collect();
        idx >>= 1;
    }
    (level[0], proof)
}

pub fn merkle_verify(root: &[u8; 32], leaf: &[u8; 32], mut idx: usize, proof: &[[u8; 32]]) -> bool {
    let mut cur = sha256_multi(&[b"\x00", leaf]);
    for sib in proof {
        cur = if idx & 1 == 0 {
            sha256_multi(&[b"\x01", &cur, sib])
        } else {
            sha256_multi(&[b"\x01", sib, &cur])
        };
        idx >>= 1;
    }
    ct_eq(&cur, root)
}

/// `#899` Branchless constant-time byte-slice equality.
#[inline(never)]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for i in 0..a.len() {
        diff |= std::hint::black_box(a[i] ^ b[i]);
    }
    diff == 0
}

/// `#900` Branchless constant-time `u64` conditional selection (`choose_a ? a : b`).
#[inline(never)]
pub fn ct_select_u64(choose_a: bool, a: u64, b: u64) -> u64 {
    let mask = 0u64.wrapping_sub(std::hint::black_box(u64::from(choose_a)));
    (a & mask) | (b & !mask)
}

pub fn kernel_kyber_ntt(iters: usize) -> u64 {
    let mut a = [0i32; KYBER_N];
    for (i, slot) in a.iter_mut().enumerate() {
        *slot = (i as i32 * 73 + 19) % KYBER_Q;
    }
    let mut acc = 0u64;
    for r in 0..iters.max(4) {
        a[0] = (a[0] + r as i32) % KYBER_Q;
        ntt_forward(&mut a);
        ntt_inverse(&mut a);
        acc = acc.wrapping_add(a[r & 255] as u64);
    }
    acc
}

pub fn kernel_lwe_pke(iters: usize) -> u64 {
    let seed = [0x42u8; 32];
    let (pk, sk) = lwe_keygen(&seed);
    let mut msg = [0x5au8; 32];
    let coins = [0x99u8; 32];
    let mut acc = 0u64;
    for r in 0..iters.max(1) {
        msg[0] = r as u8;
        let ct = lwe_encrypt(&pk, &msg, &coins);
        let dec = lwe_decrypt(&sk, &ct);
        acc = acc.wrapping_add(u64::from(dec[0]));
    }
    acc
}

pub fn kernel_lamport_ots(iters: usize) -> u64 {
    let seed = [0x11u8; 32];
    let (pk, sk) = lamport_keygen(&seed);
    let mut acc = 0u64;
    for r in 0..iters.max(1) {
        let msg = r.to_le_bytes();
        let sig = lamport_sign(&sk, &msg);
        acc += u64::from(lamport_verify(&pk, &msg, &sig));
    }
    acc
}

pub fn kernel_wots_plus(iters: usize) -> u64 {
    let sk_seed = [0x22u8; 32];
    let pub_seed = [0x33u8; 32];
    let (pk, sk) = wots_keygen(&sk_seed, &pub_seed);
    let mut acc = 0u64;
    for r in 0..iters.max(1) {
        let msg = r.to_le_bytes();
        let sig = wots_sign(&sk, &msg);
        acc += u64::from(wots_verify(&pk, &msg, &sig));
    }
    acc
}

pub fn kernel_merkle_tree(iters: usize) -> u64 {
    let leaves: Vec<[u8; 32]> = (0..16u8).map(|i| [i; 32]).collect();
    let mut acc = 0u64;
    for r in 0..iters.max(4) {
        let idx = r & 15;
        let (root, proof) = merkle_root_and_proof(&leaves, idx);
        acc += u64::from(merkle_verify(&root, &leaves[idx], idx, &proof));
    }
    acc
}

pub fn kernel_constant_time_ops(iters: usize) -> u64 {
    let a = [0xa5u8; 64];
    let b = [0xa5u8; 64];
    let mut acc = 0u64;
    for r in 0..iters.max(32) {
        let eq = ct_eq(&a, &b);
        acc = acc.wrapping_add(ct_select_u64(eq, r as u64, 0));
    }
    acc
}
