use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;

#[wasm_bindgen]
pub fn handle_websocket_frame(data: &[u8]) -> JsValue {
    // Parse WebSocket frame
    let opcode = data[0] & 0x0f; // First byte contains the opcode
    let is_masked = (data[1] & 0x80) != 0; // Second byte contains the MASK bit
    let mut payload_length = (data[1] & 0x7f) as usize; // Second byte contains the payload length

    let mut offset = 2; // Start reading after the first two bytes

    // Handle extended payload length
    if payload_length == 126 {
        payload_length = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
    } else if payload_length == 127 {
        // Note: We assume the payload length is within 32 bits
        payload_length = u32::from_be_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;
        offset += 8;
    }

    // Read the masking key (if present)
    let masking_key = if is_masked {
        Some([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ])
    } else {
        None
    };
    if is_masked {
        offset += 4;
    }

    // Get the payload slice
    let payload_slice = &data[offset..offset + payload_length];

    // Unmask the payload (if masked)
    let payload_js = if let Some(key) = masking_key {
        // Create a Uint8Array for the payload
        let payload_js = Uint8Array::new_with_length(payload_length as u32);

        // Unmask the payload directly into the Uint8Array
        for i in 0..payload_length {
            payload_js.set_index(i as u32, payload_slice[i] ^ key[i % 4]);
        }

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