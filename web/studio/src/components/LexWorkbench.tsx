import { useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import type { OperationGate } from '../adapters/operations.js';
import type { StudioSettings } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import type { MvmClient } from '../mvm/client.js';
import type { MvmWorkspace } from '../mvm/workspaceAdapter.js';
import { WorkspaceEditor } from './WorkspaceEditor.js';
import { MvmFileExplorer } from './MvmFileExplorer.js';
import { MvmTerminal } from './MvmTerminal.js';

interface LexWorkbenchProps {
    store: LifecycleStore;
    workspace: MvmWorkspace;
    client: MvmClient;
    operations: OperationGate;
    onError(message: string, operation: string): void;
    onSuccess(operation: string): void;
    settings?: StudioSettings;
    assistant: ComponentChildren;
}

export function LexWorkbench({ store, workspace, client, operations, onError, onSuccess, settings, assistant }: LexWorkbenchProps) {
    const [explorerOpen, setExplorerOpen] = useState(() => window.innerWidth >= 1024);
    const [assistantOpen, setAssistantOpen] = useState(() => window.innerWidth >= 1024);
    const [terminalOpen, setTerminalOpen] = useState(true);
    const activePath = workspace.api.treeSummary().activePath;

    function openPath(path: string): void {
        void operations.run(
            `workspace:open:${path}`,
            (async () => { await workspace.api.openFile(path); })(),
            (completed) => { onSuccess(completed); },
            (failure) => { onError(failure instanceof Error ? failure.message : String(failure), `workspace:open:${path}`); },
        );
    }

    return <div class="lex-workbench" data-testid="lex-workbench">
        <nav class="lex-workbench-toolbar" aria-label="Workbench panes">
            <button type="button" aria-expanded={explorerOpen} onClick={() => setExplorerOpen((open) => !open)}>Explorer</button>
            <button type="button" aria-expanded={assistantOpen} onClick={() => setAssistantOpen((open) => !open)}>Assistant</button>
            <button type="button" aria-expanded={terminalOpen} onClick={() => setTerminalOpen((open) => !open)}>Terminal</button>
        </nav>
        <div class="lex-workbench-grid">
            <MvmFileExplorer source={workspace.fileSource} activePath={activePath} onOpen={openPath} onError={onError} open={explorerOpen} />
            <section class="lex-pane lex-editor" data-pane="editor">
                <WorkspaceEditor store={store} files={workspace.api} operations={operations} onError={onError} onSuccess={onSuccess} settings={settings} fileSource={workspace.fileSource} showFileTree={false} showHeader={false} showActions showSaveAs={false} />
            </section>
            <aside class="lex-pane lex-assistant" data-pane="assistant" data-open={assistantOpen}>{assistant}</aside>
            <section class="lex-pane lex-terminal" data-pane="terminal" data-open={terminalOpen}>
                <MvmTerminal client={client} />
            </section>
        </div>
    </div>;
}
