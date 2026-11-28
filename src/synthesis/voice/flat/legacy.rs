//! Flat legacy CBC envelope. Signed DNNI archives use a separate reader.
use super::{Result, VoiceError};
use aes::Aes256;
use cbc::cipher::{BlockModeDecrypt, KeyIvInit, block_padding::Pkcs7};
use zeroize::Zeroizing;

const KEY_ENV: &str = "RYNTHV_FLAT_LEGACY_KEY";
const PAYLOAD_OFFSET: usize = 24;

pub(super) fn plausible_length(length: u64) -> bool {
    length >= 40 && (length - PAYLOAD_OFFSET as u64).is_multiple_of(16)
}

pub(super) fn key() -> Result<Zeroizing<[u8; 32]>> {
    let value = Zeroizing::new(std::env::var(KEY_ENV).map_err(|_| {
        VoiceError::new(format!(
            "Flat DNNI v3 legacy 需要在 {KEY_ENV} 中设置本地密钥"
        ))
    })?);
    let value = value.trim().as_bytes();
    let invalid = || VoiceError::new(format!("{KEY_ENV} 需要 64 位十六进制字符"));
    if value.len() != 64 {
        return Err(invalid());
    }
    let mut key = Zeroizing::new([0; 32]);
    for (target, pair) in key.iter_mut().zip(value.chunks_exact(2)) {
        let high = char::from(pair[0]).to_digit(16).ok_or_else(invalid)?;
        let low = char::from(pair[1]).to_digit(16).ok_or_else(invalid)?;
        *target = ((high << 4) | low) as u8;
    }
    Ok(key)
}

pub(super) fn unwrap(bytes: &mut [u8], key: &[u8; 32]) -> Result<std::ops::Range<usize>> {
    let (header, payload) = bytes.split_at_mut(PAYLOAD_OFFSET);
    let plain = cbc::Decryptor::<Aes256>::new_from_slices(key, &header[8..])
        .map_err(|_| VoiceError::new("Flat DNNI v3 legacy AES 参数无效"))?
        .decrypt_padded::<Pkcs7>(payload)
        .map_err(|_| VoiceError::new("Flat DNNI v3 legacy 无法解码：填充校验失败"))?;
    if !matches!(super::inline::version(plain), Some(1 | 2)) {
        return Err(VoiceError::new("Flat DNNI v3 legacy 的内层模型头无效"));
    }
    Ok(PAYLOAD_OFFSET..PAYLOAD_OFFSET + plain.len())
}
