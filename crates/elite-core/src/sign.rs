//! Ed25519 signatures for launcher.json and manifests.
//!
//! The public key is compiled into the launcher, so a hacked or untrusted mirror cannot
//! push modified files or announcements: anything that fails verification is ignored.
//! Signature files are `<name>.sig` next to the signed file, base64 of the 64-byte signature.
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// Public key of the Elite update channel (base64, 32 bytes).
/// Replace with your own: `elite-publish keygen` and rebuild the launcher.
pub const PUBLIC_KEY_B64: &str = include_str!("../../../keys/public.key");

pub fn verify(data: &[u8], sig_b64: &str) -> bool {
    verify_with(PUBLIC_KEY_B64.trim(), data, sig_b64)
}

pub fn verify_with(pub_b64: &str, data: &[u8], sig_b64: &str) -> bool {
    let (Ok(pk), Ok(sig)) = (B64.decode(pub_b64.trim()), B64.decode(sig_b64.trim())) else { return false };
    let (Ok(pk), Ok(sig)) = (<[u8; 32]>::try_from(pk.as_slice()), <[u8; 64]>::try_from(sig.as_slice())) else { return false };
    let Ok(vk) = VerifyingKey::from_bytes(&pk) else { return false };
    vk.verify(data, &Signature::from_bytes(&sig)).is_ok()
}

pub fn sign(secret_b64: &str, data: &[u8]) -> Result<String, String> {
    let sk = B64.decode(secret_b64.trim()).map_err(|e| e.to_string())?;
    let sk: [u8; 32] = sk.as_slice().try_into().map_err(|_| "secret key must be 32 bytes")?;
    Ok(B64.encode(SigningKey::from_bytes(&sk).sign(data).to_bytes()))
}

/// New key pair as (secret_b64, public_b64).
pub fn keygen() -> (String, String) {
    let mut seed = [0u8; 32];
    getrandom_fill(&mut seed);
    let sk = SigningKey::from_bytes(&seed);
    (B64.encode(sk.to_bytes()), B64.encode(sk.verifying_key().to_bytes()))
}

fn getrandom_fill(b: &mut [u8]) {
    getrandom::getrandom(b).expect("OS random generator unavailable");
}
