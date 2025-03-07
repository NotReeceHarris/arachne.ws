/*
 * Warming up the WebAssembly (Wasm) module is essential to ensure consistent and optimal performance.
 * The first few runs of a Wasm function can be slower due to:
 * 
 * 1. Cold Start Overhead: Initial setup of the Wasm runtime, memory, and resources.
 * 2. JIT Compilation: JavaScript engines (like V8) use Just-In-Time (JIT) compilation to optimize code.
 *    The first few runs allow the engine to identify "hot" code paths and apply optimizations.
 * 3. Memory Initialization: WebAssembly memory is initialized lazily, which can add overhead during the first few runs.
 * 4. Garbage Collection: If the function interacts with JavaScript objects (e.g., Uint8Array), the first few runs may trigger garbage collection.
 *
 * By performing warm-up runs, we allow the system to stabilize, ensuring that subsequent executions are fast and consistent.
 * This is especially important for benchmarking or production environments where performance is critical.
 */

import { handle_websocket_frame } from "handle_websocket_frame";
import { randomBytes } from 'crypto';

export default function(warmUpRuns: number = 150, warmUpDataSize: number = 1 * 1024 * 1024) {

    console.log('Warming up the wasm...');
    const data = randomBytes(warmUpDataSize);

    try {
        for (let i = 0; i < warmUpRuns; i++) {
            handle_websocket_frame(data);
        }
        console.log('Wasm warmed up successfully!');
    } catch (error) {
        console.error('Error warming up the Wasm:', error);
    }
    
}