import { useEffect, useMemo, useState } from 'preact/hooks';
import type { ModelSourceAdapter } from '../adapters/modelSourceAdapter.js';
import type { ModelSourcesApi } from '../../assets/js/model_sources.js';
import { createGenerationAdapter } from '../adapters/generation.js';
import { saveActiveFile } from '../adapters/fileSave.js';
import { syncWorkspaceSummary } from '../adapters/workflow.js';
import type { SessionApi, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import { classifyEndpoint } from '../state/lifecycle.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { AppShell } from './AppShell.js';
import { AgentComposer } from './AgentComposer.js';
import { CommandPalette } from './CommandPalette.js';
import { Guide } from './Guide.js';
import { Inspector } from './Inspector.js';
import { ModelPicker } from './ModelPicker.js';
import { SessionList } from './SessionList.js';
import { Settings } from './Settings.js';
import { WorkspaceEditor } from './WorkspaceEditor.js';
import type { PaletteCommand, ShellStatus, StudioSection } from './types.js';

interface StudioAppProps {
    store: LifecycleStore;
    adapter: ModelSourceAdapter;
    sources: ModelSourcesApi;
    files: WorkspaceApi;
    sessions: SessionApi;
    ui: StudioUiApi;
    download?(name: string, content: string): void;
}

function browserDownload(name: string, content: string) {
    const url = URL.createObjectURL(new Blob([content], { type: 'text/plain;charset=utf-8' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = name;
    document.body.appendChild(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function StudioApp({ store, adapter, sources, files, sessions, ui, download = browserDownload }: StudioAppProps) {
    const state = useLifecycle(store);
    const [activeSection, setActiveSection] = useState<StudioSection>('agent');
    const [paletteOpen, setPaletteOpen] = useState(false);
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [inspectorOpen, setInspectorOpen] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [sessionRevision, setSessionRevision] = useState(0);
    const generation = useMemo(() => createGenerationAdapter(store, sessions), [store, sessions]);
    useEffect(() => () => generation.dispose(), [generation]);
    useEffect(() => {
        const viewChanged = (event: Event) => {
            const view = (event as CustomEvent<{ view?: string }>).detail?.view;
            if (view === 'chat') setActiveSection('agent');
            if (view === 'model') setActiveSection('models');
            if (view === 'workspace' || view === 'editor' || view === 'files') setActiveSection('workspace');
            if (view === 'guide') setActiveSection('guide');
        };
        const filesChanged = (event: Event) => {
            syncWorkspaceSummary(store, (event as CustomEvent<Parameters<typeof syncWorkspaceSummary>[1]>).detail);
        };
        document.addEventListener('uor-view-changed', viewChanged);
        document.addEventListener('uor-files-changed', filesChanged);
        return () => {
            document.removeEventListener('uor-view-changed', viewChanged);
            document.removeEventListener('uor-files-changed', filesChanged);
        };
    }, [store]);

    const selected = state.selectedSource;
    const activeGeneration = state.session.activeGeneration;
    let privacy: string = activeGeneration?.privacy || 'localWASM';
    if (!activeGeneration && selected?.kind === 'api') {
        if (!selected.endpoint) privacy = 'Invalid API endpoint';
        else {
            try { privacy = classifyEndpoint(selected.endpoint); } catch { privacy = 'Invalid API endpoint'; }
        }
    } else if (!activeGeneration && !selected && state.session.lastGeneration) {
        privacy = `Last generation: ${state.session.lastGeneration.privacy}`;
    }
    const status: ShellStatus = {
        source: selected ? `${selected.kind}: ${selected.id}` : 'No source selected',
        artifact: `Artifact: ${state.artifact.phase}`,
        privacy,
        file: `File: ${state.file.phase}`,
        session: `Session: ${state.session.phase}`,
    };
    const report = (message: string) => setError(message);
    const commands: PaletteCommand[] = [
        { id: 'agent', group: 'Navigation', label: 'Go to Agent', description: 'Chat and context', run: () => setActiveSection('agent') },
        { id: 'workspace', group: 'Navigation', label: 'Go to Workspace', description: 'Files and editor', run: () => setActiveSection('workspace') },
        { id: 'models', group: 'Navigation', label: 'Go to Models', description: 'Artifacts and providers', run: () => setActiveSection('models') },
        { id: 'guide', group: 'Navigation', label: 'Go to Guide', description: 'Workflows and limits', run: () => setActiveSection('guide') },
        { id: 'new-session', group: 'Sessions', label: 'New session', description: 'Start a clean thread', run: () => {
            if (sessions.newSession()) {
                store.dispatch({ type: 'session/activated' });
                setSessionRevision((value) => value + 1);
            } else {
                report('Could not create a session.');
            }
        } },
        { id: 'open-folder', group: 'Workspace', label: 'Open folder', description: 'Use the existing folder picker', run: () => { files.openFolder().catch((reason) => report(reason instanceof Error ? reason.message : String(reason))); } },
        { id: 'save', group: 'Workspace', label: 'Save file', description: 'Save through the existing file API', shortcut: 'Ctrl+S', disabled: !store.canSave(), run: () => { void saveActiveFile(store, files, report); } },
        { id: 'save-as', group: 'Workspace', label: 'Save as', description: 'Download a copy', disabled: !store.canSave(), run: () => { files.saveAsCopy(); } },
        { id: 'load-demo', group: 'Models', label: 'Load bundled demo', description: 'Load the synthetic local artifact', run: () => { adapter.loadBundled().catch((reason) => report(reason instanceof Error ? reason.message : String(reason))); } },
        { id: 'inspector', group: 'Interface', label: 'Toggle inspector', description: 'Show or hide details', run: () => setInspectorOpen((value) => !value) },
        { id: 'settings', group: 'Interface', label: 'Open settings', description: 'Appearance, editor, and data', run: () => setSettingsOpen(true) },
        { id: 'help', group: 'Help', label: 'Open guide', description: 'Read workflows and model limits', run: () => setActiveSection('guide') },
    ];

    return (
        <AppShell
            activeSection={activeSection}
            onSection={setActiveSection}
            status={status}
            onPalette={() => setPaletteOpen(true)}
            onSettings={() => setSettingsOpen(true)}
            onInspector={() => setInspectorOpen((value) => !value)}
            inspectorOpen={inspectorOpen}
            sessions={<SessionList store={store} sessions={sessions} download={download} onError={report} onChange={() => setSessionRevision((value) => value + 1)} />}
        >
            {error && <p role="alert">{error}</p>}
            {activeSection === 'agent' && (
                <AgentComposer
                    key={`${sessions.activeSessionId() || 'new'}:${sessionRevision}`}
                    store={store}
                    sessions={sessions}
                    files={files}
                    ui={ui}
                    generation={generation}
                    copy={(text) => {
                        if (navigator.clipboard) return navigator.clipboard.writeText(text);
                        ui.toast('Clipboard unavailable.', 'error');
                    }}
                    download={download}
                />
            )}
            {activeSection === 'workspace' && <WorkspaceEditor store={store} files={files} onError={report} />}
            {activeSection === 'models' && <ModelPicker store={store} adapter={adapter} sources={sources} onError={report} />}
            {activeSection === 'guide' && <Guide />}
            <Inspector store={store} ui={ui} open={inspectorOpen} onClose={() => setInspectorOpen(false)} />
            {paletteOpen && <CommandPalette commands={commands} onClose={() => setPaletteOpen(false)} />}
            {settingsOpen && <Settings ui={ui} onClose={() => setSettingsOpen(false)} onError={report} onChange={(kind) => {
                if (kind === 'editor') store.dispatch({ type: 'file/closed' });
                else {
                    setSessionRevision((value) => value + 1);
                    store.dispatch({ type: 'session/activated' });
                }
            }} />}
        </AppShell>
    );
}
