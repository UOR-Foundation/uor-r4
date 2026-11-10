import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { GenerationAdapter } from '../adapters/generation.js';
import type { Attachment, SessionApi, StudioSettings, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import { AgentComposer } from './AgentComposer.js';

let host: HTMLDivElement;
let createObjectURL: ReturnType<typeof vi.fn>;
let revokeObjectURL: ReturnType<typeof vi.fn>;
let urlSequence = 0;
const originalCreateObjectURL = Object.getOwnPropertyDescriptor(URL, 'createObjectURL');
const originalRevokeObjectURL = Object.getOwnPropertyDescriptor(URL, 'revokeObjectURL');

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    urlSequence = 0;
    createObjectURL = vi.fn(() => `blob:test/${++urlSequence}`);
    revokeObjectURL = vi.fn();
    Object.defineProperty(URL, 'createObjectURL', { configurable: true, writable: true, value: createObjectURL });
    Object.defineProperty(URL, 'revokeObjectURL', { configurable: true, writable: true, value: revokeObjectURL });
});

afterEach(() => {
    act(() => { render(null, host); });
    host.remove();
    vi.restoreAllMocks();
    if (originalCreateObjectURL) Object.defineProperty(URL, 'createObjectURL', originalCreateObjectURL);
    else Reflect.deleteProperty(URL, 'createObjectURL');
    if (originalRevokeObjectURL) Object.defineProperty(URL, 'revokeObjectURL', originalRevokeObjectURL);
    else Reflect.deleteProperty(URL, 'revokeObjectURL');
});

function sessions(): SessionApi {
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'), listSessions: vi.fn(() => []), activeSessionId: vi.fn(() => null),
        selectSession: vi.fn(() => true), newSession: vi.fn(() => true), renameSession: vi.fn(() => true),
        duplicateSession: vi.fn(() => null), exportSession: vi.fn(() => null), deleteSession: vi.fn(() => true),
        getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true), sendPrompt: vi.fn(() => 1), predict: vi.fn(() => 1), stopGeneration: vi.fn(),
    };
}

function workspace(active = true): WorkspaceApi {
    return {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(), openFile: vi.fn(async () => undefined),
        newFile: vi.fn(), closeTab: vi.fn(), closeAll: vi.fn(), saveActive: vi.fn(async () => true), saveAsCopy: vi.fn(() => true),
        insertText: vi.fn(() => true), setActiveContent: vi.fn(),
        activeFile: vi.fn(() => active ? { path: 'notes.md', name: 'notes.md', content: 'context text', mode: 'local' } : null),
        workspaceName: vi.fn(() => 'project'), mode: vi.fn(() => 'local'), tabs: vi.fn(() => []),
        treeSummary: vi.fn(() => ({ mode: 'local', rootName: 'project', fileCount: 1, tabs: [], activePath: active ? 'notes.md' : null })),
    };
}

function ui() {
    const attachments: Attachment[] = [];
    const settings: StudioSettings = { theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false };
    return {
        attachments,
        api: {
            setView: vi.fn(), toast: vi.fn(), getSettings: vi.fn(() => settings), updateSettings: vi.fn(),
            getAttachments: vi.fn(() => attachments), addAttachment: vi.fn((entry: Attachment) => {
                const index = attachments.findIndex((item) => item.name === entry.name);
                if (index >= 0) attachments.splice(index, 1);
                attachments.push(entry);
            }),
            removeAttachment: vi.fn((name: string) => { const index = attachments.findIndex((item) => item.name === name); if (index >= 0) attachments.splice(index, 1); }),
            clearAttachments: vi.fn(() => { attachments.splice(0); }), templatePrompt: vi.fn((kind: string) => `Prompt for ${kind}`),
            clearSessions: vi.fn(() => ({ cleared: 0, total: 0 })), clearEditor: vi.fn(), updateStatus: vi.fn(),
        } satisfies StudioUiApi,
    };
}

interface MountOptions {
    selected?: boolean;
    activeFile?: boolean;
}

function mount(options: MountOptions = {}) {
    const store = new LifecycleStore();
    const sessionApi = sessions();
    const fileApi = workspace(options.activeFile !== false);
    const uiApi = ui();
    const generation: GenerationAdapter = {
        send: vi.fn(() => true), stop: vi.fn(), dispose: vi.fn(),
    };
    if (options.selected !== false) store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
    const copy = vi.fn(async () => undefined);
    const download = vi.fn();
    const props = { store, sessions: sessionApi, files: fileApi, ui: uiApi.api, generation, copy, download };
    act(() => { render(<AgentComposer {...props} />, host); });
    return { store, sessionApi, fileApi, uiApi, generation, copy, download };
}

function promptElement() {
    const input = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]');
    if (!input) throw new Error('Missing prompt textarea');
    return input;
}

function typePrompt(value: string, caret = value.length) {
    const input = promptElement();
    act(() => {
        input.value = value;
        input.setSelectionRange(caret, caret);
        input.dispatchEvent(new Event('input', { bubbles: true }));
    });
    return input;
}

function pressKey(textarea: HTMLTextAreaElement, key: string, init: { shiftKey?: boolean; isComposing?: boolean; keyCode?: number } = {}) {
    const event = new KeyboardEvent('keydown', { key, shiftKey: init.shiftKey, bubbles: true, cancelable: true });
    Object.defineProperties(event, {
        isComposing: { value: Boolean(init.isComposing) },
        keyCode: { value: init.keyCode || 0 },
    });
    textarea.dispatchEvent(event);
    return event;
}

function pasteFiles(textarea: HTMLTextAreaElement, files: File[], text = '') {
    const event = new Event('paste', { bubbles: true, cancelable: true });
    Object.defineProperty(event, 'clipboardData', { value: { files, getData: () => text } });
    textarea.dispatchEvent(event);
    return event;
}

function button(label: string) {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

function imageFile(name = 'pixel.png', bytes = [137, 80, 78, 71]) {
    return new File([new Uint8Array(bytes)], name, { type: 'image/png' });
}

async function settle() {
    await act(async () => { await Promise.resolve(); });
}

async function loadModule(name: string): Promise<any> {
    return import(/* @vite-ignore */ `./${name}.js`);
}

async function mountSurface(overrides: Record<string, unknown> = {}) {
    const { ComposerSurface } = await loadModule('ComposerSurface');
    const props = {
        value: '', onChange: vi.fn(), attachments: [], onAttach: vi.fn(), onRemove: vi.fn(), onSend: vi.fn(() => true),
        mode: 'ask', onMode: vi.fn(),
        model: 'model-a', onModel: vi.fn(), environment: 'Local browser', onEnvironment: vi.fn(),
        disabled: false, runtimeAvailable: false, templatePrompt: (kind: string) => `Prompt for ${kind}`,
        ...overrides,
    };
    const Surface = ComposerSurface as (surfaceProps: typeof props) => any;
    act(() => { render(<Surface {...props} />, host); });
    return props;
}

function fakeFile(name: string, size: number, type: string): File {
    return { name, size, type } as File;
}

function deferredImage(name: string, size: number) {
    const file = new File([new Uint8Array(1)], name, { type: 'image/png' });
    Object.defineProperty(file, 'size', { value: size });
    let resolve!: () => void;
    const pending = new Promise<ArrayBuffer>((done) => { resolve = () => done(new Uint8Array(size).buffer); });
    Object.defineProperty(file, 'arrayBuffer', { value: () => pending });
    return { file, resolve };
}

function deferredDocument(name: string, size: number, content = 'x') {
    const file = new File([content], name, { type: 'text/plain' });
    Object.defineProperty(file, 'size', { value: size });
    let resolve!: () => void;
    const pending = new Promise<string>((done) => { resolve = () => done(content); });
    Object.defineProperty(file, 'text', { value: () => pending });
    return { file, resolve };
}

async function flushAsync() {
    await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
}

describe('composer surface regression and editor behaviour', () => {
    it('existing_ime_composition_regression keeps Enter, IME composition, and history intact', () => {
        const { generation } = mount();
        const textarea = promptElement();
        typePrompt('first\nline');
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('first\nline');
        typePrompt('second');
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('first\nline');
        pressKey(textarea, 'ArrowDown');
        expect(textarea.value).toBe('second');
        typePrompt('in progress');
        pressKey(textarea, 'Enter', { isComposing: true });
        pressKey(textarea, 'Enter', { keyCode: 229 });
        expect(generation.send).toHaveBeenCalledTimes(1);
    });

    it('shift_enter_inserts_newline_at_caret', () => {
        const { generation } = mount();
        const textarea = promptElement();
        typePrompt('abcd', 2);
        const enter = pressKey(textarea, 'Enter', { shiftKey: true });
        expect(enter.defaultPrevented).toBe(false);
        expect(textarea.selectionStart).toBe(2);
        act(() => {
            textarea.value = `${textarea.value.slice(0, 2)}\n${textarea.value.slice(2)}`;
            textarea.setSelectionRange(3, 3);
            textarea.dispatchEvent(new Event('input', { bubbles: true }));
        });
        expect(textarea.value).toBe('ab\ncd');
        expect(textarea.selectionStart).toBe(3);
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('ab\ncd');
    });

    it('visible_send_button_and_prompt_label_are_preserved', () => {
        const { store, generation } = mount({ selected: false });
        expect(host.querySelector('textarea[aria-label="Prompt"]')).not.toBeNull();
        expect(button('Send').disabled).toBe(true);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        typePrompt('Hello');
        expect(button('Send').disabled).toBe(false);
        act(() => { button('Send').click(); });
        expect(generation.send).toHaveBeenCalledWith('Hello');
        expect(host.querySelector('.agent-composer[data-composer-surface="true"]')).not.toBeNull();
    });

    it('keeps one generation send path through the integrated composer', () => {
        const { generation } = mount();
        typePrompt('single path');
        act(() => { button('Send').click(); });
        expect(generation.send).toHaveBeenCalledTimes(1);
        expect(generation.send).toHaveBeenCalledWith('single path');
    });
});

describe('bounded shared attachment budget', () => {
    it('accepts exact per-file boundaries and rejects limit plus one with exact sizes', async () => {
        const { planAttachmentBatch, MAX_IMAGE_BYTES, MAX_DOCUMENT_BYTES } = await loadModule('ComposerAttachments');
        const exact = planAttachmentBatch([
            fakeFile('exact.png', MAX_IMAGE_BYTES, 'image/png'),
            fakeFile('exact.md', MAX_DOCUMENT_BYTES, 'text/markdown'),
        ], []);
        expect(exact.errors).toEqual([]);
        expect(exact.accepted.map((entry: { kind: string }) => entry.kind)).toEqual(['image', 'document']);

        const over = planAttachmentBatch([
            fakeFile('photo.png', MAX_IMAGE_BYTES + 1, 'image/png'),
            fakeFile('report.md', MAX_DOCUMENT_BYTES + 1, 'text/markdown'),
        ], []);
        expect(over.accepted).toEqual([]);
        expect(over.errors).toEqual([
            'photo.png exceeds the 10485760-byte image limit (10485761 bytes).',
            'report.md exceeds the 4194304-byte document limit (4194305 bytes).',
        ]);
    });

    it('counts existing text and images for the five-attachment limit and allows same-name replacement', async () => {
        const { planAttachmentBatch } = await loadModule('ComposerAttachments');
        const existing = Array.from({ length: 5 }, (_, index) => ({ name: `file-${index}.md`, size: 1, kind: 'text' as const }));
        const overflow = planAttachmentBatch([fakeFile('six.md', 1, 'text/markdown')], existing);
        expect(overflow.accepted).toEqual([]);
        expect(overflow.errors).toEqual(['six.md exceeds the 5-attachment limit (6 attachments requested).']);
        const replaced = planAttachmentBatch([fakeFile('file-4.md', 2, 'text/markdown')], existing);
        expect(replaced.errors).toEqual([]);
        expect(replaced.accepted).toHaveLength(1);
    });

    it('accepts the exact 25 MiB total across mixed images and documents and rejects one byte over', async () => {
        const { planAttachmentBatch, MAX_IMAGE_BYTES, MAX_DOCUMENT_BYTES, MAX_TOTAL_ATTACHMENT_BYTES } = await loadModule('ComposerAttachments');
        const exactBatch = [
            fakeFile('a.png', MAX_IMAGE_BYTES, 'image/png'),
            fakeFile('b.png', MAX_IMAGE_BYTES, 'image/png'),
            fakeFile('c.md', MAX_DOCUMENT_BYTES, 'text/markdown'),
            fakeFile('d.png', MAX_TOTAL_ATTACHMENT_BYTES - (2 * MAX_IMAGE_BYTES) - MAX_DOCUMENT_BYTES, 'image/png'),
        ];
        const exact = planAttachmentBatch(exactBatch, []);
        expect(exact.errors).toEqual([]);
        expect(exact.accepted).toHaveLength(4);
        const bytes = exactBatch.reduce((sum, file) => sum + file.size, 0);
        expect(bytes).toBe(MAX_TOTAL_ATTACHMENT_BYTES);

        const over = planAttachmentBatch([...exactBatch, fakeFile('one-byte.txt', 1, 'text/plain')], []);
        expect(over.accepted).toHaveLength(4);
        expect(over.errors).toEqual(['one-byte.txt exceeds the 26214400-byte total attachment limit (26214401 bytes).']);
    });

    it('counts a mixed batch sequentially and keeps same-name replacements from growing the budget', async () => {
        const { planAttachmentBatch, MAX_IMAGE_BYTES } = await loadModule('ComposerAttachments');
        const existing = [
            { name: 'a.png', size: 5 * 1024 * 1024, kind: 'image' as const },
            { name: 'b.png', size: 5 * 1024 * 1024, kind: 'image' as const },
            { name: 'c.md', size: 1024 * 1024, kind: 'text' as const },
            { name: 'd.md', size: 1024 * 1024, kind: 'text' as const },
        ];
        const overflow = planAttachmentBatch([
            fakeFile('e.png', MAX_IMAGE_BYTES, 'image/png'),
            fakeFile('e.md', 4 * 1024 * 1024, 'text/markdown'),
        ], existing);
        expect(overflow.accepted.map((entry: { file: { name: string } }) => entry.file.name)).toEqual(['e.png']);
        expect(overflow.errors).toEqual(['e.md exceeds the 5-attachment limit (6 attachments requested).']);

        const replacement = planAttachmentBatch([
            fakeFile('c.md', 2 * 1024 * 1024, 'text/markdown'),
            fakeFile('e.png', MAX_IMAGE_BYTES, 'image/png'),
        ], existing);
        expect(replacement.errors).toEqual([]);
        expect(replacement.accepted).toHaveLength(2);
    });

    it('applies the shared count budget to current-file and pasted text attachments', async () => {
        const { uiApi } = mount();
        for (let index = 0; index < 5; index += 1) uiApi.attachments.push({ name: `seed-${index}.md`, content: 'x', size: 1, source: 'file' });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        act(() => { button('Attach current file').click(); });
        await flushAsync();
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
        expect(uiApi.api.toast).toHaveBeenCalledWith('notes.md exceeds the 5-attachment limit (6 attachments requested).', 'error');
    });
});

describe('memory-only image and document attachments', () => {
    it('paste_image_creates_memory_only_attachment without browser persistence', async () => {
        const { uiApi } = mount();
        const textarea = promptElement();
        const setItem = vi.spyOn(Storage.prototype, 'setItem');
        const indexedOpen = vi.fn();
        const originalIndexedDB = Object.getOwnPropertyDescriptor(window, 'indexedDB');
        Object.defineProperty(window, 'indexedDB', { configurable: true, value: { open: indexedOpen } });
        pasteFiles(textarea, [imageFile()]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-kind="image"]')).not.toBeNull(); });
        expect(host.textContent).toContain('pixel.png');
        expect(createObjectURL).toHaveBeenCalledOnce();
        expect(host.querySelector<HTMLImageElement>('.composer-attachment-preview')?.src).toBe('blob:test/1');
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
        expect(uiApi.attachments).toEqual([]);
        expect(setItem).not.toHaveBeenCalled();
        expect(indexedOpen).not.toHaveBeenCalled();
        expect(host.innerHTML).not.toContain('data:image');
        if (originalIndexedDB) Object.defineProperty(window, 'indexedDB', originalIndexedDB);
        else Reflect.deleteProperty(window, 'indexedDB');
    });

    it('revokes preview URLs on remove, replacement, and unmount', async () => {
        const { } = mount();
        let textarea = promptElement();
        pasteFiles(textarea, [imageFile('photo.png', [1])]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-kind="image"]')).not.toBeNull(); });
        expect(createObjectURL).toHaveReturnedWith('blob:test/1');

        pasteFiles(textarea, [imageFile('photo.png', [2])]);
        await vi.waitFor(() => { expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/1'); });
        expect(host.querySelectorAll('[data-attachment-kind="image"]')).toHaveLength(1);

        act(() => { button('Remove photo.png').click(); });
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/2');
        expect(host.querySelector('[data-attachment-kind="image"]')).toBeNull();

        pasteFiles(textarea, [imageFile('remaining.png', [3])]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-kind="image"]')).not.toBeNull(); });
        act(() => { render(null, host); });
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/3');
    });

    it('document_over_limit_reports_exact_size and does not read or attach it', async () => {
        const { uiApi } = mount();
        const textarea = promptElement();
        const oversized = new File([new Uint8Array(4 * 1024 * 1024 + 1)], 'report.md', { type: 'text/markdown' });
        pasteFiles(textarea, [oversized]);
        await vi.waitFor(() => {
            expect(uiApi.api.toast).toHaveBeenCalledWith('report.md exceeds the 4194304-byte document limit (4194305 bytes).', 'error');
        });
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
    });

    it('accepts an exact four MiB document through readTextFiles', async () => {
        const { uiApi } = mount();
        const textarea = promptElement();
        const exact = new File(['a'.repeat(4 * 1024 * 1024)], 'exact.md', { type: 'text/markdown' });
        pasteFiles(textarea, [exact]);
        await vi.waitFor(() => {
            expect(uiApi.api.addAttachment).toHaveBeenCalledWith({ name: 'exact.md', kind: 'text', content: expect.any(String), source: 'file', size: 4194304 });
        });
        expect(uiApi.api.toast).not.toHaveBeenCalled();
        expect(host.querySelector('[data-attachment-kind="document"]')?.textContent).toContain('exact.md');
    });

    it('counts a mixed image and document batch against one budget before attaching', async () => {
        const { uiApi } = mount();
        const textarea = promptElement();
        const image = imageFile('mixed.png');
        const document = new File(['mixed document'], 'mixed.md', { type: 'text/markdown' });
        pasteFiles(textarea, [image, document]);
        await vi.waitFor(() => { expect(uiApi.api.addAttachment).toHaveBeenCalledWith(expect.objectContaining({ name: 'mixed.md' })); });
        expect(host.querySelectorAll('[data-attachment-kind="image"]')).toHaveLength(1);
        expect(host.querySelectorAll('[data-attachment-kind="document"]')).toHaveLength(1);
    });
});

describe('serialized attachment operations', () => {
    it('serializes overlapping file batches so four retained plus two 1-byte documents cannot exceed five', async () => {
        const { uiApi } = mount();
        for (let index = 0; index < 4; index += 1) uiApi.attachments.push({ name: `seed-${index}.md`, content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const textarea = promptElement();
        const first = deferredDocument('first.md', 1);
        const second = deferredDocument('second.md', 1);
        pasteFiles(textarea, [first.file]);
        pasteFiles(textarea, [second.file]);
        first.resolve();
        await flushAsync();
        second.resolve();
        await flushAsync();
        expect(uiApi.attachments).toHaveLength(5);
        expect(uiApi.attachments.map((attachment) => attachment.name)).toContain('first.md');
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('second.md');
        expect(uiApi.api.toast).toHaveBeenCalledWith('second.md exceeds the 5-attachment limit (6 attachments requested).', 'error');
    });

    it('serializes overlapping image batches so the shared 25 MiB total is never exceeded', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'existing.png', kind: 'image', content: '', size: 10 * 1024 * 1024 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const textarea = promptElement();
        const first = deferredImage('first.png', 10 * 1024 * 1024);
        const second = deferredImage('second.png', 10 * 1024 * 1024);
        pasteFiles(textarea, [first.file]);
        pasteFiles(textarea, [second.file]);
        first.resolve();
        await flushAsync();
        second.resolve();
        await flushAsync();
        expect(host.querySelectorAll('[data-attachment-kind="image"]')).toHaveLength(2);
        expect(uiApi.api.toast).toHaveBeenCalledWith('second.png exceeds the 26214400-byte total attachment limit (31457280 bytes).', 'error');
    });

    it('serializes same-name overlapping images, keeps one draft, and revokes the replaced URL', async () => {
        mount();
        const textarea = promptElement();
        const first = deferredImage('same.png', 4);
        const second = deferredImage('same.png', 5);
        pasteFiles(textarea, [first.file]);
        pasteFiles(textarea, [second.file]);
        first.resolve();
        second.resolve();
        await flushAsync();
        expect(host.querySelectorAll('[data-attachment-kind="image"]')).toHaveLength(1);
        expect(createObjectURL).toHaveBeenCalledTimes(2);
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/1');
        expect(revokeObjectURL).not.toHaveBeenCalledWith('blob:test/2');
        expect(host.querySelector<HTMLImageElement>('.composer-attachment-preview')?.src).toBe('blob:test/2');
    });

    it('queues current-file and pasted additions behind a pending file batch', async () => {
        const { uiApi } = mount();
        for (let index = 0; index < 4; index += 1) uiApi.attachments.push({ name: `seed-${index}.md`, content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const pending = deferredDocument('pending.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        act(() => { button('Attach current file').click(); });
        const pasted = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Pasted context"]')!;
        act(() => {
            pasted.value = 'pasted text';
            pasted.dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { button('Attach pasted text').click(); });
        pending.resolve();
        await flushAsync();
        expect(uiApi.attachments).toHaveLength(5);
        expect(uiApi.attachments.map((attachment) => attachment.name)).toContain('pending.md');
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('notes.md');
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('pasted-context.txt');
        expect(uiApi.api.toast).toHaveBeenCalledWith('notes.md exceeds the 5-attachment limit (6 attachments requested).', 'error');
        expect(uiApi.api.toast).toHaveBeenCalledWith('pasted-context.txt exceeds the 5-attachment limit (6 attachments requested).', 'error');
    });

    it('queues an active-file mention behind a pending batch and preserves the draft on rejection', async () => {
        const { uiApi } = mount();
        for (let index = 0; index < 4; index += 1) uiApi.attachments.push({ name: `seed-${index}.md`, content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const pending = deferredDocument('pending.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        typePrompt('@');
        const option = host.querySelector<HTMLElement>('[role="option"][data-attachment="notes.md"]')!;
        act(() => { option.click(); });
        pending.resolve();
        await flushAsync();
        expect(uiApi.attachments).toHaveLength(5);
        expect(uiApi.attachments.map((attachment) => attachment.name)).toContain('pending.md');
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('notes.md');
        expect(promptElement().value).toBe('@');
        expect(uiApi.api.toast).toHaveBeenCalledWith('notes.md exceeds the 5-attachment limit (6 attachments requested).', 'error');
    });

    it('drops a late image completion after clear without creating or adding a preview', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'seed.md', content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const pending = deferredImage('late.png', 4);
        pasteFiles(promptElement(), [pending.file]);
        act(() => { button('Clear attachments').click(); });
        pending.resolve();
        await flushAsync();
        expect(host.querySelector('[data-attachment-kind="image"]')).toBeNull();
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
        expect(createObjectURL).not.toHaveBeenCalled();
        expect(uiApi.api.toast).not.toHaveBeenCalled();
    });

    it('revokes an already-created preview URL when the remaining batch becomes stale', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'seed.md', content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const first = deferredImage('first.png', 4);
        const second = deferredImage('second.png', 4);
        pasteFiles(promptElement(), [first.file, second.file]);
        first.resolve();
        await flushAsync();
        expect(createObjectURL).toHaveBeenCalledOnce();
        act(() => { button('Clear attachments').click(); });
        second.resolve();
        await flushAsync();
        expect(host.querySelector('[data-attachment-kind="image"]')).toBeNull();
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/1');
        expect(createObjectURL).toHaveBeenCalledOnce();
    });

    it('does not attach or toast a late document after unmount', async () => {
        const { uiApi } = mount();
        const pending = deferredDocument('late-document.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        act(() => { render(null, host); });
        pending.resolve();
        await flushAsync();
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
        expect(uiApi.api.toast).not.toHaveBeenCalled();
    });

    it('shows stored image size and labels it differently from a memory-only image', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'legacy.png', kind: 'image', content: '', size: 10 * 1024 * 1024 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const budget = host.querySelector('[aria-label="Attachment budget"]')!;
        expect(budget.textContent).toContain('10 MiB/25 MiB');
        const legacy = host.querySelector<HTMLElement>('[data-attachment-id="stored:legacy.png"]')!;
        expect(legacy.textContent).toContain('image · stored context');
        expect(legacy.querySelector('[data-attachment-size]')?.getAttribute('data-attachment-size')).toBe(String(10 * 1024 * 1024));
        expect(host.textContent).toContain('Stored image context');
        pasteFiles(promptElement(), [new File([new Uint8Array(1024 * 1024)], 'new.png', { type: 'image/png' })]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-id^="image:new.png:"]')).not.toBeNull(); });
        const memory = host.querySelector<HTMLElement>('[data-attachment-id^="image:new.png:"]')!;
        expect(memory.textContent).toContain('image · memory only');
        expect(budget.textContent).toContain('11 MiB/25 MiB');
    });
});

describe('composer menus and host state', () => {
    it('at_menu_lists_editor_file_and_attachments_only', () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'external.txt', content: 'external context', source: 'workspace' });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        typePrompt('@');
        const menu = host.querySelector<HTMLElement>('[role="listbox"][aria-label="Attachment mentions"]');
        expect(menu).not.toBeNull();
        const options = Array.from(menu!.querySelectorAll<HTMLElement>('[role="option"]'));
        expect(options.map((option) => option.textContent)).toEqual(['notes.md (Active file)', 'external.txt (Attachment)']);
        expect(host.textContent).not.toContain('(Artifact)');
    });

    it('lists only explicitly supplied artifacts in the mention menu', async () => {
        const props = await mountSurface({ artifacts: [{ name: 'spec.md', content: 'artifact body', source: 'task' }] });
        typePrompt('@');
        const options = Array.from(host.querySelectorAll<HTMLElement>('[role="option"]'));
        expect(options.map((option) => option.textContent)).toEqual(['spec.md (Artifact)']);
        act(() => { options[0].click(); });
        expect(promptElement().value).toBe('@spec.md');
        expect(props.onAttach).not.toHaveBeenCalled();
    });

    it('slash_menu_lists_the_six_local_templates', async () => {
        const props = await mountSurface();
        typePrompt('/');
        const options = Array.from(host.querySelectorAll<HTMLElement>('[role="option"]'));
        expect(options.map((option) => option.dataset.template)).toEqual(['ux', 'api', 'audit', 'report', 'error', 'test']);
        act(() => { options[3].click(); });
        expect(promptElement().value).toBe('Prompt for report');
        expect(props.onSend).not.toHaveBeenCalled();
    });

    it('permission_picker_is_disabled_without_host', async () => {
        const props = await mountSurface();
        const permission = host.querySelector<HTMLSelectElement>('select[aria-label="Permission mode"]');
        expect(permission).not.toBeNull();
        expect(permission!.value).toBe('ask');
        const autonomous = Array.from(permission!.options).find((option) => option.textContent?.startsWith('Autonomous'));
        expect(autonomous?.disabled).toBe(true);
        act(() => { permission!.value = 'autonomous'; permission!.dispatchEvent(new Event('change', { bubbles: true })); });
        expect(props.onMode).not.toHaveBeenCalled();
        expect(permission!.value).toBe('ask');
    });

    it('environment_picker_shows_only_the_real_local_state', async () => {
        const props = await mountSurface();
        const environment = host.querySelector<HTMLSelectElement>('select[aria-label="Composer environment"]');
        expect(environment).not.toBeNull();
        expect(environment!.value).toBe('Local browser');
        const service = Array.from(environment!.options).find((option) => option.textContent?.includes('Harness service'));
        expect(service?.disabled).toBe(true);
        act(() => { environment!.value = 'harness'; environment!.dispatchEvent(new Event('change', { bubbles: true })); });
        expect(props.onEnvironment).not.toHaveBeenCalled();
    });

    it('model_choice_uses_the_existing_model_action', async () => {
        const props = await mountSurface();
        act(() => { button('Model: model-a').click(); });
        expect(props.onModel).toHaveBeenCalledOnce();
        expect(host.querySelectorAll('select[aria-label="Model"]')).toHaveLength(0);
    });

    it('exposes no queue, steer, or force controls before Task 8', async () => {
        const props = await mountSurface({ value: 'send me', onQueue: vi.fn(), onSteer: vi.fn(), onForce: vi.fn(), onStop: vi.fn() });
        const labels = Array.from(host.querySelectorAll('button')).map((node) => node.textContent);
        expect(labels).not.toContain('Queue');
        expect(labels).not.toContain('Steer');
        expect(labels).not.toContain('Force');
        typePrompt('send me');
        act(() => { button('Send').click(); });
        expect(props.onSend).toHaveBeenCalledWith('send me');
    });

    it('uses the existing model view action from AgentComposer', () => {
        const { uiApi } = mount();
        act(() => { button('Model: api-a').click(); });
        expect(uiApi.api.setView).toHaveBeenCalledWith('model');
    });
});
