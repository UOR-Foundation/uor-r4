import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { ModelSourceAdapter } from '../adapters/modelSourceAdapter.js';
import type { ApiProvider, LoadResult, ModelSourcesApi } from '../../assets/js/model_sources.js';
import { ModelPicker } from './ModelPicker.js';

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

const provider: ApiProvider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: 'secret-sentinel' };
const result: LoadResult = { bytes: new Uint8Array(), fileName: 'model.lut', expectedSha256: 'abc', meta: { schema: 'uor-r4.lut-stack/1' } };

function adapter(): ModelSourceAdapter {
    return {
        loadPublished: vi.fn(async () => result), loadFromUrl: vi.fn(async () => result),
        loadLocalFile: vi.fn(async () => result), loadBundled: vi.fn(async () => result),
        selectApi: vi.fn(), testApi: vi.fn(async () => ({ ok: true })), handleWorkerMessage: vi.fn(),
    };
}

function sources(): ModelSourcesApi {
    const list = () => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] });
    return {
        list: vi.fn(list), active: vi.fn(() => list().active), apiConfig: vi.fn(() => provider),
        catalog: vi.fn(() => ({ models: [{ id: 'm1-a', label: 'M1 seed A', url: 'https://example.com/m1.lut' }] })),
        setActive: vi.fn(), addApi: vi.fn((input: Partial<ApiProvider>) => ({ ...provider, ...input, id: 'api-new', apiKey: input.apiKey || '' })),
        updateApi: vi.fn((id: string, patch: Partial<ApiProvider>) => ({ ...provider, ...patch, id })), removeApi: vi.fn(() => list()),
        loadPublished: vi.fn(async () => result), loadFromUrl: vi.fn(async () => result),
        loadLocalFile: vi.fn(async () => result), loadBundled: vi.fn(async () => result),
        testApi: vi.fn(async () => ({ ok: true })),
    };
}

function mount(store = new LifecycleStore(), modelAdapter = adapter(), modelSources = sources()) {
    const onError = vi.fn();
    act(() => { render(<ModelPicker store={store} adapter={modelAdapter} sources={modelSources} onError={onError} />, host); });
    return { store, modelAdapter, modelSources, onError };
}

function input(label: string): HTMLInputElement {
    return host.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('ModelPicker', () => {
    it('subscribes to selected-not-ready and known byte progress', () => {
        const { store } = mount();
        act(() => { store.dispatch({ type: 'artifact/selected', sourceId: 'url' }); });
        expect(host.textContent).toContain('Selected model; not ready');
        act(() => { store.dispatch({ type: 'artifact/downloadProgress', operation: store.getState().operation!, loaded: 25, total: 100 }); });
        const progress = host.querySelector<HTMLProgressElement>('progress[aria-label="Model download progress"]')!;
        expect(progress.max).toBe(100);
        expect(progress.value).toBe(25);
        expect(host.textContent).toContain('25 / 100 bytes (25%)');
    });

    it('keeps unknown totals and finalization indeterminate', () => {
        const { store } = mount();
        store.dispatch({ type: 'artifact/selected', sourceId: 'url' });
        act(() => { store.dispatch({ type: 'artifact/downloadProgress', operation: store.getState().operation!, loaded: 50, total: null }); });
        const progress = host.querySelector<HTMLProgressElement>('progress')!;
        expect(progress.hasAttribute('value')).toBe(false);
        expect(host.textContent).toContain('50 bytes downloaded; total unknown');
        act(() => { store.dispatch({ type: 'artifact/finalizing', operation: store.getState().operation! }); });
        expect(host.textContent).toContain('Finalizing model; progress is indeterminate');
        expect(host.textContent).not.toContain('100%');
    });

    it('shows resident verification and next-token capability without calling it chat', () => {
        const { store } = mount();
        store.dispatch({ type: 'artifact/selected', sourceId: 'url' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                schema: 'uor-r4.lut-discrete-trunk/1', fileName: 'm1.lut', sha256: 'abc123', verified: true,
                engineSupported: true, canGenerate: false, canPredict: true, source: 'url',
            } });
        });
        expect(host.textContent).toContain('m1.lut');
        expect(host.textContent).toContain('SHA-256 abc123 (verified)');
        expect(host.textContent).toContain('Engine supported');
        expect(host.textContent).toContain('Next-token prediction only');
        expect(host.textContent).not.toContain('Conversational chat');
    });

    it('forwards bundled, published, URL, and local-file loads', () => {
        const { modelAdapter } = mount();
        button('Load bundled demo').click();
        button('Load M1 seed A').click();
        act(() => {
            input('Model URL').value = 'https://example.com/model.lut';
            input('Model URL').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="load-url"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        const file = new File([new Uint8Array()], 'local.lut');
        Object.defineProperty(input('Model file'), 'files', { configurable: true, value: [file] });
        input('Model file').dispatchEvent(new Event('change', { bubbles: true }));
        expect(modelAdapter.loadBundled).toHaveBeenCalledOnce();
        expect(modelAdapter.loadPublished).toHaveBeenCalledWith('m1-a');
        expect(modelAdapter.loadFromUrl).toHaveBeenCalledWith('https://example.com/model.lut');
        expect(modelAdapter.loadLocalFile).toHaveBeenCalledWith(file);
    });

    it('manages providers through native dialogs without rendering an existing secret', () => {
        const { store, modelAdapter, modelSources } = mount();
        modelAdapter.selectApi = vi.fn((id: string, endpoint: string) => store.dispatch({ type: 'api/selected', sourceId: id, endpoint }));
        expect(host.innerHTML).not.toContain('secret-sentinel');
        button('Use API A').click();
        expect(modelAdapter.selectApi).toHaveBeenCalledWith('api-a', provider.baseUrl);
        button('Test API A').click();
        expect(modelAdapter.selectApi).toHaveBeenCalledTimes(2);
        expect(modelAdapter.testApi).toHaveBeenCalledWith('api-a');
        act(() => { button('Edit API A').click(); });
        expect(host.querySelector('dialog')).not.toBeNull();
        expect(host.innerHTML).not.toContain('secret-sentinel');
        act(() => {
            input('Provider base URL').value = 'https://new.example.com/v1';
            input('Provider base URL').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="save-provider"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        expect(modelSources.updateApi).toHaveBeenCalledWith('api-a', expect.objectContaining({ baseUrl: 'https://new.example.com/v1' }));
        expect(vi.mocked(modelSources.updateApi).mock.calls[0][1]).not.toHaveProperty('apiKey');
        expect(modelAdapter.selectApi).toHaveBeenCalledWith('api-a', 'https://new.example.com/v1');
        act(() => { button('Remove API A').click(); });
        act(() => { button('Confirm removal').click(); });
        expect(modelSources.removeApi).toHaveBeenCalledWith('api-a');
        expect(store.getState().selectedSource?.kind).toBe('artifact');
        expect(store.canRun()).toBe(false);
    });
});
