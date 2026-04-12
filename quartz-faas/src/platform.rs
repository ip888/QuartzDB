//! Platform abstraction layer for QuartzDB
//!
//! Provides cross-platform implementations of time and random number functions.
//! On `wasm32`, delegates to `js_sys`. On all other targets, uses `std`.
//!
//! This enables the entire test suite to run on **any** host architecture
//! (`cargo test` on x86, ARM, etc.) while still using optimal JS intrinsics
//! in the production Cloudflare Workers build.

// ---------------------------------------------------------------------------
// Time
// ---------------------------------------------------------------------------

/// Current time in milliseconds since the Unix epoch.
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn now_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn now_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64
}

/// Current calendar year and month (1-indexed).
///
/// Returns `(year, month)` e.g. `(2026, 4)` for April 2026.
#[cfg(target_arch = "wasm32")]
pub fn current_year_month() -> (u32, u32) {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = now.get_month() + 1; // JS months are 0-indexed
    (year, month)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn current_year_month() -> (u32, u32) {
    // Derive year/month from millisecond epoch without external crates.
    // Accurate for dates 1970–2099 which is sufficient for billing keys.
    let epoch_days = (now_ms() / 86_400_000.0) as i64;

    // Civil date from days since 1970-01-01 (Euclidean-affine algorithm)
    let z = epoch_days + 719_468; // shift to 0000-03-01 epoch
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as u32; // day of era  [0, 146_096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (year as u32, m as u32)
}

// ---------------------------------------------------------------------------
// Random
// ---------------------------------------------------------------------------

/// Random `f64` in the range [0, 1).
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn random_f64() -> f64 {
    js_sys::Math::random()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn random_f64() -> f64 {
    // Thread-local xorshift64* PRNG — fast, reasonable quality, no deps.
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u64> = Cell::new({
            use std::time::{SystemTime, UNIX_EPOCH};
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            // Mix in thread-id bits for uniqueness across threads.
            let tid = std::thread::current().id();
            let tid_bits: u64 = tid.as_u64().get();
            t ^ tid_bits ^ 0x9e3779b97f4a7c15
        });
    }
    SEED.with(|s| {
        let mut x = s.get();
        // xorshift64*
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        s.set(x);
        let r = x.wrapping_mul(0x2545F4914F6CDD1D);
        (r >> 11) as f64 / (1u64 << 53) as f64
    })
}

// ---------------------------------------------------------------------------
// SHA-256 (pure Rust, re-exported for cross-module use)
// ---------------------------------------------------------------------------

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Compute the SHA-256 digest of `msg`.
pub fn sha256(msg: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (msg.len() as u64) * 8;
    let mut data = msg.to_vec();
    data.push(0x80);
    while (data.len() % 64) != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());

    for block in data.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7)
                ^ w[i - 15].rotate_right(18)
                ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17)
                ^ w[i - 2].rotate_right(19)
                ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

/// Hex-encode a SHA-256 digest.
pub fn sha256_hex(input: &str) -> String {
    sha256(input.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Compute HMAC-SHA256.
///
/// Standard RFC 2104 construction:
///   HMAC(K, m) = H((K' ⊕ opad) || H((K' ⊕ ipad) || m))
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;

    // Step 1: If key > block size, hash it.
    let key_prime: Vec<u8> = if key.len() > BLOCK_SIZE {
        sha256(key).to_vec()
    } else {
        key.to_vec()
    };

    // Step 2: Pad key to block size.
    let mut padded_key = [0u8; BLOCK_SIZE];
    padded_key[..key_prime.len()].copy_from_slice(&key_prime);

    // Step 3: XOR with ipad (0x36) and opad (0x5c).
    let mut i_key_pad = [0u8; BLOCK_SIZE];
    let mut o_key_pad = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        i_key_pad[i] = padded_key[i] ^ 0x36;
        o_key_pad[i] = padded_key[i] ^ 0x5c;
    }

    // Step 4: inner = H(i_key_pad || message)
    let mut inner_input = Vec::with_capacity(BLOCK_SIZE + message.len());
    inner_input.extend_from_slice(&i_key_pad);
    inner_input.extend_from_slice(message);
    let inner_hash = sha256(&inner_input);

    // Step 5: outer = H(o_key_pad || inner)
    let mut outer_input = Vec::with_capacity(BLOCK_SIZE + 32);
    outer_input.extend_from_slice(&o_key_pad);
    outer_input.extend_from_slice(&inner_hash);
    sha256(&outer_input)
}

/// Hex-encode a byte slice.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Constant-time comparison of two byte slices.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_ms_is_positive() {
        let t = now_ms();
        assert!(t > 0.0);
    }

    #[test]
    fn test_now_ms_monotonic() {
        let t1 = now_ms();
        let t2 = now_ms();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_random_f64_range() {
        for _ in 0..1000 {
            let r = random_f64();
            assert!(r >= 0.0 && r < 1.0, "random_f64 out of range: {r}");
        }
    }

    #[test]
    fn test_random_f64_not_constant() {
        // With overwhelming probability, 100 samples won't all be identical.
        let first = random_f64();
        let any_different = (0..100).any(|_| random_f64() != first);
        assert!(any_different, "random_f64 returned the same value 101 times");
    }

    #[test]
    fn test_current_year_month_reasonable() {
        let (year, month) = current_year_month();
        assert!(year >= 2024 && year <= 2100, "year out of range: {year}");
        assert!(month >= 1 && month <= 12, "month out of range: {month}");
    }

    #[test]
    fn test_sha256_known_vector() {
        // SHA-256("hello") = well-known value
        let hash = sha256_hex("hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hmac_sha256_rfc4231_vector1() {
        // RFC 4231 test vector #1:
        //   Key   = 0x0b repeated 20 times
        //   Data  = "Hi There"
        //   HMAC  = b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7
        let key = vec![0x0bu8; 20];
        let data = b"Hi There";
        let mac = hmac_sha256(&key, data);
        assert_eq!(
            hex_encode(&mac),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn test_hmac_sha256_rfc4231_vector2() {
        // RFC 4231 test vector #2:
        //   Key  = "Jefe"
        //   Data = "what do ya want for nothing?"
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(
            hex_encode(&mac),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }
}
