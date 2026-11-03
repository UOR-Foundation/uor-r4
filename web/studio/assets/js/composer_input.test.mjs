import test from 'node:test';
import assert from 'node:assert/strict';
import { createComposerInput } from './composer_input.js';

function setup() {
    const textarea = new EventTarget();
    textarea.value = '';
    textarea.selectionStart = 0;
    textarea.selectionEnd = 0;
    textarea.setSelectionRange = (start, end) => {
        textarea.selectionStart = start;
        textarea.selectionEnd = end;
    };
    const files = [];
    createComposerInput({
        textarea,
        onFiles: (value) => files.push(...value),
    });
    return { textarea, files };
}

function paste(textarea, files, text = '') {
    const event = new Event('paste', { bubbles: true, cancelable: true });
    Object.defineProperty(event, 'clipboardData', {
        value: {
            files,
            getData: (type) => type === 'text/plain' ? text : '',
        },
    });
    textarea.dispatchEvent(event);
    return event;
}

test('mixed clipboard paste attaches the file and leaves its text native', () => {
    const { textarea, files } = setup();
    const file = { name: 'note.txt', type: 'text/plain', size: 8 };
    const event = paste(textarea, [file], 'visible text');
    assert.equal(event.defaultPrevented, false);
    assert.deepEqual(files, [file]);
});

test('file-only paste remains prevented while text-only paste stays native', () => {
    const { textarea, files } = setup();
    const file = { name: 'note.txt', type: 'text/plain', size: 8 };
    assert.equal(paste(textarea, [file]).defaultPrevented, true);
    assert.deepEqual(files, [file]);

    const textOnly = paste(textarea, [], 'text only');
    assert.equal(textOnly.defaultPrevented, false);
    assert.deepEqual(files, [file]);
});
