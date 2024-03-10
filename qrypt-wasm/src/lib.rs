use wasm_bindgen::prelude::{wasm_bindgen, JsError};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, qrypt-wasm!");
}

#[wasm_bindgen]
pub fn encrypt_to_b64png(input: &str, password: &str) -> Result<String, JsError> {
    Ok(qrypt_core::encrypt_to_b64png(input, password)?)
}
