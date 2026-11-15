//! Field encryption for SSNs (AES-256-GCM) and password hashing (Argon2id).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, bail, Context};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct Cipher(Aes256Gcm);

impl Cipher {
    pub fn from_base64(key_b64: &str) -> anyhow::Result<Self> {
        let key = base64::engine::general_purpose::STANDARD
            .decode(key_b64.trim())
            .context("DATA_ENCRYPTION_KEY must be base64")?;
        if key.len() != 32 {
            bail!("DATA_ENCRYPTION_KEY must decode to exactly 32 bytes");
        }
        Ok(Self(
            Aes256Gcm::new_from_slice(&key).map_err(|_| anyhow!("bad key"))?,
        ))
    }

    pub fn encrypt(&self, plaintext: &str) -> anyhow::Result<Vec<u8>> {
        let mut nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let ct = self
            .0
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
            .map_err(|_| anyhow!("encryption failed"))?;
        let mut out = nonce.to_vec();
        out.extend(ct);
        Ok(out)
    }

    pub fn decrypt(&self, data: &[u8]) -> anyhow::Result<String> {
        if data.len() < 13 {
            bail!("ciphertext too short");
        }
        let (nonce, ct) = data.split_at(12);
        let pt = self
            .0
            .decrypt(Nonce::from_slice(nonce), ct)
            .map_err(|_| anyhow!("decryption failed"))?;
        Ok(String::from_utf8(pt)?)
    }
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let salt = SaltString::encode_b64(&salt).map_err(|e| anyhow!("salt: {e}"))?;
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("hash: {e}"))?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

pub fn new_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}
