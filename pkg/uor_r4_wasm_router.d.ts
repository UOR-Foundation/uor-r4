/* tslint:disable */
/* eslint-disable */

/**
 * The unified router core coordinator.
 */
export class UorR4Router {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Computes live UOR resonance metrics for a given input text
     */
    calculate_resonance(text: string): any;
    /**
     * Compiles a raw string thought parameter down into its content-addressed math state
     */
    compile_thought(content: string): any;
    /**
     * Returns current connection drift
     */
    connection_drift(): number;
    /**
     * Reset the alignment back to native state ($0.00\%$ error) using ZKP 2i Sync-Handshake
     */
    execute_zkp_phase_reset(): string;
    /**
     * Returns the active stream list as a JS Array
     */
    get_active_streams(): any;
    /**
     * Returns the active counts for the 64 experts
     */
    get_expert_counts(): Uint32Array;
    /**
     * Injects a new thought stream, updates MoE activations, and returns the stream
     */
    inject_thought_stream(content: string): any;
    /**
     * Exposes read-only status of manifold alignment
     */
    is_aligned(): boolean;
    /**
     * Returns the kill switch threshold limit
     */
    kill_switch_threshold(): number;
    /**
     * Instantiates the R4 Router with perfect, error-free default states
     */
    constructor(threshold: number);
    /**
     * Progresses the connection drift state using delta-time ($dt$) increments.
     * Returns a log message string if a ZKP reset occurs, otherwise returns undefined.
     */
    update_drift_physics(dt: number, drift_rate: number): string | undefined;
}

export function init_wasm(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_uorr4router_free: (a: number, b: number) => void;
    readonly init_wasm: () => void;
    readonly uorr4router_calculate_resonance: (a: number, b: number, c: number) => any;
    readonly uorr4router_compile_thought: (a: number, b: number, c: number) => any;
    readonly uorr4router_connection_drift: (a: number) => number;
    readonly uorr4router_execute_zkp_phase_reset: (a: number) => [number, number];
    readonly uorr4router_get_active_streams: (a: number) => any;
    readonly uorr4router_get_expert_counts: (a: number) => [number, number];
    readonly uorr4router_inject_thought_stream: (a: number, b: number, c: number) => any;
    readonly uorr4router_is_aligned: (a: number) => number;
    readonly uorr4router_kill_switch_threshold: (a: number) => number;
    readonly uorr4router_new: (a: number) => number;
    readonly uorr4router_update_drift_physics: (a: number, b: number, c: number) => [number, number];
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
