import { useEffect, useState } from 'preact/hooks';
import type { WorkspaceApi, WorkspaceSummary } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';

interface WorkspaceEditorProps {
    store: LifecycleStore;
    files: WorkspaceApi;
    onError(message: string): void;
}

export function WorkspaceEditor({ store, files, onError }: WorkspaceEditorProps) {
    const state = useLifecycle(store);
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);
    const report = (error: unknown) => onError(error instanceof Error ? error.message : String(error));

    function sync(summary: WorkspaceSummary) {
        if (summary.mode !== 'none' && store.getState().file.phase === 'no-folder') {
            store.dispatch({ type: 'folder/opened', path: summary.rootName });
        }
        if (summary.activePath) {
            if (store.getState().file.path !== summary.activePath) {
                store.dispatch({ type: 'file/opened', path: summary.activePath });
            }
            const activeTab = summary.tabs.find((tab) => tab.path === summary.activePath);
            if (activeTab?.dirty && !['dirty', 'saving'].includes(store.getState().file.phase)) {
                store.dispatch({ type: 'file/dirty' });
            }
        } else if (store.getState().file.path !== null) {
            store.dispatch({ type: 'file/closed' });
        }
    }

    useEffect(() => {
        const changed = (event: Event) => {
            sync((event as CustomEvent<WorkspaceSummary>).detail);
            refresh();
        };
        document.addEventListener('uor-files-changed', changed);
        document.addEventListener('uor-workspace-changed', changed);
        sync(files.treeSummary());
        return () => {
            document.removeEventListener('uor-files-changed', changed);
            document.removeEventListener('uor-workspace-changed', changed);
        };
    }, [files, store]);

    async function perform(action: () => Promise<unknown>) {
        try {
            await action();
            sync(files.treeSummary());
            refresh();
        } catch (error) {
            report(error);
        }
    }

    function immediate(action: () => unknown) {
        try {
            action();
            sync(files.treeSummary());
            refresh();
        } catch (error) {
            report(error);
        }
    }

    async function save() {
        if (!store.canSave()) return;
        store.dispatch({ type: 'file/saveStarted' });
        refresh();
        try {
            const saved = await files.saveActive();
            if (saved) {
                store.dispatch({ type: 'file/saveFinished' });
            } else {
                store.dispatch({ type: 'file/saveFailed', error: 'Save did not complete.' });
                onError('Save did not complete.');
            }
        } catch (error) {
            store.dispatch({ type: 'file/saveFailed', error: error instanceof Error ? error.message : String(error) });
            report(error);
        }
        sync(files.treeSummary());
        refresh();
    }

    const summary = files.treeSummary();
    const active = files.activeFile();
    const canSave = store.canSave();

    return (
        <section aria-label="Workspace editor">
            <h1>Workspace</h1>
            <p>{summary.rootName || 'No folder open'}</p>
            <p>File: {state.file.phase}{state.file.path ? ` · ${state.file.path}` : ' · No file open'}</p>
            {state.file.error && <p role="alert">{state.file.error}</p>}
            <div>
                <button type="button" onClick={() => void perform(files.openFolder)}>Open folder</button>
                <button type="button" onClick={() => immediate(files.importFiles)}>Import files</button>
                <button type="button" onClick={() => immediate(files.openDemo)}>Open demo</button>
                <button type="button" onClick={() => immediate(files.newFile)}>New file</button>
            </div>
            {summary.tabs.length > 0 && (
                <ul aria-label="Open editor tabs">
                    {summary.tabs.map((tab) => (
                        <li>
                            <button type="button" aria-current={summary.activePath === tab.path ? 'page' : undefined} onClick={() => void perform(() => files.openFile(tab.path))}>{`Open ${tab.path}`}</button>
                            <button type="button" aria-label={`Close ${tab.path}`} onClick={() => immediate(() => files.closeTab(tab.path))}>{`Close ${tab.path}`}</button>
                            {tab.dirty && <span> Unsaved</span>}
                        </li>
                    ))}
                </ul>
            )}
            <label>Editor
                <textarea
                    aria-label="File contents"
                    disabled={!active || state.file.phase === 'saving'}
                    value={active?.content ?? ''}
                    onInput={(event) => {
                        if (!active) return;
                        files.setActiveContent(event.currentTarget.value);
                        store.dispatch({ type: 'file/dirty' });
                        refresh();
                    }}
                />
            </label>
            <div>
                <button type="button" disabled={!canSave} onClick={() => void save()}>Save</button>
                <button type="button" disabled={!canSave} onClick={() => immediate(files.saveAsCopy)}>Save as</button>
            </div>
        </section>
    );
}
