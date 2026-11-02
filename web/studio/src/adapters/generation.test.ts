import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { createGenerationAdapter, GENERATION_FINISHED_EVENT } from './generation.js';
import type { SessionApi } from './workflow.js';

function sessions(): SessionApi {
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'), listSessions: vi.fn(() => []), activeSessionId: vi.fn(() => null),
        selectSession: vi.fn(() => true), newSession: vi.fn(() => true), renameSession: vi.fn(() => true),
        duplicateSession: vi.fn(() => null), exportSession: vi.fn(() => null), deleteSession: vi.fn(() => true),
        getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true),
        sendPrompt: vi.fn(() => true), stopGeneration: vi.fn(),
    };
}

describe('generation adapter', () => {
    let store: LifecycleStore;
    beforeEach(() => { store = new LifecycleStore(); });

    it('refuses to send before a source is ready', () => {
        const api = sessions();
        const adapter = createGenerationAdapter(store, api);
        expect(adapter.send('hello')).toBe(false);
        expect(api.sendPrompt).not.toHaveBeenCalled();
        adapter.dispose();
    });

    it('pins source and privacy through selection changes and stopped completion', () => {
        const api = sessions();
        const adapter = createGenerationAdapter(store, api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(adapter.send('hello')).toBe(true);
        expect(api.sendPrompt).toHaveBeenCalledWith('hello');
        expect(store.getState().session.activeGeneration?.privacy).toBe('remoteAPI');
        store.dispatch({ type: 'api/selected', sourceId: 'api-b', endpoint: 'http://127.0.0.1:8000/v1' });
        document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'stopped' } }));
        expect(store.getState().session.phase).toBe('stopped');
        expect(store.getState().session.lastGeneration?.source.id).toBe('api-a');
        expect(store.getState().session.lastGeneration?.privacy).toBe('remoteAPI');
        adapter.dispose();
    });

    it('records rejected sends and stops listening after disposal', () => {
        const api = sessions();
        api.sendPrompt = vi.fn(() => false);
        const adapter = createGenerationAdapter(store, api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(adapter.send('hello')).toBe(false);
        expect(store.getState().session.phase).toBe('error');
        adapter.dispose();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'complete' } }));
        expect(store.getState().session.phase).toBe('error');
    });
});
