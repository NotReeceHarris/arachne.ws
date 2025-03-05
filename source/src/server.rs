use std::net::{TcpListener, TcpStream};
use crate::{WebSocketError, handshake, frame};

/// Represents a WebSocket server.
pub struct WebSocketServer {
    listener: TcpListener,
}

impl WebSocketServer {
    /// Binds the WebSocket server to the specified address.
    pub fn bind(addr: &str) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr)?;
        Ok(WebSocketServer { listener })
    }

    /// Accepts incoming WebSocket connections.
    pub fn incoming(&self) -> impl Iterator<Item = WebSocketConnection> + '_ {
        self.listener.incoming().filter_map(|stream| {
            match stream {
                Ok(stream) => {
                    match WebSocketConnection::new(stream) {
                        Ok(conn) => Some(conn),
                        Err(e) => {
                            eprintln!("Failed to establish WebSocket connection: {:?}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to accept TCP connection: {}", e);
                    None
                }
            }
        })
    }
}

/// Represents a WebSocket connection.
pub struct WebSocketConnection {
    stream: TcpStream,
}

impl WebSocketConnection {
    /// Creates a new WebSocket connection from a TCP stream.
    pub fn new(mut stream: TcpStream) -> Result<Self, WebSocketError> {
        // Perform the WebSocket handshake
        handshake::perform_handshake(&mut stream)?;
        Ok(WebSocketConnection { stream })
    }

    /// Reads a WebSocket message from the connection.
    pub fn read_message(&mut self) -> Result<crate::WebSocketMessage, WebSocketError> {
        let frame = frame::parse_frame(&mut self.stream)?;
        match frame.opcode {
            frame::OpCode::Text => {
                let text = String::from_utf8(frame.payload)
                    .map_err(|_| WebSocketError::InvalidFrame)?;
                Ok(crate::WebSocketMessage::Text(text))
            }
            frame::OpCode::Binary => {
                Ok(crate::WebSocketMessage::Binary(frame.payload))
            }
            frame::OpCode::Ping => {
                Ok(crate::WebSocketMessage::Ping(frame.payload))
            }
            frame::OpCode::Pong => {
                Ok(crate::WebSocketMessage::Pong(frame.payload))
            }
            frame::OpCode::Close => {
                Ok(crate::WebSocketMessage::Close)
            }
            _ => Err(WebSocketError::InvalidFrame),
        }
    }

    /// Sends a WebSocket message over the connection.
    pub fn send_message(&mut self, message: crate::WebSocketMessage) -> Result<(), WebSocketError> {
        let frame = match message {
            crate::WebSocketMessage::Text(text) => frame::WebSocketFrame {
                opcode: frame::OpCode::Text,
                payload: text.into_bytes(),
            },
            crate::WebSocketMessage::Binary(data) => frame::WebSocketFrame {
                opcode: frame::OpCode::Binary,
                payload: data,
            },
            crate::WebSocketMessage::Ping(data) => frame::WebSocketFrame {
                opcode: frame::OpCode::Ping,
                payload: data,
            },
            crate::WebSocketMessage::Pong(data) => frame::WebSocketFrame {
                opcode: frame::OpCode::Pong,
                payload: data,
            },
            crate::WebSocketMessage::Close => frame::WebSocketFrame {
                opcode: frame::OpCode::Close,
                payload: Vec::new(),
            },
        };
        frame::construct_frame(&mut self.stream, frame)?;
        Ok(())
    }
}