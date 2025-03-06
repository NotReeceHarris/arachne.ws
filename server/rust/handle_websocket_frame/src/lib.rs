use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;

#[wasm_bindgen]
pub fn handle_websocket_frame(data: &[u8]) -> JsValue {
    // Directly access wasm memory (faster, less GC overhead)

    // First byte contains the opcode
    let opcode = data[0] & 0x0f;
    let is_masked = (data[1] & 0x80) != 0;
    let mut payload_length = (data[1] & 0x7f) as usize;

    let mut offset = 2;

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

    let masking_key = if is_masked {
        Some([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
    } else {
        None
    };
    
    if is_masked {
        offset += 4;
    }

    let payload_slice = &data[offset..offset + payload_length];

    // Directly copy and unmask the payload into WebAssembly memory
    let payload_js = Uint8Array::new_with_length(payload_length as u32);
    if let Some(key) = masking_key {
        for (i, &byte) in payload_slice.iter().enumerate() {
            payload_js.set_index(i as u32, byte ^ key[i % 4]);
        }
    } else {
        // If not masked, directly copy the data
        payload_js.copy_from(&payload_slice.to_vec());
    }

    // Use wasm memory and avoid unnecessary JS Object creation
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &"opcode".into(), &opcode.into()).unwrap();
    js_sys::Reflect::set(&result, &"payload".into(), &payload_js.into()).unwrap();

    result.into()
}
