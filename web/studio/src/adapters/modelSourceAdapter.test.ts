import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { createModelSourceAdapter, LOAD_PROGRESS_EVENT } from './modelSourceAdapter.js';

function sources() {
    const result = { bytes: new Uint8Array(), fileName: 'model.lut', meta: { schema: 'uor-r4.lut-stack/1' } };
    return {
        loadPublished: vi.fn(async () => result),
        loadFromUrl: vi.fn(async () => result),
        loadLocalFile: vi.fn(async () => result),
        loadBundled: vi.fn(async () => result),
        testApi: vi.fn(async () => ({ ok: true, detail: 'ok', models: [] })),
    };
}

function worker() {
    return { postMessage: vi.fn() };
}

describe('model source adapter', () => {
    let store: LifecycleStore;
    beforeEach(() => {
        store = new LifecycleStore();
        document.dispatchEvent(new CustomEvent('uor-test-reset'));
    });

    it('forwards URL loads and waits for worker readiness', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        await adapter.loadFromUrl('https://example.com/model.lut');
        expect(api.loadFromUrl).toHaveBeenCalledWith('https://example.com/model.lut', { operationId: 1, sourceId: 'url' });
        expect(store.getState().artifact.phase).toBe('finalizing');
        expect(store.canRun()).toBe(false);
        adapter.handleWorkerMessage({ action: 'model_loaded', capabilities: { schema: 'uor-r4.lut-stack/1' } });
        expect(store.getState().artifact.phase).toBe('ready');
        expect(store.canRun()).toBe(true);
    });

    it('maps structured byte progress and finalization without inventing percentages', async () => {
        const adapter = createModelSourceAdapter(store, sources(), worker());
        const load = adapter.loadFromUrl('https://example.com/model.lut');
        const operation = store.getState().operation!;
        document.dispatchEvent(new CustomEvent(LOAD_PROGRESS_EVENT, {
            detail: { operationId: operation.operationId, loaded: 25, total: 100, phase: 'downloading' },
        }));
        expect(store.getState().progress).toEqual({ loaded: 25, total: 100, percent: 25 });
        document.dispatchEvent(new CustomEvent(LOAD_PROGRESS_EVENT, {
            detail: { operationId: operation.operationId, loaded: 50, total: null, phase: 'downloading' },
        }));
        expect(store.getState().progress?.percent).toBeNull();
        await load;
        expect(store.getState().artifact.phase).toBe('finalizing');
    });

    it('failed replacement leaves run disabled rather than falling back', async () => {
        const adapter = createModelSourceAdapter(store, sources(), worker());
        await adapter.loadFromUrl('https://example.com/a.lut');
        adapter.handleWorkerMessage({ action: 'model_loaded', capabilities: { schema: 'old' } });
        expect(store.canRun()).toBe(true);
        await adapter.loadFromUrl('https://example.com/b.lut');
        adapter.handleWorkerMessage({ action: 'operation_error', error: 'bad model' });
        expect(store.getState().artifact.phase).toBe('error');
        expect(store.getState().residentArtifact?.sourceId).toBe('url');
        expect(store.canRun()).toBe(false);
    });

    it('forwards published, local-file, and bundled operations', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        await adapter.loadPublished('published-a');
        expect(api.loadPublished).toHaveBeenCalledWith('published-a', { operationId: 1, sourceId: 'published-a' });
        await adapter.loadLocalFile(new File([new Uint8Array()], 'local.lut'));
        expect(api.loadLocalFile).toHaveBeenCalledWith(expect.any(File), { operationId: 2, sourceId: 'local-file' });
        await adapter.loadBundled();
        expect(api.loadBundled).toHaveBeenCalledWith({ operationId: 3, sourceId: 'bundled-demo' });
    });

    it('uses the existing API test path without an artifact-ready claim', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        adapter.selectApi('api-a', 'https://api.example.com/v1');
        expect(store.getState().api.testStatus).toBe('untested');
        expect(store.getState().artifact.phase).toBe('unselected');
        await adapter.testApi('api-a');
        expect(api.testApi).toHaveBeenCalledWith('api-a');
        expect(store.getState().api.testStatus).toBe('passed');
    });
});
