import type { Duplex } from 'stream';

function generateId(): string {
    const randomPart = (Math.random() * 0xffffffff).toString(16).padStart(8, '0');
    const timePart = Date.now().toString(16).padStart(8, '0');
    return randomPart.replace('.','') + timePart;
}

export default class Connection {
    private server: any;

    public readonly id: string;
    public readonly socket: Duplex;
    public readonly events: { [key: string]: (message: string) => void } = {};

    constructor(socket: Duplex, server: any) {
        this.id = generateId();
        this.socket = socket;
        this.server = server;
    }

    on(event: string, callback: (message: string) => void): void {
        this.events[event] = callback;
    }

    send(message: string) {

        const payload = Buffer.from(message, 'utf-8');
        const payloadLength = payload.length;

        // Use a pre-allocated buffer for small messages (<= 125 bytes)
        if (payloadLength <= 125) {
            const frame = Buffer.allocUnsafe(2 + payloadLength); // Unsafe to avoid zero-filling
            frame[0] = 0x81; // FIN bit set + opcode for text frame
            frame[1] = payloadLength; // No masking, so payload length is just the length
            payload.copy(frame, 2); // Copy payload into the frame
            this.socket.write(frame);
            return;
        }

        // Use a pre-allocated buffer for medium messages (126 to 65535 bytes)
        if (payloadLength <= 65535) {
            const frame = Buffer.allocUnsafe(4 + payloadLength); // Unsafe to avoid zero-filling
            frame[0] = 0x81; // FIN bit set + opcode for text frame
            frame[1] = 126; // Extended payload length (16-bit)
            frame.writeUInt16BE(payloadLength, 2); // Write payload length
            payload.copy(frame, 4); // Copy payload into the frame
            this.socket.write(frame);
            return;
        }

        // Handle large messages (> 65535 bytes)
        const frame = Buffer.allocUnsafe(10 + payloadLength); // Unsafe to avoid zero-filling
        frame[0] = 0x81; // FIN bit set + opcode for text frame
        frame[1] = 127; // Extended payload length (64-bit)
        frame.writeUInt32BE(0, 2); // High 32 bits of payload length (assume 0)
        frame.writeUInt32BE(payloadLength, 6); // Low 32 bits of payload length
        payload.copy(frame, 10); // Copy payload into the frame
        this.socket.write(frame);
    }
    

    broadcast(message: string, include_self = false) {
        for (const connection of this.server.connections) {
            if (!include_self && connection === this) continue;
            connection.send(message);
        }
    }
}

export type ConnectionType = InstanceType<typeof Connection>;