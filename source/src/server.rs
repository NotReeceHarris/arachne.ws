use std::net::{TcpListener, TcpStream};
use std::io::{Read};
use crate::{WebSocketError, handshake, frame, WebSocketMessage, HandshakeError};

pub struct WebSocketServer {
    listener: TcpListener,
}

impl WebSocketServer {
    pub fn bind(addr: &str) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr)?;
        Ok(WebSocketServer { listener })
    }

    pub fn incoming(&self) -> impl Iterator<Item = WebSocketConnection> + '_ {
        self.listener.incoming().filter_map(|stream| {
            match stream {
                Ok(stream) => {
                    match WebSocketConnection::new(stream) {
                        Ok(conn) => Some(conn),
                        Err(e) => {
                            eprintln!("Failed to establish connection: {:?}", e);
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

pub struct WebSocketConnection {
    stream: TcpStream,
}

impl WebSocketConnection {
    pub fn new(mut stream: TcpStream) -> Result<Self, WebSocketError> {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string(); // Convert to owned String
    
        println!("Received HTTP request: {}", request);
    
        // Check if it's a WebSocket upgrade request
        if request.contains("Upgrade: websocket") {
            handshake::perform_handshake(&mut stream, &request)?; // Pass the request to perform_handshake
            Ok(WebSocketConnection { stream })
        } else {
            // Handle HTTP request (if needed)
            Err(WebSocketError::HandshakeError(HandshakeError::InvalidKey(
                "Not a WebSocket request".to_string(),
            )))
        }
    }

    pub fn read_message(&mut self) -> Result<WebSocketMessage, WebSocketError> {
        let frame = frame::parse_frame(&mut self.stream)?;
        match frame.opcode {
            frame::OpCode::Text => {
                let text = String::from_utf8(frame.payload)
                    .map_err(|_| WebSocketError::InvalidFrame)?;
                Ok(WebSocketMessage::Text(text))
            }
            frame::OpCode::Binary => {
                Ok(WebSocketMessage::Binary(frame.payload))
            }
            frame::OpCode::Ping => {
                Ok(WebSocketMessage::Ping(frame.payload))
            }
            frame::OpCode::Pong => {
                Ok(WebSocketMessage::Pong(frame.payload))
            }
            frame::OpCode::Close => {
                Ok(WebSocketMessage::Close)
            }
            _ => Err(WebSocketError::InvalidFrame),
        }
    }

    pub fn send_message(&mut self, message: WebSocketMessage) -> Result<(), WebSocketError> {
        let frame = match message {
            WebSocketMessage::Text(text) => {
                println!("Sending text message: {}", text);
                frame::WebSocketFrame {
                    opcode: frame::OpCode::Text,
                    payload: text.into_bytes(),
                }
            }
            WebSocketMessage::Binary(data) => {
                println!("Sending binary message: {:?}", data);
                frame::WebSocketFrame {
                    opcode: frame::OpCode::Binary,
                    payload: data,
                }
            }
            WebSocketMessage::Ping(data) => {
                println!("Sending ping message: {:?}", data);
                frame::WebSocketFrame {
                    opcode: frame::OpCode::Ping,
                    payload: data,
                }
            }
            WebSocketMessage::Pong(data) => {
                println!("Sending pong message: {:?}", data);
                frame::WebSocketFrame {
                    opcode: frame::OpCode::Pong,
                    payload: data,
                }
            }
            WebSocketMessage::Close => {
                println!("Sending close message");
                frame::WebSocketFrame {
                    opcode: frame::OpCode::Close,
                    payload: Vec::new(),
                }
            }
        };

        // Debug: Print the frame being sent
        println!("Sending frame: {:?}", frame);

        frame::construct_frame(&mut self.stream, frame)?;
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        // Try to clone the stream to check if it is still open
        match self.stream.try_clone() {
            Ok(_) => false, // Connection is still open
            Err(_) => true, // Connection is closed or in an invalid state
        }
    }
}