import type { Server } from 'http';
import { createHash } from 'crypto';
import Connection from './connection';

import type { Duplex } from 'stream';
import type { ConnectionType } from './connection';

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
            console.time('handleWebSocketFrame');
            this.handleWebSocketFrame(connection, data);
            console.timeEnd('handleWebSocketFrame');
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

    private handleWebSocketFrame(connection: ConnectionType, data: Buffer) {
        // Parse WebSocket frame
        const opcode = data[0] & 0x0f; // First byte contains the opcode
        const isMasked = (data[1] & 0x80) !== 0; // Second byte contains the MASK bit
        let payloadLength = data[1] & 0x7f; // Second byte contains the payload length
    
        let offset = 2; // Start reading after the first two bytes
    
        // Handle extended payload length
        if (payloadLength === 126) {
            payloadLength = data.readUInt16BE(offset);
            offset += 2;
        } else if (payloadLength === 127) {
            // Note: JavaScript cannot handle 64-bit integers, so we assume the payload length is within 32 bits
            payloadLength = data.readUInt32BE(offset + 4);
            offset += 8;
        }
    
        // Read the masking key (if present)
        let maskingKey: Uint8Array | null = null;
        if (isMasked) {
            maskingKey = new Uint8Array(data.buffer, offset, 4);
            offset += 4;
        }
    
        // Read the payload data
        const payload = new Uint8Array(data.buffer, offset, payloadLength);
    
        // Unmask the payload (if masked)
        if (isMasked && maskingKey) {
            this.unmaskPayload(payload, maskingKey);
        }
    
        // Handle the frame based on the opcode
        if (opcode === 0x8) {
            // Close frame
            console.log('Received close frame');
            connection.socket.end();
            this.connections.delete(connection);
            return;
        }
    
        if (opcode === 0x1) {
            // Text frame
            const message = Buffer.from(payload).toString('utf-8');
            console.log('Received message:', message);

            const event = connection.events['message'];
            if (event) event(message);
        }
    }
    
    private unmaskPayload(payload: Uint8Array, maskingKey: Uint8Array) {
        // Fast unmasking using loop unrolling
        const len = payload.length;
        const key = maskingKey;
        let i = 0;
    
        // Process 4 bytes at a time
        for (; i + 3 < len; i += 4) {
            payload[i] ^= key[0];
            payload[i + 1] ^= key[1];
            payload[i + 2] ^= key[2];
            payload[i + 3] ^= key[3];
        }
    
        // Process remaining bytes
        for (; i < len; i++) {
            payload[i] ^= key[i % 4];
        }
    }

    on(event: string, callback: (connection: ConnectionType) => void): void {
        this.events[event] = callback;
    }
}