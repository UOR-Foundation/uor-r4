import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'preact';
import { LifecycleStore } from './state/lifecycle.js';
import { mountTypedStudio, typedPreviewRequested } from './bootstrap.js';
import { StudioApp } from './components/StudioApp.js';
import { activateStudioMigration, studioMigrationStatus } from './persistence/migration.js';

vi.mock('preact', async (importOriginal) => {
    const actual = await importOriginal<typeof import('preact')>();
    return { ...actual, render: vi.fn() };
});

vi.mock('../assets/js/model_sources.js', () => ({ ModelSources: {}, LOAD_PROGRESS_EVENT: 'uor-model-load-progress' }));

class FakeWorker {
    postMessage = vi.fn();
    addEventListener = vi.fn();
    removeEventListener = vi.fn();
    terminate = vi.fn();
}

const legacyWindow = window as unknown as {
    __studioApp?: { worker: FakeWorker };
    __studioFiles: Record<string, unknown>;
    __studioUi: Record<string, unknown>;
};

describe('typed Studio bootstrap', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.stubGlobal('Worker', FakeWorker);
        activateStudioMigration(false);
        window.localStorage.clear();
        delete legacyWindow.__studioApp;
        legacyWindow.__studioFiles = { tabs: () => [], activeFile: () => null };
        legacyWindow.__studioUi = {
            getSettings: () => ({}),
            getAttachments: () => [],
            listSessions: () => [],
            activeSessionId: () => null,
            getConversationMessages: () => [],
        };
    });

    it('defaults to typed mounting while preserving explicit legacy selections', () => {
        expect(typedPreviewRequested('?typed=1')).toBe(true);
        expect(typedPreviewRequested('?typed=0')).toBe(false);
        expect(typedPreviewRequested('?legacy=1')).toBe(false);
        expect(typedPreviewRequested('?other=1')).toBe(true);
        expect(typedPreviewRequested('')).toBe(true);
    });

    it('passes the real StudioApp with lifecycle, model worker, and workflow APIs to Preact', async () => {
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        expect(render).toHaveBeenCalledOnce();
        expect(render).toHaveBeenCalledWith(expect.objectContaining({ type: StudioApp }), root);
        expect(root.hidden).toBe(false);
        expect(mounted.store).toBeInstanceOf(LifecycleStore);
        expect(mounted.worker).toBeInstanceOf(FakeWorker);
        expect(mounted.worker.addEventListener).toHaveBeenCalledWith('message', expect.any(Function));
        expect(mounted.files).toBe(legacyWindow.__studioFiles);
        expect(mounted.sessions).toBe(legacyWindow.__studioUi);
        expect(mounted.ui).toBe(legacyWindow.__studioUi);
        expect(['migrated', 'current']).toContain(studioMigrationStatus().state);
        mounted.dispose();
        expect(root.hidden).toBe(true);
    });

    it('preserves an already migrated store instead of reactivating it', async () => {
        activateStudioMigration(true);
        expect(studioMigrationStatus().state).toBe('migrated');
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        expect(studioMigrationStatus().state).toBe('migrated');
        mounted.dispose();
    });

    it('reuses an existing model worker instead of starting another engine', async () => {
        const root = document.createElement('div');
        const existing = new FakeWorker();
        legacyWindow.__studioApp = { worker: existing };
        const mounted = await mountTypedStudio(root);
        expect(mounted.worker).toBe(existing);
        expect(mounted.worker.addEventListener).toHaveBeenCalledWith('message', expect.any(Function));
        mounted.dispose();
        expect(existing.terminate).not.toHaveBeenCalled();
        expect(root.hidden).toBe(true);
    });

    it('refuses to mount when session methods are missing', async () => {
        const root = document.createElement('div');
        legacyWindow.__studioUi = { getSettings: () => ({}), getAttachments: () => [] };
        await expect(mountTypedStudio(root, { timeoutMs: 10 })).rejects.toThrow('did not initialize');
        expect(render).not.toHaveBeenCalled();
        expect(root.hidden).toBe(false);
    });

    it('disposes the installed legacy shortcut guard when workflow initialization rejects', async () => {
        const palette = document.createElement('div');
        palette.id = 'commandPalette';
        palette.hidden = true;
        document.body.appendChild(palette);
        const remove = vi.spyOn(window, 'removeEventListener');
        const root = document.createElement('div');
        legacyWindow.__studioUi = { getSettings: () => ({}), getAttachments: () => [] };
        try {
            await expect(mountTypedStudio(root, { timeoutMs: 10 })).rejects.toThrow('did not initialize');
            expect(remove).toHaveBeenCalledWith('keydown', expect.any(Function), true);

            const blocked = new KeyboardEvent('keydown', { key: 'k', metaKey: true, bubbles: true, cancelable: true });
            document.body.dispatchEvent(blocked);
            expect(blocked.defaultPrevented, 'the legacy Cmd/Ctrl+K blocker must be removed').toBe(false);

            palette.hidden = false;
            window.dispatchEvent(new CustomEvent('uor-studio-v2-ready'));
            await new Promise((resolve) => setTimeout(resolve, 0));
            expect(palette.hidden, 'the readiness listener must be removed').toBe(false);

            palette.hidden = true;
            await new Promise((resolve) => setTimeout(resolve, 0));
            palette.hidden = false;
            await new Promise((resolve) => setTimeout(resolve, 0));
            expect(palette.hidden, 'the palette observer must be disconnected').toBe(false);
        } finally {
            palette.remove();
            remove.mockRestore();
        }
    });

    it('suppresses legacy Cmd/Ctrl+comma for editable, IME and modified chords', async () => {
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        const editable = document.createElement('div');
        editable.setAttribute('contenteditable', 'true');
        const child = document.createElement('span');
        editable.appendChild(child);
        document.body.appendChild(editable);
        const input = document.createElement('input');
        document.body.appendChild(input);
        const outside = document.createElement('button');
        document.body.appendChild(outside);
        const press = (target: EventTarget, init: KeyboardEventInit & { isComposing?: boolean; keyCode?: number }) => {
            const event = new KeyboardEvent('keydown', { key: ',', bubbles: true, cancelable: true, ...init });
            Object.defineProperties(event, {
                isComposing: { value: Boolean(init.isComposing) },
                keyCode: { value: init.keyCode ?? 0 },
            });
            target.dispatchEvent(event);
            return event;
        };
        const legacy = vi.fn();
        document.addEventListener('keydown', legacy);
        try {
            expect(press(child, { metaKey: true }).defaultPrevented, 'contenteditable descendant').toBe(true);
            expect(press(input, { ctrlKey: true }).defaultPrevented, 'input').toBe(true);
            expect(legacy).not.toHaveBeenCalled();

            expect(press(outside, { metaKey: true }).defaultPrevented, 'outside editable').toBe(false);
            expect(legacy).toHaveBeenCalledTimes(1);
            const baseline = legacy.mock.calls.length;

            for (const [context, target, init] of [
                ['IME composition in editable', child, { metaKey: true, isComposing: true }],
                ['IME keyCode 229 in editable', child, { metaKey: true, keyCode: 229 }],
                ['Alt+Meta comma in editable', child, { metaKey: true, altKey: true }],
                ['Alt+Ctrl comma outside', outside, { ctrlKey: true, altKey: true }],
                ['Shift+Meta comma in editable', child, { metaKey: true, shiftKey: true }],
                ['Shift+Meta comma outside', outside, { metaKey: true, shiftKey: true }],
            ] as const) {
                const event = press(target, init);
                expect(event.defaultPrevented, context).toBe(false);
                expect(legacy.mock.calls.length, context).toBe(baseline);
            }

            expect(press(child, { metaKey: true, shiftKey: true, key: 'P' } as KeyboardEventInit).defaultPrevented, 'scoped palette chord').toBe(false);
        } finally {
            document.removeEventListener('keydown', legacy);
            editable.remove();
            input.remove();
            outside.remove();
            mounted.dispose();
        }
    });

    it('refuses Preact rendering when migration fails', async () => {
        window.localStorage.setItem('uor.studio.settings.v1', '{bad-json');
        expect(activateStudioMigration(true).state).toBe('error');
        expect(studioMigrationStatus().state).toBe('error');
        const root = document.createElement('div');

        await expect(mountTypedStudio(root)).rejects.toThrow('migration');
        expect(render).not.toHaveBeenCalled();
    });

    it('applies an explicit theme query through the existing settings API', async () => {
        const updateSettings = vi.fn();
        legacyWindow.__studioUi = { ...legacyWindow.__studioUi, updateSettings };
        window.history.replaceState({}, '', '/?typed=1&theme=light');
        try {
            const root = document.createElement('div');
            const mounted = await mountTypedStudio(root);
            expect(updateSettings).toHaveBeenCalledWith({ theme: 'light' });
            mounted.dispose();
        } finally {
            window.history.replaceState({}, '', '/');
        }
    });
});
