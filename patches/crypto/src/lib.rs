use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn base64_encode(input: &str) -> String {
    STANDARD.encode(input.as_bytes())
}

#[wasm_bindgen]
pub fn base64_decode(input: &str) -> Result<String, JsValue> {
    let bytes = STANDARD.decode(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
