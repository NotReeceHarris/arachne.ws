/**
 * Configuration options for initializing the WebSocket server.
 */
export interface Options {

    /**
     * The verbosity level for logging messages. 
     * 
     *  - 'debug': Log all (there are alot of logs).
     * - 'info': Log informational messages, warnings, and errors.
     * - 'warn': Log warnings and errors.
     * - 'error': Log only errors.
     * - 'silent': Disable all logging.
     * - 'trace': Log all messages with stack traces.
     * 
     * You can also specify a comma-separated list of levels to log, e.g., 'info,error'.
     * 
     * @default 'info,error'
     */
    verbose?: 'debug' | 'info' | 'warn' | 'error' | 'silent' | 'trace' | string;

    /**
     * Whether to enable performance benchmarks for the WebSocket server.
     * Enabling benchmarks logs additional performance metrics for each WebSocket frame.
     * 
     * @default false
     */
    benchmarks?: boolean;

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
    verbose: 'info',
    benchmarks: false
};