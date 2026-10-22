#!/usr/bin/env node
// Automated verification of the Studio Web Worker (`uor_model_worker.js`) and
// `web/studio/index.html` against the `crates/uor-r4-integer` (`serving-wasm`)
// browser engine and the reference `docs/wasm-serving/verify.mjs` oracle.

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

const wasmPath = process.env.UOR_WASM_PATH
    || resolve(__dirname, 'pkg/uor_r4_integer_bg.wasm');
const stackPath = process.env.UOR_STACK_PATH
    || resolve(__dirname, 'pkg/synthetic-stack.lut');

const wasmBytes = readFileSync(wasmPath);
const stackBytes = readFileSync(stackPath);

// 1. Intercept all network APIs to enforce zero remote inference calls.
const remoteRequests = [];
globalThis.fetch = async (url) => {
    const urlStr = String(url);
    remoteRequests.push(urlStr);
    throw new Error(`Forbidden remote fetch in Studio worker: ${urlStr}`);
};
globalThis.XMLHttpRequest = class {
    open(_method, url) {
        remoteRequests.push(String(url));
        throw new Error(`Forbidden XMLHttpRequest in Studio worker: ${url}`);
    }
};
globalThis.WebSocket = class {
    constructor(url) {
        remoteRequests.push(String(url));
        throw new Error(`Forbidden WebSocket in Studio worker: ${url}`);
    }
};

// Provide preloaded WASM bytes for Node execution without network fetch.
globalThis.__UOR_WASM_BYTES__ = wasmBytes;

// Minimal Web Worker `self` harness for Node ESM execution.
const emittedMessages = [];
let messageWaiters = [];

globalThis.self = {
    postMessage(msg) {
        emittedMessages.push(msg);
        const remaining = [];
        for (const w of messageWaiters) {
            if (w.predicate(msg)) {
                w.resolve(msg);
            } else {
                remaining.push(w);
            }
        }
        messageWaiters = remaining;
    },
    onmessage: null,
};

function waitForMessage(predicate, timeoutMs = 5000) {
    for (const msg of emittedMessages) {
        if (predicate(msg)) {
            const idx = emittedMessages.indexOf(msg);
            if (idx !== -1) emittedMessages.splice(idx, 1);
            return Promise.resolve(msg);
        }
    }
    return new Promise((resolvePromise, rejectPromise) => {
        const timer = setTimeout(() => {
            rejectPromise(new Error(`Timed out waiting for worker message; seen: ${JSON.stringify(emittedMessages)}`));
        }, timeoutMs);
        messageWaiters.push({
            predicate,
            resolve: (msg) => {
                clearTimeout(timer);
                resolvePromise(msg);
            },
        });
    });
}

async function sendToWorker(data) {
    if (typeof globalThis.self.onmessage !== 'function') {
        throw new Error('uor_model_worker.js did not attach self.onmessage');
    }
    await globalThis.self.onmessage({ data });
}

function assert(cond, msg) {
    if (!cond) {
        console.error('FAIL:', msg);
        process.exit(1);
    }
}

// Import the Studio worker module and trigger `prewarm`.
await import('./assets/js/uor_model_worker.js');
await sendToWorker({ action: 'prewarm' });

const runtimeMsg = await waitForMessage((m) => m.action === 'runtime_loaded' || m.action === 'operation_error');
assert(runtimeMsg.action === 'runtime_loaded', `prewarm failed: ${JSON.stringify(runtimeMsg)}`);
assert(runtimeMsg.engine === 'uor-r4-integer/serving-wasm', `unexpected engine tag: ${runtimeMsg.engine}`);

// 2. Load the synthetic stack via `load_model` -> `loadStack`.
await sendToWorker({
    action: 'load_model',
    payload: {
        bytes: stackBytes.buffer.slice(stackBytes.byteOffset, stackBytes.byteOffset + stackBytes.byteLength),
        fileName: 'synthetic-stack.lut',
    },
});

const loadedMsg = await waitForMessage((m) => m.action === 'model_loaded' || m.action === 'operation_error');
assert(loadedMsg.action === 'model_loaded', `load_model failed: ${JSON.stringify(loadedMsg)}`);
assert(loadedMsg.capabilities.schema === 'uor-r4.lut-stack/1', `schema mismatch: ${loadedMsg.capabilities.schema}`);
assert(loadedMsg.capabilities.synthetic === true, 'expected synthetic === true for mk-synthetic-stack artifact');
assert(loadedMsg.capabilities.status_label === 'synthetic stand-in', `expected "synthetic stand-in", got "${loadedMsg.capabilities.status_label}"`);
assert(loadedMsg.capabilities.weights_per_token === 15968, `expected weights_per_token=15968, got ${loadedMsg.capabilities.weights_per_token}`);
assert(loadedMsg.capabilities.shape.vocab === 40 && loadedMsg.capabilities.shape.width === 32, 'shape mismatch');

// 3. Run 32-token generation from PROMPT = [0] and compare with docs/wasm-serving/verify.mjs.
const PROMPT = [0];
const EXPECTED_32 = [
    38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30,
    30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
];

await sendToWorker({
    action: 'generate',
    id: 'gen_full_32',
    payload: {
        conversationId: 'conv_verify_full',
        tokenIds: PROMPT,
        options: { max_new_tokens: 32 },
    },
});

const genFull = await waitForMessage((m) => (m.action === 'generate_complete' || m.action === 'generate_error') && m.id === 'gen_full_32');
assert(genFull.action === 'generate_complete', `generate failed: ${JSON.stringify(genFull)}`);
assert(JSON.stringify(genFull.tokenIds) === JSON.stringify(EXPECTED_32), `32-token output mismatch: got ${JSON.stringify(genFull.tokenIds)}, expected ${JSON.stringify(EXPECTED_32)}`);
assert(genFull.tokenCount === 32, `expected tokenCount=32, got ${genFull.tokenCount}`);
assert(genFull.allocDelta === 0, `expected zero steady-state allocations during step loop, got ${genFull.allocDelta}`);

await sendToWorker({
    action: 'export_session',
    payload: { conversationId: 'conv_verify_full' },
});
const exportedFull32 = await waitForMessage((m) => m.action === 'session_exported' && m.conversationId === 'conv_verify_full');
assert(exportedFull32.bytes && exportedFull32.bytes.byteLength === 9782, `expected 9782 checkpoint bytes after 32 steps, got ${exportedFull32.bytes?.byteLength}`);

// 4. Mid-stream saveSession / restoreSession round-trip (16 + 16 tokens).
await sendToWorker({
    action: 'generate',
    id: 'gen_first_16',
    payload: {
        conversationId: 'conv_split_a',
        tokenIds: PROMPT,
        options: { max_new_tokens: 16 },
    },
});
const genFirst16 = await waitForMessage((m) => m.action === 'generate_complete' && m.id === 'gen_first_16');
assert(genFirst16.tokenIds.length === 16, 'first half length != 16');

await sendToWorker({
    action: 'export_session',
    payload: { conversationId: 'conv_split_a' },
});
const exportedMsg = await waitForMessage((m) => m.action === 'session_exported' && m.conversationId === 'conv_split_a');
assert(exportedMsg.bytes && exportedMsg.bytes.byteLength === 5366, `expected 5366 checkpoint bytes after 16 steps, got ${exportedMsg.bytes?.byteLength}`);

await sendToWorker({
    action: 'import_session',
    payload: {
        conversationId: 'conv_split_b',
        bytes: exportedMsg.bytes,
        lastToken: exportedMsg.lastToken,
    },
});
await waitForMessage((m) => m.action === 'session_imported' && m.conversationId === 'conv_split_b');

await sendToWorker({
    action: 'generate',
    id: 'gen_second_16',
    payload: {
        conversationId: 'conv_split_b',
        tokenIds: [genFirst16.tokenIds[15]],
        options: { max_new_tokens: 16 },
    },
});
const genSecond16 = await waitForMessage((m) => m.action === 'generate_complete' && m.id === 'gen_second_16');
const combined32 = [...genFirst16.tokenIds, ...genSecond16.tokenIds];
assert(JSON.stringify(combined32) === JSON.stringify(EXPECTED_32), `save/restore continuation mismatch: ${JSON.stringify(combined32)}`);

// 5. Verify Studio index.html static invariants (D36 banner + zero remote inference/search fetches).
const indexHtml = readFileSync(resolve(__dirname, 'index.html'), 'utf8');
assert(indexHtml.includes('id="d36EngineStatusBanner"'), 'missing d36EngineStatusBanner in index.html');
assert(indexHtml.includes('0/8') && indexHtml.includes('35.1%'), 'd36EngineStatusBanner must state 0/8 milestones and 35.1% tok/s');
assert(!indexHtml.includes('en.wikipedia.org/w/api.php'), 'live Wikipedia search fetch must be removed from index.html');
assert(!indexHtml.includes('api.duckduckgo.com'), 'live DuckDuckGo search fetch must be removed from index.html');
assert(remoteRequests.length === 0, `expected 0 remote requests, got ${JSON.stringify(remoteRequests)}`);

console.log(JSON.stringify({
    verdict: 'ALL STUDIO WORKER CHECKS PASSED',
    engine: runtimeMsg.engine,
    statusLabel: loadedMsg.capabilities.status_label,
    weightsPerToken: loadedMsg.capabilities.weights_per_token,
    tokens32: genFull.tokenIds,
    checkpointBytes32: exportedFull32.bytes.byteLength,
    checkpointBytes16: exportedMsg.bytes.byteLength,
    saveRestoreBitIdentical: true,
    remoteRequests: remoteRequests.length,
}));
