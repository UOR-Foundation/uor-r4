import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { SessionApi, SessionSummary } from '../adapters/workflow.js';
import { SessionList } from './SessionList.js';

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

const summary: SessionSummary = { id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 2, preview: 'Latest message' };

function sessions(items: SessionSummary[] = [summary]): SessionApi {
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'), listSessions: vi.fn(() => items), activeSessionId: vi.fn(() => items[0]?.id ?? null),
        selectSession: vi.fn(() => true), newSession: vi.fn(() => true), renameSession: vi.fn(() => true),
        duplicateSession: vi.fn(() => 'session-copy'), exportSession: vi.fn(() => '{"schema":"uor-r4.studio-session/1"}'),
        deleteSession: vi.fn(() => true), getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true),
        sendPrompt: vi.fn(() => 1), predict: vi.fn(() => 1), stopGeneration: vi.fn(),
    };
}

function mount(api = sessions()) {
    const store = new LifecycleStore();
    const download = vi.fn();
    const onError = vi.fn();
    act(() => { render(<SessionList store={store} sessions={api} download={download} onError={onError} />, host); });
    return { store, api, download, onError };
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('SessionList', () => {
    it('shows an honest empty state', () => {
        mount(sessions([]));
        expect(host.textContent).toContain('No sessions yet');
        expect(button('New session').disabled).toBe(false);
    });

    it('supports new, select, rename, duplicate, export, and confirmed delete', () => {
        const { store, api, download } = mount();
        act(() => { button('New session').click(); });
        expect(api.newSession).toHaveBeenCalledOnce();
        expect(store.getState().session.phase).toBe('active');
        act(() => { button('Select Session A').click(); });
        expect(api.selectSession).toHaveBeenCalledWith('session-a');
        act(() => { button('Rename Session A').click(); });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Session title"]')!;
        act(() => { input.value = 'Renamed session'; input.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => { button('Save title').click(); });
        expect(api.renameSession).toHaveBeenCalledWith('session-a', 'Renamed session');
        act(() => { button('Duplicate Session A').click(); });
        expect(api.duplicateSession).toHaveBeenCalledWith('session-a');
        act(() => { button('Export Session A').click(); });
        expect(download).toHaveBeenCalledWith('Session A.json', '{"schema":"uor-r4.studio-session/1"}');
        act(() => { button('Delete Session A').click(); });
        expect(host.querySelector('dialog')).not.toBeNull();
        act(() => { button('Confirm delete').click(); });
        expect(api.deleteSession).toHaveBeenCalledWith('session-a');
    });
});
