import { describe, expect, it, vi } from 'vitest';

import { saveActiveFile } from '../adapters/fileSave.js';
import { LifecycleStore } from '../state/lifecycle.js';
import { createMvmWorkspaceAdapter } from './workspaceAdapter.js';
import type { MvmClient } from './client.js';

function fixture() {
    const files = new Map([
        ['/workspace/a.c', { text: 'int main(void){return 0;}', revision: 4 }],
        ['/workspace/src/main.c', { text: 'int main(void){return 7;}', revision: 2 }],
    ]);
    const calls: Array<{ path: string; text: string; baseRevision?: number }> = [];
    const client = {
        list: vi.fn(async () => {
            return [...files.keys()].map((path) => ({
                path,
                kind: 'file' as const,
                sizeBytes: files.get(path)!.text.length,
            }));
        }),
        read: vi.fn(async (path: string) => {
            const file = files.get(path);
            if (!file) throw new Error(`not found: ${path}`);
            return { text: file.text, revision: file.revision };
        }),
        write: vi.fn(async (path: string, text: string, baseRevision?: number) => {
            calls.push({ path, text, baseRevision });
            const file = files.get(path);
            if (file && baseRevision !== file.revision) throw new Error('stale editor save');
            if (file) {
                file.text = text;
                file.revision += 1;
                return { path, revision: file.revision };
            }
            files.set(path, { text, revision: 1 });
            return { path, revision: 1 };
        }),
    } as unknown as MvmClient;
    return { client, files, calls };
}

describe('MVM workspace adapter', () => {
    it('mvm_file_tree_lists_mvm_workspace_not_host_tree', async () => {
        const { client } = fixture();
        const workspace = createMvmWorkspaceAdapter(client);
        const tree = await workspace.fileSource.list();
        expect(tree.nodes.map((node) => node.path)).toContain('/workspace/a.c');
        expect(tree.nodes.map((node) => node.path)).toContain('/workspace/src');
        expect(tree.nodes.map((node) => node.path)).not.toContain('/host/secret.txt');
        expect(client.list).toHaveBeenCalledWith('/workspace');
    });

    it('mvm_virtual_file_open_and_save_updates_worker', async () => {
        const { client, calls } = fixture();
        const workspace = createMvmWorkspaceAdapter(client);
        await workspace.api.openFile('/workspace/a.c');
        workspace.api.setActiveContent('int main(void){return 42;}');
        await workspace.api.saveActive();
        expect(calls).toEqual([{ path: '/workspace/a.c', text: 'int main(void){return 42;}', baseRevision: 4 }]);
        expect(workspace.api.activeFile()?.content).toBe('int main(void){return 42;}');
        expect(workspace.api.treeSummary().activePath).toBe('/workspace/a.c');
        expect(workspace.api.treeSummary().tabs[0]?.dirty).toBe(false);
    });

    it('mvm_editor_save_uses_active_path_save_id_and_revision', async () => {
        const { client, calls } = fixture();
        const workspace = createMvmWorkspaceAdapter(client);
        const store = new LifecycleStore();
        await workspace.api.openFile('/workspace/a.c');
        const { syncWorkspaceSummary } = await import('../adapters/workflow.js');
        syncWorkspaceSummary(store, workspace.api.treeSummary());
        const saved = await saveActiveFile(store, workspace.api, vi.fn());
        expect(saved).toBe(true);
        expect(calls[0]).toEqual({ path: '/workspace/a.c', text: 'int main(void){return 0;}', baseRevision: 4 });
        expect(store.getState().file.phase).toBe('saved');
        expect(store.getState().file.saveId).toBeNull();
        expect(store.getState().file.path).toBe('/workspace/a.c');
    });

    it('mvm_stale_editor_save_conflicts_with_terminal_write', async () => {
        const { client, files, calls } = fixture();
        const workspace = createMvmWorkspaceAdapter(client);
        await workspace.api.openFile('/workspace/a.c');
        workspace.api.setActiveContent('editor-change');
        files.get('/workspace/a.c')!.text = 'terminal-change';
        files.get('/workspace/a.c')!.revision += 1;
        const onError = vi.fn();
        const store = new LifecycleStore();
        const { syncWorkspaceSummary } = await import('../adapters/workflow.js');
        syncWorkspaceSummary(store, workspace.api.treeSummary());
        const saved = await saveActiveFile(store, workspace.api, onError);
        expect(saved).toBe(false);
        expect(onError).toHaveBeenCalledWith('stale editor save', 'file:save:/workspace/a.c');
        expect(calls).toHaveLength(1);
        expect(store.getState().file.phase).toBe('error');
    });

    it('mvm_static_unavailable_state_is_explicit', async () => {
        const client = { list: vi.fn(async () => { throw new Error('WASM unavailable'); }) } as unknown as MvmClient;
        const workspace = createMvmWorkspaceAdapter(client);
        await expect(workspace.fileSource.list()).rejects.toThrow('WASM unavailable');
        expect(workspace.api.saveAsCopy()).toBe(false);
        await expect(workspace.api.openFolder()).rejects.toThrow(/repository|folder|unsupported/i);
    });
});
