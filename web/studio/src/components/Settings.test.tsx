import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Settings } from './Settings.js';
import { makeUi } from './studioTestFixtures.js';

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('Settings', () => {
    it('updates appearance and editor preferences through the existing writer', () => {
        const ui = makeUi();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} />, host); });
        const theme = host.querySelector<HTMLSelectElement>('select[aria-label="Theme"]')!;
        act(() => { theme.value = 'light'; theme.dispatchEvent(new Event('change', { bubbles: true })); });
        const reduce = host.querySelector<HTMLInputElement>('input[aria-label="Reduce motion"]')!;
        act(() => { reduce.checked = true; reduce.dispatchEvent(new Event('change', { bubbles: true })); });
        const font = host.querySelector<HTMLInputElement>('input[aria-label="Editor font size"]')!;
        act(() => { font.value = '16'; font.dispatchEvent(new Event('input', { bubbles: true })); });
        const tab = host.querySelector<HTMLInputElement>('input[aria-label="Tab size"]')!;
        act(() => { tab.value = '2'; tab.dispatchEvent(new Event('input', { bubbles: true })); });
        const wrap = host.querySelector<HTMLInputElement>('input[aria-label="Word wrap"]')!;
        act(() => { wrap.checked = true; wrap.dispatchEvent(new Event('change', { bubbles: true })); });
        expect(ui.updateSettings).toHaveBeenCalledWith({ theme: 'light' });
        expect(ui.updateSettings).toHaveBeenCalledWith({ reduceMotion: true });
        expect(ui.updateSettings).toHaveBeenCalledWith({ fontSize: 16 });
        expect(ui.updateSettings).toHaveBeenCalledWith({ tabSize: 2 });
        expect(ui.updateSettings).toHaveBeenCalledWith({ wordWrap: true });
    });

    it('reports partial session clearing through the typed error channel', () => {
        const ui = makeUi();
        ui.clearSessions = vi.fn(() => ({ cleared: 1, total: 2 }));
        const onError = vi.fn();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} onError={onError} />, host); });
        act(() => { button('Clear sessions').click(); });
        act(() => { button('Confirm clear sessions').click(); });
        expect(onError).toHaveBeenCalledWith('Cleared 1 of 2 sessions; stop the current reply before deleting the rest.');
    });

    it('requires confirmation before clearing sessions or editor tabs', () => {
        const ui = makeUi();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} />, host); });
        act(() => { button('Clear sessions').click(); });
        expect(ui.clearSessions).not.toHaveBeenCalled();
        act(() => { button('Confirm clear sessions').click(); });
        expect(ui.clearSessions).toHaveBeenCalledOnce();
        act(() => { button('Clear editor tabs').click(); });
        expect(ui.clearEditor).not.toHaveBeenCalled();
        act(() => { button('Confirm clear editor tabs').click(); });
        expect(ui.clearEditor).toHaveBeenCalledOnce();
    });
});
