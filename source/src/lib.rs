//! A minimal WebSocket framework implemented in Rust.
//!
//! This library provides the building blocks for creating WebSocket servers
//! without relying on external WebSocket packages.

// Re-export modules for public use
pub mod handshake;
pub mod frame;
pub mod connection_manager;
pub mod server;

// Re-export important types
pub use connection_manager::ConnectionManager;
pub use server::{WebSocketServer, WebSocketConnection};
pub use frame::{WebSocketFrame, OpCode};
pub use handshake::HandshakeError;

/// Represents a WebSocket connection.
pub struct WebSocket {
    stream: std::net::TcpStream,
}

impl WebSocket {
    /// Creates a new WebSocket connection from a TCP stream.
    pub fn new(stream: std::net::TcpStream) -> Self {
        WebSocket { stream }
    }

    /// Reads a WebSocket frame from the connection.
    pub fn read_frame(&mut self) -> Result<frame::WebSocketFrame, WebSocketError> {
        frame::parse_frame(&mut self.stream)
    }

    /// Sends a WebSocket frame over the connection.
    pub fn send_frame(&mut self, frame: frame::WebSocketFrame) -> Result<(), WebSocketError> {
        frame::construct_frame(&mut self.stream, frame)
    }
}

/// Represents a WebSocket message.
#[derive(Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

/// Errors that can occur during WebSocket operations.
#[derive(Debug)]
pub enum WebSocketError {
    IoError(std::io::Error),
    HandshakeError(HandshakeError), // Wrap HandshakeError
    InvalidFrame,
}

impl From<std::io::Error> for WebSocketError {
    fn from(err: std::io::Error) -> Self {
        WebSocketError::IoError(err)
    }
}

impl From<HandshakeError> for WebSocketError {
    fn from(err: HandshakeError) -> Self {
        WebSocketError::HandshakeError(err)
    }
}