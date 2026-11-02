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
});
