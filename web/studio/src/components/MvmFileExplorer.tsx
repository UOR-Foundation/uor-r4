import { useEffect, useRef, useState } from 'preact/hooks';
import type { FileTreeMeta } from './FileTree.js';
import { FileTree } from './FileTree.js';
import type { FileNode } from './types.js';
import type { MvmFileSource } from '../mvm/workspaceAdapter.js';

interface MvmFileExplorerProps {
    source: MvmFileSource;
    activePath: string | null;
    onOpen(path: string): void;
    onError(message: string, operation: string): void;
    open?: boolean;
}

interface TreeState {
    phase: 'loading' | 'ready' | 'error';
    nodes: FileNode[];
    meta: FileTreeMeta;
    error: string | null;
}

const emptyMeta: FileTreeMeta = { truncated: false, truncatedReported: false, symlinksSkipped: 0, readErrors: 0 };

export function MvmFileExplorer({ source, activePath, onOpen, onError, open = true }: MvmFileExplorerProps) {
    const mounted = useRef(true);
    const version = useRef(0);
    const searchRef = useRef<HTMLInputElement>(null);
    const [tree, setTree] = useState<TreeState>({ phase: 'loading', nodes: [], meta: emptyMeta, error: null });

    useEffect(() => {
        mounted.current = true;
        const load = async () => {
            const current = ++version.current;
            setTree((tree) => ({ ...tree, phase: 'loading', error: null }));
            try {
                const result = await source.list();
                if (!mounted.current || current !== version.current) return;
                setTree({ phase: 'ready', nodes: result.nodes, meta: result.meta, error: null });
            } catch (error) {
                if (!mounted.current || current !== version.current) return;
                const message = error instanceof Error ? error.message : String(error);
                setTree({ phase: 'error', nodes: [], meta: emptyMeta, error: message });
                onError(message, 'mvm:file-tree');
            }
        };
        void load();
        const reload = () => { void load(); };
        document.addEventListener('uor-files-changed', reload);
        document.addEventListener('uor-workspace-changed', reload);
        const unsubscribe = source.subscribe?.(reload);
        return () => {
            mounted.current = false;
            unsubscribe?.();
            document.removeEventListener('uor-files-changed', reload);
            document.removeEventListener('uor-workspace-changed', reload);
        };
    }, [source]);

    return <section aria-label="Repository explorer" class="mvm-file-explorer" data-pane="explorer" data-open={open}>
        <header class="lex-pane-heading"><div><h2>Explorer</h2><p>MVM sandbox</p></div></header>
        <FileTree nodes={tree.nodes} meta={tree.meta} activePath={activePath} phase={tree.phase} error={tree.error} onOpen={onOpen} searchRef={searchRef} />
    </section>;
}
