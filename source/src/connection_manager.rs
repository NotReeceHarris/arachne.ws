use std::sync::{Arc, Mutex};
use crate::WebSocketConnection;

/// Manages a list of active WebSocket connections.
pub struct ConnectionManager {
    connections: Arc<Mutex<Vec<Arc<Mutex<WebSocketConnection>>>>>,
}

impl ConnectionManager {
    /// Create a new `ConnectionManager`.
    pub fn new() -> Self {
        ConnectionManager {
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a new connection to the manager.
    pub fn add_connection(&self, connection: Arc<Mutex<WebSocketConnection>>) {
        self.connections.lock().unwrap().push(connection);
    }

    /// Remove closed connections from the manager.
    pub fn cleanup_closed_connections(&self) {
        self.connections.lock().unwrap().retain(|conn| !conn.lock().unwrap().is_closed());
    }

    /// Broadcast a message to all active connections.
    pub fn broadcast_message(&self, message: crate::WebSocketMessage) {
        let connections = self.connections.lock().unwrap();

        println!("Broadcasting message to {} connections", connections.len());

        for conn in connections.iter() {
            let mut conn = conn.lock().unwrap();
            if let Err(e) = conn.send_message(message.clone()) {
                eprintln!("Failed to send message: {:?}", e);
            }
        }
    }
}