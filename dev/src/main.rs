use arachne_ws::{WebSocketServer, WebSocketMessage};

fn main() -> Result<(), arachne_ws::WebSocketError> {
    // Bind the WebSocket server to an address
    let server = WebSocketServer::bind("127.0.0.1:8008")?;
    println!("WebSocket server listening on http://127.0.0.1:8008");

    // Handle incoming connections
    for mut connection in server.incoming() {
        if let Ok(message) = connection.read_message() {
            match message {
                WebSocketMessage::Text(text) => {
                    println!("Received text message: {}", text);
                    // Echo the message back to the client
                    connection.send_message(WebSocketMessage::Text(text))?;
                }
                WebSocketMessage::Binary(data) => {
                    println!("Received binary message: {:?}", data);
                }
                _ => {}
            }
        }
    }
    Ok(())
}