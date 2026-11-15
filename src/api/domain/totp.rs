//! Time-based one-time passwords (RFC 6238: HMAC-SHA1, 30-second step, 6 digits),
//! the scheme every authenticator app implements.
use hmac::{Hmac, Mac};
use sha1::Sha1;
use subtle::ConstantTimeEq;

const STEP_SECONDS: u64 = 30;
const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn base32_encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    let (mut buffer, mut bits) = (0u32, 0u32);
    for byte in bytes {
        buffer = (buffer << 8) | u32::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ALPHABET[((buffer >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((buffer << (5 - bits)) & 31) as usize] as char);
    }
    out
}

pub fn base32_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let (mut buffer, mut bits) = (0u32, 0u32);
    for c in text.bytes().filter(|c| *c != b'=' && *c != b' ') {
        let value = ALPHABET.iter().position(|a| *a == c.to_ascii_uppercase())?;
        buffer = (buffer << 5) | u32::try_from(value).ok()?;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

fn code_at(secret: &[u8], counter: u64) -> Option<u32> {
    let mut mac = Hmac::<Sha1>::new_from_slice(secret).ok()?;
    mac.update(&counter.to_be_bytes());
    let digest = mac.finalize().into_bytes();
    let offset = usize::from(digest[19] & 0x0f);
    let binary = u32::from_be_bytes([
        digest[offset] & 0x7f,
        digest[offset + 1],
        digest[offset + 2],
        digest[offset + 3],
    ]);
    Some(binary % 1_000_000)
}

/// Accepts the current code and one step either side (clock drift). The
/// comparison is constant-time.
pub fn verify(secret_base32: &str, code: &str, unix_seconds: u64) -> bool {
    let digits: String = code.chars().filter(|c| !c.is_whitespace()).collect();
    if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let Some(secret) = base32_decode(secret_base32) else {
        return false;
    };
    let counter = unix_seconds / STEP_SECONDS;
    let mut ok = 0u8;
    for step in [counter.saturating_sub(1), counter, counter + 1] {
        if let Some(expected) = code_at(&secret, step) {
            ok |= format!("{expected:06}").as_bytes().ct_eq(digits.as_bytes()).unwrap_u8();
        }
    }
    ok == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 appendix B, SHA-1 secret "12345678901234567890" (last six digits).
    #[test]
    fn matches_the_rfc_6238_test_vectors() {
        let secret = base32_encode(b"12345678901234567890");
        assert!(verify(&secret, "287082", 59));
        assert!(verify(&secret, "081804", 1_111_111_109));
        assert!(verify(&secret, "050471", 1_111_111_111));
        assert!(verify(&secret, "005924", 1_234_567_890));
        assert!(!verify(&secret, "287083", 59));
        assert!(!verify(&secret, "287082", 59 + 120), "codes expire");
        assert!(!verify(&secret, "28708", 59));
    }

    #[test]
    fn base32_round_trips() {
        let bytes = [0u8, 255, 16, 32, 77, 1, 2, 3, 4, 5, 6];
        assert_eq!(base32_decode(&base32_encode(&bytes)).unwrap(), bytes);
    }
}
