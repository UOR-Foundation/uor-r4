import { describe, expect, it, vi } from 'vitest';
// @ts-expect-error The plain-JavaScript controller has no declaration file.
import { createComposerInput, readTextFiles } from '../../assets/js/composer_input.js';

function key(textarea: HTMLTextAreaElement, key: string, init: { shiftKey?: boolean; isComposing?: boolean; keyCode?: number } = {}) {
    const event = new KeyboardEvent('keydown', { key, shiftKey: init.shiftKey, bubbles: true, cancelable: true });
    Object.defineProperties(event, {
        isComposing: { value: Boolean(init.isComposing) },
        keyCode: { value: init.keyCode || 0 },
    });
    textarea.dispatchEvent(event);
    return event;
}

function setup(history: string[] = []) {
    document.body.innerHTML = '<textarea id="chat"></textarea><input id="files" type="file" multiple>';
    const textarea = document.querySelector<HTMLTextAreaElement>('#chat')!;
    const picker = document.querySelector<HTMLInputElement>('#files')!;
    const onSend = vi.fn(() => 1);
    const onFiles = vi.fn();
    const onHistoryChange = vi.fn();
    const controller = createComposerInput({
        textarea,
        picker,
        onSend,
        onFiles,
        getHistory: () => history,
        onHistoryChange,
    });
    return { textarea, picker, onSend, onFiles, onHistoryChange, controller };
}

describe('conversational composer keyboard and history', () => {
    it('sends on Enter, keeps Shift+Enter native, and never sends during IME composition', () => {
        const { textarea, onSend } = setup();
        textarea.value = 'hello';
        expect(key(textarea, 'Enter').defaultPrevented).toBe(true);
        expect(key(textarea, 'Enter', { shiftKey: true }).defaultPrevented).toBe(false);
        expect(key(textarea, 'Enter', { isComposing: true }).defaultPrevented).toBe(true);
        expect(key(textarea, 'Enter', { keyCode: 229 }).defaultPrevented).toBe(true);
        expect(onSend).toHaveBeenCalledTimes(1);
    });

    it('recalls complete raw prompts and restores the in-progress draft', () => {
        const long = 'x'.repeat(120001);
        const { textarea, onHistoryChange, controller } = setup();
        controller.recordSubmitted('first\nline');
        controller.recordSubmitted(long);
        textarea.value = 'draft';
        key(textarea, 'ArrowUp');
        expect(textarea.value).toBe(long);
        key(textarea, 'ArrowUp');
        expect(textarea.value).toBe('first\nline');
        key(textarea, 'ArrowDown');
        expect(textarea.value).toBe(long);
        key(textarea, 'ArrowDown');
        expect(textarea.value).toBe('draft');
        expect(onHistoryChange.mock.calls[0][0]).toEqual(['first\nline']);
        expect(onHistoryChange.mock.calls[1][0]).toEqual(['first\nline', long]);
    });

    it('moves history only at multiline boundaries and leaves history edits unsaved', () => {
        const { textarea, onHistoryChange, controller } = setup(['one\nfirst', 'two\nsecond']);
        textarea.value = 'draft\nline';
        textarea.setSelectionRange(0, 0);
        key(textarea, 'ArrowUp');
        expect(textarea.value).toBe('two\nsecond');
        expect(textarea.selectionStart).toBe('two\nsecond'.length);
        key(textarea, 'ArrowUp');
        expect(textarea.value).toBe('one\nfirst');
        key(textarea, 'ArrowDown');
        expect(textarea.value).toBe('two\nsecond');
        textarea.setSelectionRange(0, 0);
        expect(key(textarea, 'ArrowDown').defaultPrevented).toBe(false);
        expect(textarea.value).toBe('two\nsecond');
        expect(onHistoryChange).not.toHaveBeenCalled();
        controller.destroy();
    });
});

describe('conversational composer file channels', () => {
    it('passes pasted files to the attachment callback without sending or swallowing text paste', () => {
        const { textarea, onSend, onFiles } = setup();
        const file = new File(['contents'], 'note.txt', { type: 'text/plain' });
        const paste = new Event('paste', { bubbles: true, cancelable: true });
        Object.defineProperty(paste, 'clipboardData', { value: { files: [file] } });
        textarea.dispatchEvent(paste);
        expect(paste.defaultPrevented).toBe(true);
        expect(onFiles).toHaveBeenCalledWith([file]);
        expect(onSend).not.toHaveBeenCalled();
        const textPaste = new Event('paste', { bubbles: true, cancelable: true });
        Object.defineProperty(textPaste, 'clipboardData', { value: { files: [], getData: () => 'x'.repeat(120001) } });
        textarea.dispatchEvent(textPaste);
        expect(textPaste.defaultPrevented).toBe(false);
        expect(onFiles).toHaveBeenCalledTimes(1);
    });

    it('passes dropped files to the attachment callback without sending', () => {
        const { textarea, onSend, onFiles } = setup();
        const file = new File(['contents'], 'note.txt', { type: 'text/plain' });
        const drop = new Event('drop', { bubbles: true, cancelable: true });
        Object.defineProperty(drop, 'dataTransfer', { value: { files: [file] } });
        textarea.dispatchEvent(drop);
        expect(drop.defaultPrevented).toBe(true);
        expect(onFiles).toHaveBeenCalledWith([file]);
        expect(onSend).not.toHaveBeenCalled();
    });

    it('passes multiple picked files to the attachment callback', () => {
        const { picker, onFiles } = setup();
        const files = [new File(['a'], 'a.txt'), new File(['b'], 'b.md')];
        Object.defineProperty(picker, 'files', { value: files, configurable: true });
        picker.dispatchEvent(new Event('change', { bubbles: true }));
        expect(onFiles).toHaveBeenCalledWith(files);
    });
});

describe('text attachment reading', () => {
    it('preserves a complete long text file without the old silent slice', async () => {
        const content = 'x'.repeat(120001);
        const result = await readTextFiles([new File([content], 'long.txt', { type: 'text/plain' })], { maxFileBytes: 120100, maxTotalBytes: 120100 });
        expect(result.errors).toEqual([]);
        expect(result.attachments).toHaveLength(1);
        expect(result.attachments[0].content).toBe(content);
        expect(result.attachments[0]).toMatchObject({ name: 'long.txt', kind: 'text', source: 'file', size: 120001 });
    });

    it('refuses per-file and total-size overflow without partial attachments', async () => {
        const perFile = await readTextFiles([new File(['123456'], 'big.txt')], { maxFileBytes: 5, maxTotalBytes: 20 });
        expect(perFile.attachments).toEqual([]);
        expect(perFile.errors[0]).toMatch(/big.txt.*5-byte per-file.*6 bytes/);
        const total = await readTextFiles([new File(['1234'], 'a.txt'), new File(['5678'], 'b.txt')], { maxFileBytes: 5, maxTotalBytes: 5 });
        expect(total.attachments.map((item: { name: string }) => item.name)).toEqual(['a.txt']);
        expect(total.errors[0]).toMatch(/b.txt.*total/i);
    });

    it('refuses binary files and text with NUL bytes loudly', async () => {
        const binary = await readTextFiles([new File([new Uint8Array([1, 2, 3])], 'image.bin', { type: 'application/octet-stream' })]);
        expect(binary.attachments).toEqual([]);
        expect(binary.errors[0]).toMatch(/binary|text/i);
        const nul = await readTextFiles([new File(['a\0b'], 'fake.txt', { type: 'text/plain' })]);
        expect(nul.attachments).toEqual([]);
        expect(nul.errors[0]).toMatch(/binary|NUL/i);
    });
});
