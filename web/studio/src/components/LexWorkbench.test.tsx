import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createOperationGate } from '../adapters/operations.js';
import { LifecycleStore } from '../state/lifecycle.js';
import { createMvmWorkspaceAdapter } from '../mvm/workspaceAdapter.js';
import type { MvmClient } from '../mvm/client.js';
import { LexWorkbench } from './LexWorkbench.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function client(): MvmClient {
    return {
        status: vi.fn(async () => ({ architecture: 'riscv64-rv64imac', guestCid: 3, dirtyCowSectorCount: 0, cowStateSha256: 'a'.repeat(64) })),
        list: vi.fn(async () => [{ path: '/workspace/src/main.c', kind: 'file', sizeBytes: 24 }]),
        read: vi.fn(async () => ({ text: 'int main(void){return 0;}', revision: 1 })),
        write: vi.fn(async () => ({ path: '/workspace/src/main.c', revision: 2 })),
        exec: vi.fn(async () => ({ exitCode: 0, stdout: 'ok\n', stderr: '' })),
        reset: vi.fn(async () => undefined),
        dispose: vi.fn(async () => undefined),
        history: vi.fn(() => []),
        onFilesChanged: vi.fn(() => () => undefined),
    } as unknown as MvmClient;
}

function mount() {
    const mvm = client();
    const workspace = createMvmWorkspaceAdapter(mvm);
    act(() => {
        render(<LexWorkbench
            store={new LifecycleStore()}
            workspace={workspace}
            client={mvm}
            operations={createOperationGate()}
            onError={vi.fn()}
            onSuccess={vi.fn()}
            assistant={<div>Assistant pane</div>}
        />, host);
    });
    return { mvm, workspace };
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('LexWorkbench', () => {
    it('lexworkbench_has_three_panes_and_real_bottom_terminal', async () => {
        const { mvm } = mount();
        expect(host.querySelector('[data-pane="explorer"]')).not.toBeNull();
        expect(host.querySelector('[data-pane="editor"]')).not.toBeNull();
        expect(host.querySelector('[data-pane="assistant"]')).not.toBeNull();
        expect(host.querySelector('[data-pane="terminal"]')).not.toBeNull();
        expect(host.querySelector('[data-testid="mvm-terminal"]')).not.toBeNull();
        await vi.waitFor(() => expect(host.textContent).toContain('riscv64-rv64imac'));
        expect(mvm.status).toHaveBeenCalled();
        expect(host.textContent).toContain('Assistant pane');
    });

    it('mvm_mobile_drawers_preserve_editor_and_terminal', () => {
        mount();
        const explorer = button('Explorer');
        const assistant = button('Assistant');
        act(() => { explorer.click(); });
        expect(host.querySelector('[data-pane="explorer"]')!.getAttribute('data-open')).toBe('false');
        expect(host.querySelector('[data-pane="editor"]')).not.toBeNull();
        expect(host.querySelector('[data-pane="terminal"]')).not.toBeNull();
        act(() => { assistant.click(); });
        expect(host.querySelector('[data-pane="assistant"]')!.getAttribute('data-open')).toBe('false');
        act(() => { explorer.click(); assistant.click(); });
        expect(host.querySelector('[data-pane="explorer"]')!.getAttribute('data-open')).toBe('true');
        expect(host.querySelector('[data-pane="assistant"]')!.getAttribute('data-open')).toBe('true');
    });
});
