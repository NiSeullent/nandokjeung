use crate::error::{CompileError, Result};
use crate::obfuscator::{self, ObfuscateMode, ObfuscateOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;

const FORTRESS_VERSION: u8 = 2;
const MAGIC: &str = "FORTRESSv2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FortressEnvelope {
    pub magic: String,
    pub version: u8,
    pub nonce: String,
    pub noise: Vec<usize>,
    pub shards: Vec<String>,
}

pub fn server_secret() -> String {
    env::var("NANDOKJEUNG_SECRET").unwrap_or_else(|_| "nandokjeung-dev-secret-change-me".into())
}

/// 강화 난독화: XOR + 노이즈 셔드 + 정신뼝! jp 다중 래핑
pub fn fortress_obfuscate(plaintext: &str, nonce: &str) -> Result<String> {
    let secret = server_secret();
    let key = derive_key(&secret, nonce);
    let encrypted = xor_bytes(plaintext.as_bytes(), &key);

    let shard_count = 4usize.max(1);
    let chunk_size = (encrypted.len().max(1) + shard_count - 1) / shard_count;
    let parts: Vec<Vec<u8>> = encrypted
        .chunks(chunk_size.max(1))
        .map(|c| c.to_vec())
        .collect();

    let mut shards = Vec::new();
    let mut noise = Vec::new();

    for (i, part) in parts.iter().enumerate() {
        let _idx = shards.len();
        let b64 = base64_encode(part);
        shards.push(wrap_shard_jp(&b64, i)?);

        if i % 2 == 0 {
            let nidx = shards.len();
            let decoy = format!("NOISE:{nonce}:{i}");
            shards.push(wrap_shard_jp(&decoy, i + 1000)?);
            noise.push(nidx);
        }
    }

    let envelope = FortressEnvelope {
        magic: MAGIC.to_string(),
        version: FORTRESS_VERSION,
        nonce: nonce.to_string(),
        noise,
        shards,
    };

    let json = serde_json::to_string(&envelope)
        .map_err(|e| CompileError::new("E080", e.to_string(), 0, 0))?;

    let opts = ObfuscateOptions {
        mode: ObfuscateMode::Heavy,
        split_chunks: 4,
    };
    Ok(obfuscator::obfuscate_text(&json, &opts))
}

/// 서버 전용 해독
pub fn fortress_reveal(outer_jp: &str) -> Result<String> {
    let json = obfuscator::reveal_source(outer_jp)?;
    let envelope: FortressEnvelope = serde_json::from_str(&json)
        .map_err(|e| CompileError::new("E081", format!("invalid fortress envelope: {e}"), 0, 0))?;

    if envelope.magic != MAGIC || envelope.version != FORTRESS_VERSION {
        return Err(CompileError::new("E082", "fortress version mismatch", 0, 0));
    }

    let noise: HashSet<usize> = envelope.noise.into_iter().collect();
    let mut recovered = Vec::new();

    for (i, jp) in envelope.shards.into_iter().enumerate() {
        if noise.contains(&i) {
            continue;
        }
        let inner = obfuscator::reveal_source(&jp)?;
        if inner.starts_with("NOISE:") {
            continue;
        }
        recovered.extend(base64_decode(&inner)?);
    }

    let secret = server_secret();
    let key = derive_key(&secret, &envelope.nonce);
    let plain = xor_bytes(&recovered, &key);

    String::from_utf8(plain)
        .map_err(|e| CompileError::new("E083", format!("utf8 decode failed: {e}"), 0, 0))
}

fn wrap_shard_jp(data: &str, salt: usize) -> Result<String> {
    let tagged = format!("{salt:04x}:{data}");
    let opts = ObfuscateOptions {
        mode: ObfuscateMode::Print,
        split_chunks: 2,
    };
    Ok(obfuscator::obfuscate_text(&tagged, &opts))
}

fn derive_key(secret: &str, nonce: &str) -> Vec<u8> {
    let seed = format!("{secret}:{nonce}:fortress");
    let mut key = vec![0u8; 32];
    for (i, b) in seed.bytes().enumerate() {
        key[i % 32] ^= b.wrapping_add((i as u8).wrapping_mul(31));
    }
    key
}

fn xor_bytes(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let payload = if let Some((_tag, data)) = s.split_once(':') {
        data
    } else {
        s
    };
    let bytes: Vec<u8> = payload.bytes().filter(|&b| b != b'=').collect();
    let mut out = Vec::new();
    for chunk in bytes.chunks(4) {
        let mut n = 0u32;
        for (i, &b) in chunk.iter().enumerate() {
            n |= (val(b).ok_or_else(|| CompileError::new("E084", String::from("bad base64"), 0, 0))? as u32)
                << (18 - i * 6);
        }
        out.push((n >> 16) as u8);
        if chunk.len() > 2 {
            out.push((n >> 8) as u8);
        }
        if chunk.len() > 3 {
            out.push(n as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fortress_roundtrip() {
        let text = "난독증 fortress 테스트 payload";
        let nonce = "test-nonce-001";
        let obf = fortress_obfuscate(text, nonce).expect("obf");
        let plain = fortress_reveal(&obf).expect("reveal");
        assert_eq!(plain, text);
    }
}
