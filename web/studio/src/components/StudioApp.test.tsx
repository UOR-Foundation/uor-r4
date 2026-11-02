import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { StudioApp } from './StudioApp.js';
import { makeFiles, makeModelAdapter, makeSessions, makeSources, makeUi } from './studioTestFixtures.js';

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function destination(label: string): HTMLButtonElement {
    const navigation = host.querySelector('nav[aria-label="Studio"]')!;
    return Array.from(navigation.querySelectorAll('button')).find((node) => node.textContent === label)!;
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('StudioApp', () => {
    it('composes all four destinations with one status source and on-demand inspector', () => {
        const store = new LifecycleStore();
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelectorAll('main')).toHaveLength(1);
        expect(host.querySelectorAll('[role="status"]')).toHaveLength(1);
        expect(host.textContent).toContain('No messages yet');
        act(() => { destination('Workspace').click(); });
        expect(host.textContent).toContain('No folder open');
        act(() => { destination('Models').click(); });
        expect(host.textContent).toContain('API providers');
        act(() => { destination('Guide').click(); });
        expect(host.textContent).toContain('next-token prediction');
        expect(host.textContent).toContain('not a conversational assistant');
        act(() => { destination('Agent').click(); });
        expect(host.textContent).toContain('No messages yet');
        act(() => { button('Inspector').click(); });
        expect(host.querySelector('#studio-inspector')).not.toBeNull();
        act(() => { button('Close inspector').click(); });
        expect(host.querySelector('#studio-inspector')).toBeNull();
        act(() => { button('Settings').click(); });
        expect(host.textContent).toContain('Appearance');
        expect(host.querySelector('dialog')).not.toBeNull();
    });

    it('does not label an API selection without an endpoint as local execution', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: undefined as unknown as string });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelector('[role="status"]')?.textContent).toContain('Invalid API endpoint');
        expect(host.querySelector('[role="status"]')?.textContent).not.toContain('localWASM');
    });

    it('uses lifecycle transitions for palette saves', async () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/dirty' });
        const files = makeFiles();
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={files} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Command palette').click(); });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Search commands"]')!;
        act(() => { input.value = 'Save file'; input.dispatchEvent(new Event('input', { bubbles: true })); });
        await act(async () => {
            input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
            await Promise.resolve();
        });
        expect(store.getState().file.phase).toBe('saved');
    });

    it('uses current selection privacy after a previous generation completes', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } });
        store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 });
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: { schema: 'uor-r4.lut-stack/1', canGenerate: true } });
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelector('[role="status"]')?.textContent).toContain('localWASM');
        expect(host.querySelector('[role="status"]')?.textContent).not.toContain('remoteAPI');
    });

    it('closes lifecycle file state when editor tabs are cleared outside Workspace', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/dirty' });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Settings').click(); });
        act(() => { button('Clear editor tabs').click(); });
        act(() => { button('Confirm clear editor tabs').click(); });
        expect(store.getState().file.phase).toBe('no-open-file');
        expect(store.getState().file.path).toBeNull();
    });

    it('refreshes the session list after confirmed clearing', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        const ui = makeUi();
        let items = [{ id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 1, preview: 'message' }];
        sessions.listSessions = vi.fn(() => items);
        ui.clearSessions = vi.fn(() => { items = []; });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={ui} download={vi.fn()} />, host);
        });
        expect(host.textContent).toContain('Session A');
        act(() => { button('Settings').click(); });
        act(() => { button('Clear sessions').click(); });
        act(() => { button('Confirm clear sessions').click(); });
        expect(host.textContent).toContain('No sessions yet');
    });
});
