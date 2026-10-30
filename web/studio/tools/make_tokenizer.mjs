#!/usr/bin/env node
// Generate the stack-bound tokenizer artifact (`uor-r4.studio-tokenizer/1`) for the Studio worker.
// Supports the engine's container family `uor-r4.lut-stack/1` .. `/7` (`shape.vocab`) and the
// legacy `uor-r4.lut-discrete/1` header (`shape.vocabulary`; tokenizer-compatible, not served by
// the WASM stack engine). Default mode is `byte-bpe` (with `--source-tokenizer <hf.json>` for
// custom BPE vocabularies); `--mode byte-mod` emits the legacy 256-entry `byte % vocab` table.

import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, resolve } from 'node:path';
import {
    TOKENIZER_SCHEMA,
    BYTE_BPE_MODE,
    BYTE_MOD_MODE,
    createDefaultSynthetic40TokenizerArtifact,
    validateTokenizerArtifact,
} from '../assets/js/uor_tokenizer_artifact.js';

const STACK_MAGIC = 'UORLUT01';
const STACK_SCHEMA_PREFIX = 'uor-r4.lut-stack/';
const STACK_SCHEMA_MIN = 1;
const STACK_SCHEMA_MAX = 7;
const DISCRETE_SCHEMA = 'uor-r4.lut-discrete/1';

/// True for the engine's `uor-r4.lut-stack/1`..`/7` family and the legacy discrete header.
export function supportedStackSchema(schema) {
    if (schema === DISCRETE_SCHEMA) return true;
    if (typeof schema !== 'string' || !schema.startsWith(STACK_SCHEMA_PREFIX)) return false;
    const level = Number(schema.slice(STACK_SCHEMA_PREFIX.length));
    return Number.isInteger(level) && level >= STACK_SCHEMA_MIN && level <= STACK_SCHEMA_MAX;
}

export function parseStackHeader(bytes) {
    const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes || []);
    const magic = new TextDecoder('ascii').decode(u8.subarray(0, 8));
    if (u8.byteLength < 16 || magic !== STACK_MAGIC) {
        throw new Error(`expected a UORLUT01 stack (magic ${STACK_MAGIC}), got "${magic}"`);
    }
    const view = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
    const jsonLen = Number(view.getBigUint64(8, true));
    if (!Number.isSafeInteger(jsonLen) || jsonLen <= 0 || 16 + jsonLen > u8.byteLength) {
        throw new Error(`invalid UORLUT01 header length (${jsonLen} bytes)`);
    }
    const header = JSON.parse(new TextDecoder('utf-8').decode(u8.subarray(16, 16 + jsonLen)));
    const vocab = header?.shape
        ? (Number.isInteger(header.shape.vocab) ? header.shape.vocab : header.shape.vocabulary)
        : null;
    if (!supportedStackSchema(header?.schema) || !Number.isInteger(vocab) || vocab <= 0) {
        throw new Error(`unsupported stack schema/header (${header?.schema}); expected ${STACK_SCHEMA_PREFIX}${STACK_SCHEMA_MIN}..${STACK_SCHEMA_MAX} or ${DISCRETE_SCHEMA}`);
    }
    return { ...header, vocab_size: vocab };
}

export function stackSha256(stackBytes) {
    return createHash('sha256').update(stackBytes).digest('hex');
}

export function buildArtifact(stackBytes, options = {}) {
    const header = parseStackHeader(stackBytes);
    const vocab = header.vocab_size;
    const sha = stackSha256(stackBytes);
    const mode = options.mode || BYTE_BPE_MODE;
    if (mode === BYTE_MOD_MODE) {
        return {
            schema: TOKENIZER_SCHEMA,
            mode: BYTE_MOD_MODE,
            stack_sha256: sha,
            vocab_size: vocab,
            byte_to_id: Array.from({ length: 256 }, (_, byte) => byte % vocab),
            label_note: 'synthetic stand-in: token id = prompt byte mod stack vocab; bound to this stack sha256',
        };
    }
    if (mode !== BYTE_BPE_MODE) {
        throw new Error(`unsupported tokenizer mode "${mode}"`);
    }
    if (options.sourceTokenizer) {
        const src = options.sourceTokenizer;
        const candidate = {
            schema: TOKENIZER_SCHEMA,
            mode: BYTE_BPE_MODE,
            stack_sha256: sha,
            vocab_size: vocab,
            unk_id: Number.isInteger(src.unk_id) ? src.unk_id : 2,
            pre_tokenizer: src.pre_tokenizer || { type: 'ByteLevel', add_prefix_space: false },
            added_tokens: Array.isArray(src.added_tokens) ? src.added_tokens : [],
            model: src.model,
            label_note: 'stack-bound ByteLevel BPE tokenizer artifact',
        };
        if (!validateTokenizerArtifact(candidate, sha, vocab)) {
            throw new Error(`source tokenizer does not validate against stack vocab_size=${vocab}`);
        }
        return candidate;
    }
    return createDefaultSynthetic40TokenizerArtifact(sha, vocab);
}

function usage() {
    console.error('usage: node web/studio/tools/make_tokenizer.mjs <stack.lut> [--out <tokenizer.json>] [--source-tokenizer <hf.json>] [--mode byte-bpe|byte-mod] [--check <tokenizer.json>]');
    process.exit(2);
}

if (import.meta.url === `file://${process.argv[1]}`) {
    const args = process.argv.slice(2);
    if (args.length < 1) usage();
    const stackPath = resolve(args[0]);
    const stackBytes = readFileSync(stackPath);
    const outIdx = args.indexOf('--out');
    const checkIdx = args.indexOf('--check');
    const srcIdx = args.indexOf('--source-tokenizer');
    const modeIdx = args.indexOf('--mode');
    const mode = modeIdx >= 0 ? args[modeIdx + 1] : BYTE_BPE_MODE;
    const sourceTokenizer = srcIdx >= 0 ? JSON.parse(readFileSync(resolve(args[srcIdx + 1]), 'utf8')) : null;
    const artifact = buildArtifact(stackBytes, { mode, sourceTokenizer });
    if (checkIdx >= 0) {
        const existing = JSON.parse(readFileSync(resolve(args[checkIdx + 1]), 'utf8'));
        if (!validateTokenizerArtifact(existing, artifact.stack_sha256, artifact.vocab_size)) {
            console.error('FAIL: tokenizer artifact does not match the shipped stack (hash/vocab/table)');
            process.exit(1);
        }
        console.log(JSON.stringify({ verdict: 'TOKENIZER ARTIFACT MATCHES THE STACK', mode: existing.mode, vocab_size: artifact.vocab_size, stack_sha256: artifact.stack_sha256.slice(0, 12) }));
        process.exit(0);
    }
    const outPath = outIdx >= 0 ? resolve(args[outIdx + 1]) : resolve(dirname(stackPath), 'tokenizer.json');
    writeFileSync(outPath, JSON.stringify(artifact, null, 2) + '\n');
    console.log(JSON.stringify({ verdict: 'WROTE STACK-BOUND TOKENIZER', out: outPath, mode: artifact.mode, vocab_size: artifact.vocab_size, stack_sha256: artifact.stack_sha256.slice(0, 12) }));
}
