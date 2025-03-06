use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;

#[wasm_bindgen]
pub fn handle_websocket_frame(data: &[u8]) -> JsValue {
    let opcode = data[0] & 0x0f; // Extract the opcode from the first byte
    let is_masked = (data[1] & 0x80) != 0; // Check if the frame is masked
    let mut payload_length = (data[1] & 0x7f) as usize; // Extract payload length

    let mut offset = 2; // Start reading from byte 2

    // Handle extended payload length
    if payload_length == 126 {
        payload_length = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
    } else if payload_length == 127 {
        payload_length = u32::from_be_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;
        offset += 8;
    }

    // Read the masking key if the frame is masked
    let masking_key = if is_masked {
        Some([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
    } else {
        None
    };
    if is_masked {
        offset += 4;
    }

    // Get the payload slice
    let payload_slice = &data[offset..offset + payload_length];

    // Unmask the payload (if the frame is masked)
    let payload_js = if let Some(key) = masking_key {
        // Create a Uint8Array for the payload
        let payload_js = Uint8Array::new_with_length(payload_length as u32);

        // Unmask the payload in-place using SIMD-like xor operation directly on the slice
        payload_js.copy_from(&payload_slice.iter().enumerate().map(|(i, &byte)| byte ^ key[i % 4]).collect::<Vec<_>>().into_boxed_slice());

        payload_js
    } else {
        // If not masked, create a Uint8Array directly from the payload slice
        Uint8Array::from(payload_slice)
    };

    // Return the result as a plain JavaScript object
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &"opcode".into(), &opcode.into()).unwrap();
    js_sys::Reflect::set(&result, &"payload".into(), &payload_js.into()).unwrap();

    result.into()
}
