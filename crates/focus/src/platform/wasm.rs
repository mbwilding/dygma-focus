use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Focus {
    // TODO: serial
    pub(crate) response_buffer: Vec<u8>,
}
