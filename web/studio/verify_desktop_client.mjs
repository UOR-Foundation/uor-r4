#!/usr/bin/env node
// Automated offline & native-bridge verification for the Tauri desktop Studio
// (`src-tauri/` + `web/studio/index.html` + `web/studio/assets/js/uor_offline_vendor.js`).

import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, '../..');

function assert(cond, msg) {
    if (!cond) {
        console.error('FAIL:', msg);
        process.exit(1);
    }
}

// 1. Verify offline static invariants on web/studio/index.html and vendor assets.
const indexHtml = readFileSync(resolve(__dirname, 'index.html'), 'utf8');
const vendorJs = readFileSync(resolve(__dirname, 'assets/js/uor_offline_vendor.js'), 'utf8');
const fontsCss = readFileSync(resolve(__dirname, 'assets/css/uor_offline_fonts.css'), 'utf8');
const tauriConf = JSON.parse(readFileSync(resolve(repoRoot, 'src-tauri/tauri.conf.json'), 'utf8'));

for (const [name, content] of [
    ['index.html', indexHtml],
    ['uor_offline_vendor.js', vendorJs],
    ['uor_offline_fonts.css', fontsCss],
]) {
    assert(!content.includes('http://') && !content.includes('https://'), `${name} contains external http(s) URL`);
}

assert(indexHtml.includes('./assets/css/uor_offline_fonts.css'), 'index.html must link local uor_offline_fonts.css');
assert(indexHtml.includes('./assets/js/uor_offline_vendor.js'), 'index.html must load local uor_offline_vendor.js');
assert(indexHtml.includes('createTauriNativeModelWorker'), 'index.html must wire createTauriNativeModelWorker in Tauri mode');
assert(indexHtml.includes("invoke('native_grant_folder'"), 'index.html must grant folder before git/file access');
assert(indexHtml.includes("invoke('native_list_directory'"), 'index.html must list workspace files via native_list_directory');
assert(
    indexHtml.includes('userConfirmed: true') && indexHtml.includes('confirmedCommand: cmd'),
    'index.html must pass userConfirmed and confirmedCommand to native_run_terminal_command',
);
assert(
    indexHtml.includes("action: 'save_session_durable'"),
    'index.html must persist conversations via save_session_durable in Tauri mode',
);

// Verify CSP in src-tauri/tauri.conf.json blocks remote connections.
const csp = tauriConf?.app?.security?.csp || '';
assert(csp.includes("connect-src 'self' ipc: http://ipc.localhost"), 'tauri.conf.json CSP must restrict connect-src to local IPC');
assert(!csp.includes('https://'), 'tauri.conf.json CSP must not allow external https origins');

// 2. Trap all network APIs and evaluate uor_offline_vendor.js in an isolated offline global scope.
const remoteRequests = [];
function trap(api) {
    return (...args) => {
        remoteRequests.push(`${api}:${String(args[0] || '')}`);
        throw new Error(`Forbidden network call in offline desktop Studio: ${api}`);
    };
}
globalThis.window = globalThis;
globalThis.fetch = trap('fetch');
globalThis.XMLHttpRequest = class { open(_m, u) { trap('XMLHttpRequest')(u); } };
globalThis.WebSocket = class { constructor(u) { trap('WebSocket')(u); } };
globalThis.EventSource = class { constructor(u) { trap('EventSource')(u); } };

await import('./assets/js/uor_offline_vendor.js');

assert(typeof globalThis.marked?.parse === 'function', 'offline marked.parse missing');
assert(typeof globalThis.katex?.renderToString === 'function', 'offline katex.renderToString missing');
assert(typeof globalThis.renderMathInElement === 'function', 'offline renderMathInElement missing');
assert(typeof globalThis.monaco?.editor?.create === 'function', 'offline monaco.editor.create missing');
assert(typeof globalThis.require === 'function' && typeof globalThis.require.config === 'function', 'offline AMD require shim missing');
assert(typeof globalThis.createTauriNativeModelWorker === 'function', 'createTauriNativeModelWorker missing');

// 3. Execute the native desktop client self-test binary if UOR_DESKTOP_BIN is provided,
// and verify the Tauri Native Model Worker bridge against the reference 32-token trajectory.
const EXPECTED_32 = [
    38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30,
    30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
];

let selfTestReceipt = null;
if (process.env.UOR_DESKTOP_BIN) {
    const stdout = execFileSync(process.env.UOR_DESKTOP_BIN, ['--self-test'], { encoding: 'utf8' }).trim();
    selfTestReceipt = JSON.parse(stdout);
    assert(selfTestReceipt.verdict === 'PASS', `desktop --self-test failed: ${stdout}`);
    assert(selfTestReceipt.engine === 'uor-r4-integer/native-desktop', `unexpected engine: ${selfTestReceipt.engine}`);
    assert(selfTestReceipt.artifactBytes === 469252, `unexpected artifactBytes: ${selfTestReceipt.artifactBytes}`);
    assert(
        selfTestReceipt.artifactSha256 === 'd60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df',
        `unexpected artifactSha256: ${selfTestReceipt.artifactSha256}`,
    );
    assert(selfTestReceipt.weightsPerToken === 15968, `unexpected weightsPerToken: ${selfTestReceipt.weightsPerToken}`);
    assert(JSON.stringify(selfTestReceipt.tokens32) === JSON.stringify(EXPECTED_32), 'self-test 32-token trajectory mismatch');
    assert(selfTestReceipt.ur4sess1Bytes === 9782, `expected ur4sess1Bytes=9782, got ${selfTestReceipt.ur4sess1Bytes}`);
    assert(selfTestReceipt.storeRecords === 446, `expected storeRecords=446, got ${selfTestReceipt.storeRecords}`);
    assert(
        selfTestReceipt.objectVersion === 'bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca',
        `unexpected objectVersion: ${selfTestReceipt.objectVersion}`,
    );
}

// 4. Verify createTauriNativeModelWorker IPC message protocol.
const invokedCommands = [];
const mockTauriCore = {
    async invoke(cmd, args = {}) {
        invokedCommands.push(cmd);
        switch (cmd) {
            case 'native_engine_status':
                return {
                    loaded: false,
                    engine: 'uor-r4-integer/native-desktop',
                    capabilities: null,
                    artifact_bytes: 0,
                    artifact_sha256: null,
                    active_sessions: 0,
                };
            case 'native_load_model':
                return {
                    schema: 'uor-r4.lut-stack/1',
                    synthetic: true,
                    status_label: 'synthetic stand-in',
                    tokenizer_label: 'UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)',
                    weights_per_token: 15968,
                    artifact_bytes: 469252,
                    artifact_sha256: 'd60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df',
                    shape: { vocab: 40, width: 32, heads: 4, context: 64 },
                };
            case 'native_step_tokens':
                return {
                    events: EXPECTED_32.slice(0, args.maxNewTokens || 32).map((tok, i) => ({
                        step_index: i,
                        position: i + 1,
                        token_id: tok,
                        margin_q16: 0,
                        chunk: String.fromCharCode(65 + (tok % 26)),
                        full_text: '[stream]',
                    })),
                    receipt: {
                        conversation_id: args.conversationId,
                        token_ids: EXPECTED_32.slice(0, args.maxNewTokens || 32),
                        full_text: '[token 38][token 9]',
                        weights_per_token: 15968,
                        status_label: 'synthetic stand-in',
                        tokenizer_label: 'UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)',
                    },
                };
            case 'native_save_session_durable':
                return {
                    conversation_id: args.conversationId,
                    scope: `local-user/studio/${args.conversationId}`,
                    object_version: 'bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca',
                    store_records: 446,
                    ur4sess1_bytes: 9782,
                    last_token: 30,
                };
            case 'native_restore_session_durable':
                return {
                    conversation_id: args.conversationId,
                    scope: `local-user/studio/${args.conversationId}`,
                    object_version: 'bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca',
                    store_records: 446,
                    ur4sess1_bytes: 9782,
                    last_token: 30,
                };
            default:
                return null;
        }
    },
};

const workerEvents = [];
const worker = globalThis.createTauriNativeModelWorker(mockTauriCore);
worker.onmessage = (e) => workerEvents.push(e.data);

worker.postMessage({ action: 'prewarm' });
await new Promise((r) => setTimeout(r, 20));
assert(workerEvents.some((e) => e.action === 'runtime_loaded' && e.engine === 'uor-r4-integer/native-desktop'), 'prewarm did not emit native-desktop runtime_loaded');

worker.postMessage({ action: 'load_local_model', payload: { path: '/tmp/synthetic-stack.lut' } });
await new Promise((r) => setTimeout(r, 20));
assert(workerEvents.some((e) => e.action === 'model_loaded' && e.bytes === 469252), 'load_local_model did not emit model_loaded');

worker.postMessage({
    action: 'generate',
    id: 'gen_desktop_1',
    payload: { conversationId: 'conv_desktop_1', tokenIds: [0], options: { max_new_tokens: 32 } },
});
await new Promise((r) => setTimeout(r, 30));
const genDone = workerEvents.find((e) => e.action === 'generate_complete' && e.id === 'gen_desktop_1');
assert(genDone && JSON.stringify(genDone.tokenIds) === JSON.stringify(EXPECTED_32), 'desktop worker generate_complete tokens mismatch');

worker.postMessage({
    action: 'save_session_durable',
    payload: { conversationId: 'conv_desktop_1', messagesJson: '[{"role":"user","content":"hi"}]' },
});
await new Promise((r) => setTimeout(r, 20));
const durableSaved = workerEvents.find((e) => e.action === 'session_durable_saved');
assert(
    durableSaved && durableSaved.receipt.object_version === 'bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca',
    'save_session_durable did not emit expected object_version receipt',
);
assert(remoteRequests.length === 0, `expected 0 remote requests, got ${JSON.stringify(remoteRequests)}`);

console.log(JSON.stringify({
    verdict: 'ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED',
    offlineExternalUrls: 0,
    invokedCommands,
    selfTestReceipt,
    remoteRequests: remoteRequests.length,
}));
