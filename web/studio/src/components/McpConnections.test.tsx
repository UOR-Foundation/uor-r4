import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Attachment } from '../adapters/workflow.js';
import { makeUi } from './studioTestFixtures.js';
import { McpConnections } from './McpConnections.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); window.localStorage.clear(); vi.restoreAllMocks(); });

function ui() {
    const attachments: Attachment[] = [];
    const api = makeUi();
    api.getAttachments = vi.fn(() => attachments);
    api.addAttachment = vi.fn((entry: Attachment) => { attachments.push(entry); });
    return { api, attachments };
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

function input(label: string, value: string) {
    const field = host.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;
    act(() => { field.value = value; field.dispatchEvent(new Event('input', { bubbles: true })); });
}

function args(value: string) {
    const field = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Tool arguments as JSON"]')!;
    act(() => { field.value = value; field.dispatchEvent(new Event('input', { bubbles: true })); });
}

describe('McpConnections', () => {
    it('connects, lists, calls, and attaches exact results with inert text and memory-only credentials', async () => {
        const result = { content: [{ type: 'text', text: '<img src=x onerror=window.__mcpMarkupExecuted=true>' }] };
        const session = {
            listTools: vi.fn(async () => [{ name: 'read_file', description: '<img src=x onerror=window.__mcpMarkupExecuted=true>' }]),
            callTool: vi.fn(async () => result),
            disconnect: vi.fn(async () => undefined),
        };
        const connectRemote = vi.fn(async () => session);
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        expect(connectRemote).not.toHaveBeenCalled();
        expect(session.listTools).not.toHaveBeenCalled();
        expect(button('Attach result').disabled).toBe(true);
        input('MCP server URL', 'http://127.0.0.1:9000/mcp');
        input('MCP credential', 'memory-only');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        expect(connectRemote).toHaveBeenCalledWith({ url: 'http://127.0.0.1:9000/mcp', credential: 'memory-only' });
        expect(session.listTools).toHaveBeenCalledOnce();
        expect((document.getElementById('mcpCredentialInput') as HTMLInputElement).value).toBe('');
        expect(document.querySelector('.mcp-tool-button img')).toBeNull();
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('<img'));
        expect(session.callTool).toHaveBeenCalledWith('read_file', {});
        expect(document.querySelector('#mcpResultOutput img')).toBeNull();
        expect((window as unknown as { __mcpMarkupExecuted?: boolean }).__mcpMarkupExecuted).toBeUndefined();
        await act(async () => { button('Attach result').click(); await Promise.resolve(); });
        expect(api.addAttachment).toHaveBeenCalledWith({ name: 'mcp-read_file.json', kind: 'text', content: JSON.stringify(result, null, 2), source: 'mcp' });
        expect(window.localStorage.getItem('uor.studio.mcp.credential')).toBeNull();
    });

    it('refuses invalid JSON and non-object arguments without calling or attaching', async () => {
        const session = { listTools: vi.fn(async () => [{ name: 'read_file', description: 'Read' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => session)} />, host); });
        input('MCP server URL', 'http://127.0.0.1:9000/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        args('[]');
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        expect(document.getElementById('mcpResultOutput')?.textContent).toContain('Tool arguments must be a JSON object.');
        expect(session.callTool).not.toHaveBeenCalled();
        expect(api.addAttachment).not.toHaveBeenCalled();
        expect(button('Attach result').disabled).toBe(true);
    });

    it('disconnects the previous session after a successful replacement', async () => {
        const first = { listTools: vi.fn(async () => [{ name: 'first', description: 'First' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const second = { listTools: vi.fn(async () => [{ name: 'second', description: 'Second' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const connectRemote = vi.fn()
            .mockImplementationOnce(async () => first)
            .mockImplementationOnce(async () => second);
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://first.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        input('MCP server URL', 'http://second.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('second'));
        expect(first.disconnect).toHaveBeenCalledOnce();
    });

    it('preserves the working session and tools when a replacement fails', async () => {
        const first = { listTools: vi.fn(async () => [{ name: 'first', description: 'First' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const connectRemote = vi.fn()
            .mockImplementationOnce(async () => first)
            .mockImplementationOnce(async () => { throw new Error('replacement failed'); });
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://first.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        input('MCP server URL', 'http://bad.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('replacement failed'));
        expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('first');
        expect(first.disconnect).not.toHaveBeenCalled();
    });

    it('disconnects a stale candidate on unmount without listing or attaching it', async () => {
        let resolveCandidate!: () => void;
        const candidate = { listTools: vi.fn(async () => [{ name: 'late', description: 'Late' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const connectRemote = vi.fn(() => new Promise<typeof candidate>((resolve) => { resolveCandidate = () => resolve(candidate); }));
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://late.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        act(() => { render(null, host); });
        await act(async () => { resolveCandidate(); await Promise.resolve(); });
        expect(candidate.disconnect).toHaveBeenCalledOnce();
        expect(candidate.listTools).not.toHaveBeenCalled();
        expect(api.addAttachment).not.toHaveBeenCalled();
    });

    it('reports connection cancellation without listing tools', async () => {
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => null)} />, host); });
        input('MCP server URL', 'http://cancel.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('cancelled'));
        expect(document.querySelector('.mcp-tool-button')).toBeNull();
        expect(button('Attach result').disabled).toBe(true);
    });
});
