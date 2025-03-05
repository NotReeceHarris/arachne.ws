use std::io::{Read, Write};
use sha1::{Sha1, Digest};
use base64::engine::general_purpose;
use base64::Engine as _;
use crate::WebSocketError;

/// Performs the WebSocket handshake on a given TCP stream.
pub fn perform_handshake<S>(stream: &mut S, request: &str) -> Result<(), WebSocketError>
where
    S: Read + Write,
{
    println!("Performing handshake");

    // Debug: Print the raw request
    println!("Raw request:\n{}", request);

    // Extract the WebSocket key
    let key = extract_websocket_key(request).ok_or(WebSocketError::HandshakeError(
        HandshakeError::InvalidKey("Missing or invalid Sec-WebSocket-Key".to_string()),
    ))?;

    // Debug: Print the extracted key
    println!("Extracted Sec-WebSocket-Key: {}", key);

    // Generate the accept key
    let accept_key = generate_accept_key(&key);

    // Debug: Print the generated accept key
    println!("Generated Sec-WebSocket-Accept: {}", accept_key);

    // Send the HTTP 101 Switching Protocols response
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        accept_key
    );

    // Debug: Print the response
    println!("Sending response:\n{}", response);

    // Write the response to the stream
    stream.write_all(response.as_bytes())?;

    println!("Handshake complete");

    Ok(())
}

/// Extracts the `Sec-WebSocket-Key` from the HTTP request headers.
fn extract_websocket_key(request: &str) -> Option<String> {
    request
        .lines()
        .find(|line| line.starts_with("Sec-WebSocket-Key:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .map(|key| key.trim().to_string())
}

/// Generates the `Sec-WebSocket-Accept` key using the provided `Sec-WebSocket-Key`.
fn generate_accept_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"); // Magic GUID from RFC 6455
    general_purpose::STANDARD.encode(hasher.finalize())
}

/// Errors specific to the WebSocket handshake.
#[derive(Debug)]
pub enum HandshakeError {
    MissingKey,
    InvalidKey(String), // Accepts a String for detailed error messages
}

impl std::fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandshakeError::MissingKey => write!(f, "Missing Sec-WebSocket-Key header"),
            HandshakeError::InvalidKey(msg) => write!(f, "Invalid Sec-WebSocket-Key: {}", msg),
        }
    }
}

impl std::error::Error for HandshakeError {}