use std::sync::{Arc, Mutex};
use std::thread;
use arachne_ws::{WebSocketServer, WebSocketMessage, WebSocketError, ConnectionManager};

fn main() -> Result<(), WebSocketError> {
    // Bind the WebSocket server to an address
    let server = WebSocketServer::bind("127.0.0.1:8008")?;
    println!("WebSocket server listening on http://127.0.0.1:8008");

    // Create a connection manager
    let connection_manager = Arc::new(Mutex::new(ConnectionManager::new()));

    // Handle incoming connections
    for connection in server.incoming() {
        // Clone the connection manager
        let connection_manager = Arc::clone(&connection_manager);

        // Wrap the connection in an Arc<Mutex<...>>
        let connection = Arc::new(Mutex::new(connection));

        // Add the new connection to the manager
        connection_manager.lock().unwrap().add_connection(Arc::clone(&connection));

        // Spawn a new thread for each connection
        thread::spawn(move || {
            loop {
                // Read messages from the connection
                let mut connection = connection.lock().unwrap();
                match connection.read_message() {
                    Ok(message) => {
                        match message {
                            WebSocketMessage::Text(text) => {
                                println!("Received text message: {}", text);

                                // Broadcast the message to all connections
                                connection_manager.lock().unwrap().broadcast_message(WebSocketMessage::Text(text));
                            }
                            WebSocketMessage::Binary(data) => {
                                println!("Received binary message: {:?}", data);
                            }
                            WebSocketMessage::Close => {
                                println!("Client requested to close the connection.");
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading message: {:?}", e);
                        break;
                    }
                }
            }

            // Clean up closed connections
            connection_manager.lock().unwrap().cleanup_closed_connections();
            println!("Connection closed.");
        });
    }

    Ok(())
}