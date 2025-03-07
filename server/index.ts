import type { Server } from 'http';
import { createHash } from 'crypto';
import Connection from './connection';
import { handle_websocket_frame } from 'handle_websocket_frame';
import warmup from './warmup';
import { DefaultOptions } from './options';
import type { Options } from './options';

import type { Duplex } from 'stream';
import type { ConnectionType } from './connection';

// RFC 6455 WebSocket protocol GUID for handshake key hashing (https://www.rfc-editor.org/rfc/rfc6455)
const RFC_6455 = '258EAFA5-E914-47DA-95CA-C5AB0DC85B11';

export default class WebSocketServer {
    // Event handlers for WebSocket events (e.g., 'connection', 'message')
    private events: Record<string, (connection: ConnectionType) => void> = {};

    // Reference to the underlying HTTP server
    private httpServer: Server;

    // Set of active WebSocket connections
    public readonly connections: Set<ConnectionType> = new Set();

    private options: Options;

    constructor(httpServer: Server, options: Options = DefaultOptions) {
        // Validate the provided HTTP server
        if (!httpServer || typeof httpServer !== 'object') {
            throw new Error('Invalid HTTP server');
        }

        // Validate the provided options
        this.options = { ...DefaultOptions, ...options };

        // Warm up the WebAssembly module to ensure optimal performance
        if (this.options.do_warmup) warmup(this.options.warmup_runs, this.options.warmup_data_size);

        // Store the HTTP server reference
        this.httpServer = httpServer;

        // Listen for the 'upgrade' event to handle WebSocket connections
        this.httpServer.on('upgrade', (request, socket, head) => {
            this.handleUpgrade(request, socket, head);
        });
    }

    /**
     * Handles the HTTP upgrade request to establish a WebSocket connection.
     *
     * @param request - The HTTP request object.
     * @param socket - The underlying socket for the connection.
     * @param head - The first packet of the upgraded stream.
     */
    private handleUpgrade(request: any, socket: Duplex, head: Buffer) {
        // Validate the WebSocket handshake key
        const key = request.headers['sec-websocket-key'];
        if (!key) {
            socket.end('HTTP/1.1 400 Bad Request\r\n\r\n');
            return;
        }

        // Generate the WebSocket accept key for the handshake response
        const acceptKey = this.generateAcceptKey(key);
        const responseHeaders = [
            'HTTP/1.1 101 Switching Protocols',
            'Upgrade: websocket',
            'Connection: Upgrade',
            `Sec-WebSocket-Accept: ${acceptKey}`,
            '\r\n',
        ].join('\r\n');

        // Send the handshake response
        socket.write(responseHeaders);

        // Create a new WebSocket connection
        const connection = new Connection(socket, this);
        this.connections.add(connection);

        // Emit the 'connection' event if a handler is registered
        const connectionHandler = this.events['connection'];
        if (connectionHandler) {
            connectionHandler(connection);
        }

        // Handle incoming WebSocket frames
        socket.on('data', (data) => {
            this.handleWebSocketFrame(connection, data);
        });

        // Handle connection close
        socket.on('close', () => {
            this.connections.delete(connection);
        });
    }

    /**
     * Generates the WebSocket accept key for the handshake response.
     *
     * @param key - The WebSocket handshake key from the client.
     * @returns The accept key encoded in base64.
     */
    private generateAcceptKey(key: string): string {
        return createHash('sha1')
            .update(key + RFC_6455) // Concatenate key with RFC 6455 GUID
            .digest('base64'); // Generate base64-encoded hash
    }

    /**
     * Handles incoming WebSocket frames.
     *
     * @param connection - The WebSocket connection.
     * @param data - The raw WebSocket frame data.
     */
    private async handleWebSocketFrame(connection: ConnectionType, data: Buffer) {
        // Measure the time taken to process the WebSocket frame
        console.time('handleWebSocketFrame');

        // Decode the WebSocket frame using the WebAssembly module
        const { opcode, payload } = handle_websocket_frame(data);

        console.timeEnd('handleWebSocketFrame');

        // Handle close frames (opcode 0x8)
        if (opcode === 0x8) {
            connection.socket.end(); // Close the connection
            this.connections.delete(connection); // Remove the connection from the set
            return;
        }

        // Handle text frames (opcode 0x1)
        if (opcode === 0x1) {
            const message = Buffer.from(payload).toString('utf-8'); // Convert payload to string
            const messageHandler = connection.events['message'];
            if (messageHandler) {
                messageHandler(message); // Emit the 'message' event
            }
        }
    }

    /**
     * Registers an event handler for WebSocket events.
     *
     * @param event - The event name (e.g., 'connection', 'message').
     * @param callback - The event handler function.
     */
    on(event: string, callback: (connection: ConnectionType) => void): void {
        this.events[event] = callback;
    }
}