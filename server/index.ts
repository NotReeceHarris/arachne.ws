import type { Server } from 'http';
import { createHash } from 'crypto';
import Connection from './connection';

import type { Duplex } from 'stream';
import type { ConnectionType } from './connection';

import { handle_websocket_frame } from "handle_websocket_frame";

// https://www.rfc-editor.org/rfc/rfc6455
const RFC_6455 = '258EAFA5-E914-47DA-95CA-C5AB0DC85B11';

export default class {
    private events = {};
    private httpServer: Server;
    public readonly connections: Set<ConnectionType> = new Set();

    constructor(httpServer: Server) {
        // Validate the http server
        if (!httpServer || typeof httpServer !== 'object') {
            throw new Error('Invalid http server');
        }

        this.httpServer = httpServer;

        // Listen for the 'upgrade' event to handle WebSocket connections
        this.httpServer.on('upgrade', (request, socket, head) => {
            this.handleUpgrade(request, socket, head);
        });
    }

    private handleUpgrade(request: any, socket: Duplex, head: Buffer) {
        // Validate WebSocket headers
        const key = request.headers['sec-websocket-key'];
        if (!key) {
            socket.end('HTTP/1.1 400 Bad Request\r\n\r\n');
            return;
        }

        // Perform the WebSocket handshake
        const acceptKey = this.generateAcceptKey(key);
        const responseHeaders = [
            'HTTP/1.1 101 Switching Protocols',
            'Upgrade: websocket',
            'Connection: Upgrade',
            `Sec-WebSocket-Accept: ${acceptKey}`,
            '\r\n',
        ].join('\r\n');

        socket.write(responseHeaders);

        // Create a new connection
        const connection = new Connection(socket, this);
        this.connections.add(connection);

        // Emit the 'connection' event
        const event = this.events['connection'];
        if (event) {
            event(connection);
        }

        // Handle WebSocket frames
        socket.on('data', (data) => {
            this.handleWebSocketFrame(connection, data);
        });

        // Handle connection close
        socket.on('close', () => {
            this.connections.delete(connection);
        });
    }

    private generateAcceptKey(key: string): string {
        return createHash('sha1')
            .update(key + RFC_6455)
            .digest('base64');
    }

    private async handleWebSocketFrame(connection: ConnectionType, data: Buffer) {

        console.time('handleWebSocketFrame');
        const {opcode, payload} = handle_websocket_frame(data);
        console.timeEnd('handleWebSocketFrame');
    
        // Close frame
        if (opcode === 0x8) {
            connection.socket.end();
            return this.connections.delete(connection);
        }
    
        // Text frame
        if (opcode === 0x1) {
            const message = Buffer.from(payload).toString('utf-8');
            const event = connection.events['message'];
            if (event) event(message);
        }   
    }

    on(event: string, callback: (connection: ConnectionType) => void): void {
        this.events[event] = callback;
    }
}