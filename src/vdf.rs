use once_cell::sync::Lazy;
use vdf::{VDFParams, VDF, WesolowskiVDFParams};

const BITS: u16 = 512;
pub const DEFAULT_T: u64 = 10_000;

static VDF_INSTANCE: Lazy<vdf::WesolowskiVDF> =
    Lazy::new(|| WesolowskiVDFParams(BITS).new());

pub fn generate(msg: &str, t: u64) -> Result<Vec<u8>, String> {
    VDF_INSTANCE
        .solve(msg.as_bytes(), t)
        .map_err(|e| format!("{:?}", e))
}

pub fn verify_proof(msg: &str, t: u64, proof_hex: &str) -> bool {
    let Ok(proof) = hex::decode(proof_hex) else {
        return false;
    };
    VDF_INSTANCE.verify(msg.as_bytes(), t, &proof).is_ok()
}
