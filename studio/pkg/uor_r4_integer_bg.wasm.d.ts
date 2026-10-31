/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const clearReplyStart: (a: number) => [number, number];
export const discreteInfo: (a: number) => [number, number, number, number];
export const loadDiscrete: (a: number, b: number) => [number, number, number];
export const loadStack: (a: number, b: number) => [number, number, number];
export const markReplyStart: (a: number) => [number, number];
export const memoryInfo: (a: number) => [number, number, number, number];
export const predictDiscrete: (a: number, b: number, c: number) => [number, number, number, number];
export const restoreSession: (a: number, b: number, c: number) => [number, number];
export const saveSession: (a: number) => [number, number, number, number];
export const step: (a: number, b: number) => [number, number, number, number];
export const weightsPerToken: (a: number) => [bigint, number, number];
export const allocEvents: () => bigint;
export const leakedModels: () => bigint;
export const tickClock: (a: bigint) => void;
export const init: () => void;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_start: () => void;
