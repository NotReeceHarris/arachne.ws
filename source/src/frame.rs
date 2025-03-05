/* 
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-------+-+-------------+-------------------------------+
|F|R|R|R| opcode|M| Payload len |    Extended payload length    |
|I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
|N|V|V|V|       |S|             |   (if payload len == 126/127) |
| |1|2|3|       |K|             |                               |
+-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
|     Extended payload length continued, if payload len == 127  |
+ - - - - - - - - - - - - - - - +-------------------------------+
|                               |Masking-key, if MASK set to 1  |
+-------------------------------+-------------------------------+
| Masking-key (continued)       |          Payload Data         |
+-------------------------------- - - - - - - - - - - - - - - +
|                     Payload Data continued ...                |
+---------------------------------------------------------------+
*/


use std::io::{Read, Write};
use crate::WebSocketError;

/// Represents a WebSocket frame.
#[derive(Debug)]
pub struct WebSocketFrame {
    pub opcode: OpCode,
    pub payload: Vec<u8>,
}

/// Represents the opcode of a WebSocket frame.
#[derive(Debug, PartialEq)]
pub enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl TryFrom<u8> for OpCode {
    type Error = WebSocketError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(OpCode::Continuation),
            0x1 => Ok(OpCode::Text),
            0x2 => Ok(OpCode::Binary),
            0x8 => Ok(OpCode::Close),
            0x9 => Ok(OpCode::Ping),
            0xA => Ok(OpCode::Pong),
            _ => Err(WebSocketError::InvalidFrame),
        }
    }
}

/// Parses a WebSocket frame from a TCP stream.
pub fn parse_frame(stream: &mut dyn Read) -> Result<WebSocketFrame, WebSocketError> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header)?;

    let _fin = (header[0] & 0x80) != 0;
    let opcode = OpCode::try_from(header[0] & 0x0F)?;
    let masked = (header[1] & 0x80) != 0;
    let mut payload_len = (header[1] & 0x7F) as usize;

    // Read extended payload length if necessary
    if payload_len == 126 {
        let mut len_bytes = [0u8; 2];
        stream.read_exact(&mut len_bytes)?;
        payload_len = u16::from_be_bytes(len_bytes) as usize;
    } else if payload_len == 127 {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes)?;
        payload_len = u64::from_be_bytes(len_bytes) as usize;
    }

    // Read masking key if present
    let masking_key = if masked {
        let mut key = [0u8; 4];
        stream.read_exact(&mut key)?;
        Some(key)
    } else {
        None
    };

    // Read payload
    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload)?;

    // Unmask the payload if necessary
    if let Some(key) = masking_key {
        for i in 0..payload.len() {
            payload[i] ^= key[i % 4];
        }
    }

    Ok(WebSocketFrame { opcode, payload })
}

/// Constructs a WebSocket frame and writes it to a TCP stream.
pub fn construct_frame(stream: &mut dyn Write, frame: WebSocketFrame) -> Result<(), WebSocketError> {
    let mut header = [0u8; 2];
    header[0] = 0x80 | (frame.opcode as u8); // FIN bit set + opcode

    println!("Constructing frame");

    if frame.payload.len() <= 125 {
        header[1] = frame.payload.len() as u8;
        stream.write_all(&header)?; // Write header for small payloads
    } else if frame.payload.len() <= 65535 {
        header[1] = 126;
        let len_bytes = (frame.payload.len() as u16).to_be_bytes();
        stream.write_all(&header)?; // Write header
        stream.write_all(&len_bytes)?; // Write extended payload length
    } else {
        header[1] = 127;
        let len_bytes = (frame.payload.len() as u64).to_be_bytes();
        stream.write_all(&header)?; // Write header
        stream.write_all(&len_bytes)?; // Write extended payload length
    }

    println!("Header: {:?}", header);
    println!("Payload: {:?}", frame.payload);

    stream.write_all(&frame.payload)?; // Write payload
    Ok(())
}