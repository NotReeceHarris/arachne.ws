/**
 * Configuration options for initializing the WebSocket server.
 */
export interface Options {

    /**
     * The verbosity level for logging messages.
     * 
     * @default 'info'
     */
    verbose?: 'debug' | 'info' | 'warn' | 'error';

    /**
     * Whether to perform a warm-up of the WebAssembly module.
     * Warming up ensures optimal performance by triggering JIT compilation
     * and memory initialization before handling actual WebSocket frames.
     * 
     * @default true
     */
    do_warmup?: boolean;

    /**
     * The number of warm-up runs to perform.
     * A higher number of runs ensures better optimization but increases startup time.
     * 
     * @default 150
     */
    warmup_runs?: number;
    
    /**
     * The size of the data payload (in bytes) to use for warm-up.
     * A larger payload simulates more realistic WebSocket frames but may increase memory usage.
     * 
     * @default 1MB (1 * 1024 * 1024)
     */
    warmup_data_size?: number;
}

export const DefaultOptions: Options = {
    do_warmup: true,
    warmup_runs: 150,
    warmup_data_size: 1 * 1024 * 1024,
    verbose: 'info'
};