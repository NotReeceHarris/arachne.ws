use arachne_ws::{WebSocketServer, WebSocketMessage};

fn main() -> Result<(), arachne_ws::WebSocketError> {
    // Bind the WebSocket server to an address
    let server = WebSocketServer::bind("127.0.0.1:8008")?;
    println!("WebSocket server listening on http://127.0.0.1:8008");

    // Handle incoming connections
    for mut connection in server.incoming() {
        // Spawn a new thread for each connection
        std::thread::spawn(move || {
            if let Err(e) = connection.handle_messages() {
                eprintln!("Connection error: {:?}", e);
            }
        });
    }
    Ok(())
}