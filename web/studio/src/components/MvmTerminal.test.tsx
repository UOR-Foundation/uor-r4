import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { MvmClient } from '../mvm/client.js';
import { MvmTerminal } from './MvmTerminal.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function mount(overrides: Partial<MvmClient> = {}) {
    const client = {
        status: vi.fn(async () => ({ architecture: 'riscv64-rv64imac', guestCid: 3, dirtyCowSectorCount: 0, cowStateSha256: 'a'.repeat(64) })),
        exec: vi.fn(async () => ({ exitCode: 0, stdout: 'ok\n', stderr: '' })),
        reset: vi.fn(async () => undefined),
        dispose: vi.fn(async () => undefined),
        history: vi.fn(() => [{ cmd: 'mvm status', cwd: '/workspace', persisted: false }]),
        ...overrides,
    } as unknown as MvmClient;
    act(() => { render(<MvmTerminal client={client} />, host); });
    return client;
}

function input(): HTMLInputElement {
    const found = host.querySelector<HTMLInputElement>('input[aria-label="MVM command"]');
    if (!found) throw new Error('missing command input');
    return found;
}

describe('MvmTerminal', () => {
    it('mvm_terminal_runs_real_commands_and_memory_history', async () => {
        const client = mount();
        await vi.waitFor(() => expect(host.textContent).toContain('riscv64-rv64imac'));
        const command = input();
        command.value = 'pwd';
        act(() => { command.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => { host.querySelector('form')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        await vi.waitFor(() => expect(host.textContent).toContain('ok'));
        expect(client.exec).toHaveBeenCalledWith('pwd', '/workspace');
        act(() => {
            const historyInput = host.querySelector<HTMLInputElement>('input[aria-label="MVM command"]')!;
            historyInput.value = 'history';
            historyInput.dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        await vi.waitFor(() => expect(host.textContent).toContain('mvm status'));
        expect(client.history).toHaveBeenCalled();
        act(() => { host.querySelector<HTMLButtonElement>('button[aria-label="Clear terminal"]')!.click(); });
        expect(host.querySelector('pre')!.textContent).toBe('');
        act(() => { host.querySelector<HTMLButtonElement>('button[aria-label="Reset sandbox"]')!.click(); });
        await vi.waitFor(() => expect(client.reset).toHaveBeenCalled());
    });

    it('mvm_terminal_unmount_preserves_the_application_owned_client', async () => {
        const client = mount();
        act(() => { render(null, host); });
        expect(client.dispose).not.toHaveBeenCalled();
        await client.status();
        expect(client.status).toHaveBeenCalledTimes(2);
    });

    it('mvm_static_unavailable_state_is_explicit', async () => {
        mount({ status: vi.fn(async () => { throw new Error('WebAssembly unavailable'); }) });
        await vi.waitFor(() => expect(host.textContent).toContain('MVM unavailable: WebAssembly unavailable'));
        expect(input().disabled).toBe(true);
        expect(host.textContent).not.toContain('MVM_AGENT_ONLINE');
    });

    it('mvm_terminal_ime_enter_does_not_submit_until_composition_end', async () => {
        const client = mount();
        const command = input();
        command.value = 'pwd';
        act(() => { command.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => {
            command.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }));
            command.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
        });
        expect(client.exec).not.toHaveBeenCalled();
        act(() => { host.querySelector('form')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        expect(client.exec).not.toHaveBeenCalled();
        act(() => { command.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true })); });
        act(() => { command.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })); });
        await vi.waitFor(() => expect(client.exec).toHaveBeenCalledWith('pwd', '/workspace'));
    });

    it('mvm_terminal_native_ime_keycode_does_not_submit', () => {
        const client = mount();
        const command = input();
        command.value = 'pwd';
        act(() => { command.dispatchEvent(new Event('input', { bubbles: true })); });
        const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true });
        Object.defineProperties(event, { keyCode: { value: 229 }, isComposing: { value: false } });
        act(() => { input().dispatchEvent(event); });
        expect(client.exec).not.toHaveBeenCalled();
    });
});
