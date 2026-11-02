import test from 'node:test';
import assert from 'node:assert/strict';

function artifactBytes() {
    const header = new TextEncoder().encode(JSON.stringify({ schema: 'uor-r4.lut-stack/1', shape: { vocab: 17, context: 64 } }));
    const bytes = new Uint8Array(16 + header.byteLength);
    bytes.set(new TextEncoder().encode('UORLUT01'), 0);
    new DataView(bytes.buffer).setBigUint64(8, BigInt(header.byteLength), true);
    bytes.set(header, 16);
    return bytes;
}

function response(bytes, total = bytes.byteLength) {
    const headers = total === null ? {} : { 'content-length': String(total) };
    return new Response(bytes, { headers });
}

function deferred() {
    let resolve;
    const promise = new Promise((resolvePromise) => { resolve = resolvePromise; });
    return { promise, resolve };
}

let moduleCounter = 0;
async function fresh(fetchImpl) {
    const listeners = new Map();
    const events = [];
    const values = new Map();
    globalThis.localStorage = {
        getItem(key) { return values.has(key) ? values.get(key) : null; },
        setItem(key, value) { values.set(key, value); },
        removeItem(key) { values.delete(key); },
    };
    globalThis.document = {
        baseURI: 'https://example.test/',
        addEventListener(type, listener) {
            if (!listeners.has(type)) listeners.set(type, []);
            listeners.get(type).push(listener);
        },
        dispatchEvent(event) { events.push(event); return true; },
    };
    globalThis.fetch = fetchImpl;
    globalThis.CustomEvent = class {
        constructor(type, init = {}) { this.type = type; this.detail = init.detail; }
    };
    moduleCounter += 1;
    const module = await import(`./model_sources.js?test=${moduleCounter}`);
    return { api: module.ModelSources, events, LOAD_PROGRESS_EVENT: module.LOAD_PROGRESS_EVENT };
}

test('loadFromUrl emits real byte progress and one finalization', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => response(bytes));
    const result = await api.loadFromUrl('https://example.com/model.lut', { operationId: 1, sourceId: 'url' });
    assert.equal(result.stale, undefined);
    const progress = events.filter((event) => event.type === LOAD_PROGRESS_EVENT);
    assert.ok(progress.some((event) => event.detail.phase === 'downloading' && event.detail.loaded === bytes.byteLength && event.detail.total === bytes.byteLength));
    assert.equal(progress.at(-1).detail.phase, 'finalizing');
    assert.ok(events.some((event) => event.type === 'uor-model-source-changed'));
});

test('unknown totals never acquire a percentage', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => response(bytes, null));
    await api.loadFromUrl('https://example.com/model.lut', { operationId: 1, sourceId: 'url' });
    const progress = events.filter((event) => event.type === LOAD_PROGRESS_EVENT);
    assert.ok(progress.length > 0);
    for (const event of progress) {
        assert.equal(event.detail.total, null);
        assert.equal('percent' in event.detail, false);
    }
});

test('a stale completion cannot select or emit a source change', async () => {
    const bytes = artifactBytes();
    const first = deferred();
    const second = deferred();
    const responses = [first.promise, second.promise];
    const { api, events } = await fresh(async () => responses.shift());
    const firstLoad = api.loadFromUrl('https://example.com/a.lut', { operationId: 1, sourceId: 'a' });
    const secondLoad = api.loadFromUrl('https://example.com/b.lut', { operationId: 2, sourceId: 'b' });
    second.resolve(response(bytes));
    const secondResult = await secondLoad;
    first.resolve(response(bytes));
    const firstResult = await firstLoad;
    assert.equal(secondResult.stale, undefined);
    assert.equal(firstResult.stale, true);
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed').length, 1);
});

test('an explicit API selection supersedes an in-flight metadata load', async () => {
    const bytes = artifactBytes();
    const gate = deferred();
    const { api, events } = await fresh(() => gate.promise);
    const active = api.list().api[0];
    const load = api.loadFromUrl('https://example.com/a.lut', { operationId: 1, sourceId: 'url' });
    api.setActive({ kind: 'api', id: active.id });
    gate.resolve(response(bytes));
    const result = await load;
    assert.equal(result.stale, true);
    assert.deepEqual(api.active(), { kind: 'api', id: active.id });
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed' && event.detail.kind === 'wasm').length, 0);
});

test('legacy calls without metadata retain their behavior', async () => {
    const bytes = artifactBytes();
    const { api, events } = await fresh(async () => response(bytes));
    const result = await api.loadFromUrl('https://example.com/model.lut');
    assert.equal(result.stale, undefined);
    assert.equal(api.active().kind, 'wasm');
    assert.equal(api.active().id, 'url');
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed').length, 1);
});

test('published loads forward operation metadata', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => response(bytes));
    const entry = { id: 'published-a', label: 'Published A', url: 'https://example.com/published.lut' };
    const result = await api.loadPublished(entry, { operationId: 4, sourceId: 'published-a' });
    assert.equal(result.stale, undefined);
    assert.ok(events.some((event) => event.type === LOAD_PROGRESS_EVENT && event.detail.operationId === 4));
});

test('local files finalize without fabricated byte stages', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => { throw new Error('network must not run'); });
    const file = new File([bytes], 'local.lut');
    const result = await api.loadLocalFile(file, { operationId: 2, sourceId: 'local-file' });
    assert.equal(result.stale, undefined);
    const progress = events.filter((event) => event.type === LOAD_PROGRESS_EVENT);
    assert.equal(progress.length, 1);
    assert.equal(progress[0].detail.phase, 'finalizing');
});

test('bundled loads accept operation metadata and finalize', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => response(bytes));
    const result = await api.loadBundled({ operationId: 3, sourceId: 'bundled-demo' });
    assert.equal(result.stale, undefined);
    const progress = events.filter((event) => event.type === LOAD_PROGRESS_EVENT);
    assert.equal(progress.at(-1).detail.phase, 'finalizing');
    assert.equal(progress.at(-1).detail.operationId, 3);
});
