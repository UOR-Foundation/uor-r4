import type { FileTreeMeta } from '../components/FileTree.js';
import type { FileNode } from '../components/types.js';
import type { StudioFile, WorkspaceApi, WorkspaceSummary, WorkspaceTab } from '../adapters/workflow.js';
import { MvmClientError, type MvmClient } from './client.js';

export interface MvmFileSource {
    list(): Promise<{ nodes: FileNode[]; meta: FileTreeMeta }>;
    subscribe?(listener: () => void): () => void;
}

export interface MvmWorkspace {
    api: WorkspaceApi;
    fileSource: MvmFileSource;
}

interface WriteResult {
    path: string;
    revision: number;
}

const emptyMeta: FileTreeMeta = { truncated: false, truncatedReported: false, symlinksSkipped: 0, readErrors: 0 };

function basename(path: string): string {
    return path.split('/').pop() || path;
}

function directories(files: Array<{ path: string; kind: 'file'; sizeBytes: number }>): FileNode[] {
    const directories = new Set<string>(['/workspace']);
    for (const file of files) {
        const parts = file.path.slice('/workspace/'.length).split('/');
        for (let index = 1; index < parts.length; index += 1) {
            directories.add(`/workspace/${parts.slice(0, index).join('/')}`);
        }
    }
    const nodes = [...directories].map((path): FileNode => ({ path, name: basename(path), kind: 'directory' }));
    nodes.push(...files.map((file): FileNode => ({ path: file.path, name: basename(file.path), kind: 'file', sizeBytes: file.sizeBytes })));
    return nodes.sort((left, right) => left.path === right.path ? 0 : left.kind === right.kind ? left.path.localeCompare(right.path) : left.kind === 'directory' ? -1 : 1);
}

export function createMvmWorkspaceAdapter(client: MvmClient): MvmWorkspace {
    let active: StudioFile | null = null;
    let tabs: WorkspaceTab[] = [];
    const summary: WorkspaceSummary = { mode: 'directory', rootName: 'MVM sandbox', fileCount: 0, tabs, activePath: null };

    function refreshSummary(): void {
        summary.tabs = tabs;
        summary.activePath = active?.path ?? null;
        summary.fileCount = tabs.length;
    }

    function setContent(content: string): void {
        if (active === null) return;
        active.content = content;
        tabs = tabs.map((tab) => tab.path === active!.path ? { ...tab, dirty: true } : tab);
        refreshSummary();
    }

    const api: WorkspaceApi = {
        async openFolder(): Promise<void> {
            throw new MvmClientError('unsupported', 'Local folder selection is unavailable in the MVM sandbox; load a repository through the approved GitHub path.');
        },
        importFiles(): void {
            throw new MvmClientError('unsupported', 'File import is unavailable in the MVM sandbox; load a repository through the approved GitHub path.');
        },
        async openDemo(): Promise<void> {
            await client.write('/workspace/README.md', '# MVM sandbox\n\nA real RV64IMAC browser sandbox.\n');
            await api.openFile('/workspace/README.md');
        },
        async openFile(path: string): Promise<void> {
            const result = await client.read(path);
            active = { path, name: basename(path), content: result.text, mode: 'mvm', revision: result.revision };
            if (!tabs.some((tab) => tab.path === path)) tabs.push({ path, dirty: false, mode: 'mvm' });
            refreshSummary();
        },
        newFile(): void {
            const path = '/workspace/untitled.txt';
            active = { path, name: 'untitled.txt', content: '', mode: 'mvm', revision: 1 };
            if (!tabs.some((tab) => tab.path === path)) tabs.push({ path, dirty: true, mode: 'mvm' });
            refreshSummary();
        },
        closeTab(path: string): void {
            tabs = tabs.filter((tab) => tab.path !== path);
            if (active?.path === path) active = null;
            refreshSummary();
        },
        closeAll(): void {
            tabs = [];
            active = null;
            refreshSummary();
        },
        async saveActive(): Promise<boolean> {
            if (active === null) return false;
            const path = active.path;
            const result = await client.write(path, active.content, active.revision) as WriteResult;
            active = { ...active, revision: result.revision };
            tabs = tabs.map((tab) => tab.path === path ? { ...tab, dirty: false } : tab);
            refreshSummary();
            return true;
        },
        saveAsCopy(): boolean {
            return false;
        },
        insertText(text: string, name = 'untitled.txt'): boolean {
            if (active === null) {
                const path = `/workspace/${name}`;
                active = { path, name: basename(path), content: text, mode: 'mvm', revision: 1 };
                tabs.push({ path, dirty: true, mode: 'mvm' });
            } else {
                active.content += text;
                tabs = tabs.map((tab) => tab.path === active!.path ? { ...tab, dirty: true } : tab);
            }
            refreshSummary();
            return true;
        },
        setActiveContent: setContent,
        activeFile(): StudioFile | null {
            return active;
        },
        workspaceName(): string {
            return summary.rootName;
        },
        mode(): string {
            return summary.mode;
        },
        tabs(): WorkspaceTab[] {
            return tabs.map((tab) => ({ ...tab }));
        },
        treeSummary(): WorkspaceSummary {
            return { ...summary, tabs: summary.tabs.map((tab) => ({ ...tab })) };
        },
    };

    return {
        api,
        fileSource: {
            async list() {
                const files = await client.list('/workspace');
                return { nodes: directories(files), meta: emptyMeta };
            },
            subscribe: (listener) => client.onFilesChanged(listener),
        },
    };
}
