use wasm_bindgen::prelude::*;
use js_sys::{Uint8Array, Object};

#[wasm_bindgen]
pub fn handle_websocket_frame(data: &[u8]) -> Result<JsValue, JsValue> {
    // Validate input length
    if data.len() < 2 {
        return Err(JsValue::from_str("Invalid WebSocket frame: data too short"));
    }

    // Parse WebSocket frame header
    let opcode = data[0] & 0x0f;
    let is_masked = (data[1] & 0x80) != 0;
    let mut payload_length = (data[1] & 0x7f) as usize;

    let mut offset = 2;

    // Handle extended payload length
    if payload_length == 126 {
        if data.len() < offset + 2 {
            return Err(JsValue::from_str("Invalid WebSocket frame: data too short"));
        }
        payload_length = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
    } else if payload_length == 127 {
        if data.len() < offset + 8 {
            return Err(JsValue::from_str("Invalid WebSocket frame: data too short"));
        }
        payload_length = u64::from_be_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]) as usize;
        offset += 8;
    }

    // Handle masking key
    let masking_key = if is_masked {
        if data.len() < offset + 4 {
            return Err(JsValue::from_str("Invalid WebSocket frame: data too short"));
        }
        let key = [
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ];
        offset += 4;
        Some(key)
    } else {
        None
    };

    // Validate payload length
    if data.len() < offset + payload_length {
        return Err(JsValue::from_str("Invalid WebSocket frame: data too short"));
    }

    // Extract payload
    let payload_slice = &data[offset..offset + payload_length];
    let payload_js = Uint8Array::new_with_length(payload_length as u32);

    // Unmask payload if necessary
    if let Some(key) = masking_key {
        for i in 0..payload_length {
            payload_js.set_index(i as u32, payload_slice[i] ^ key[i % 4]);
        }
    } else {
        payload_js.copy_from(payload_slice);
    }

    // Create a plain JavaScript object
    let result = Object::new();
    js_sys::Reflect::set(&result, &"opcode".into(), &JsValue::from(opcode))?;
    js_sys::Reflect::set(&result, &"payload".into(), &payload_js.into())?;

    Ok(result.into())
}