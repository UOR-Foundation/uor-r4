/* tslint:disable */
/* eslint-disable */

/**
 * Allocation events since module init (loads only).
 */
export function allocEvents(): bigint;

/**
 * Module init. Registers the host clock seam and the panic hook so a panic
 * surfaces as a JS exception with a message, not a silent trap.
 */
export function init(): void;

/**
 * Models leaked for the module lifetime (one per `loadStack`).
 */
export function leakedModels(): bigint;

/**
 * Load a `UORLUT01` stack artifact from bytes. Returns the session handle.
 */
export function loadStack(artifact: Uint8Array): number;

/**
 * Restore a serialized session into the same loaded model.
 */
export function restoreSession(handle: number, state: Uint8Array): void;

/**
 * Serialize the session state.
 */
export function saveSession(handle: number): Uint8Array;

/**
 * Feed one token; returns `[next_token, margin_q16, position]`.
 */
export function step(handle: number, token: number): Int32Array;

/**
 * Push a monotonic host clock reading (nanoseconds). The JS glue calls this
 * from `performance.now()` at the start of every step batch.
 */
export function tickClock(ns: bigint): void;

/**
 * Weights touched per token (R3 per-token parameter reads).
 */
export function weightsPerToken(handle: number): bigint;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly loadStack: (a: number, b: number) => [number, number, number];
    readonly restoreSession: (a: number, b: number, c: number) => [number, number];
    readonly saveSession: (a: number) => [number, number, number, number];
    readonly step: (a: number, b: number) => [number, number, number, number];
    readonly weightsPerToken: (a: number) => [bigint, number, number];
    readonly allocEvents: () => bigint;
    readonly leakedModels: () => bigint;
    readonly tickClock: (a: bigint) => void;
    readonly init: () => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
