import { useEffect, useRef, useState } from 'preact/hooks';
import { readTextFiles } from '../adapters/composerInput.js';
import type { ComposerInputController } from '../adapters/composerInput.js';
import { GENERATION_FINISHED_EVENT } from '../adapters/generation.js';
import type { GenerationAdapter } from '../adapters/generation.js';
import type { Attachment, SessionApi, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { AgentTranscript } from './AgentTranscript.js';
import { attachmentSize, draftFromStoredAttachment, planAttachmentBatch } from './ComposerAttachments.js';
import type { AttachmentDraft } from './ComposerAttachments.js';
import { MAX_DOCUMENT_BYTES, MAX_TOTAL_ATTACHMENT_BYTES } from './ComposerAttachments.js';
import { LOCAL_TEMPLATES } from './ComposerMenu.js';
import { ComposerSurface } from './ComposerSurface.js';
import type { PermissionMode } from './PermissionPicker.js';
import { Icon } from './ui/Icon.js';

interface AgentComposerProps {
    store: LifecycleStore;
    sessions: SessionApi;
    files: WorkspaceApi;
    ui: StudioUiApi;
    generation: GenerationAdapter;
    copy(text: string): void | Promise<void>;
    download(name: string, content: string): void;
    draft?: string;
    pasted?: string;
    onDraftChange?(value: string): void;
    onPastedChange?(value: string): void;
}

let insertSequence = 0;
let imageSequence = 0;

function nextInsertName(): string {
    insertSequence += 1;
    return `agent-message-${Date.now()}-${insertSequence}.md`;
}

function revokePreview(draft: AttachmentDraft): void {
    if (draft.previewUrl && typeof URL.revokeObjectURL === 'function') URL.revokeObjectURL(draft.previewUrl);
}

export function AgentComposer({ store, sessions, files, ui, generation, copy, download, draft: controlledDraft, pasted: controlledPasted, onDraftChange, onPastedChange }: AgentComposerProps) {
    const state = useLifecycle(store);
    const [pastedState, setPastedState] = useState('');
    const pasted = controlledPasted ?? pastedState;
    const setPasted = (value: string) => { setPastedState(value); onPastedChange?.(value); };
    const [visibleDraft, setVisibleDraft] = useState(controlledDraft ?? '');
    const [imageDrafts, setImageDrafts] = useState<AttachmentDraft[]>([]);
    const [permissionMode, setPermissionMode] = useState<PermissionMode>('ask');
    const initialDraftRef = useRef(controlledDraft ?? '');
    const previousControlledDraft = useRef(controlledDraft);
    const ownControlledDraft = useRef<string | null>(null);
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);
    const controllerRef = useRef<ComposerInputController | null>(null);
    const imageDraftsRef = useRef<AttachmentDraft[]>([]);
    const mountedRef = useRef(true);

    function notifyDraftChange(value: string) {
        ownControlledDraft.current = value;
        onDraftChange?.(value);
    }

    function setDraft(value: string) {
        if (controllerRef.current) controllerRef.current.reset(value);
        setVisibleDraft(value);
        notifyDraftChange(value);
    }

    useEffect(() => {
        const changed = () => refresh();
        document.addEventListener(GENERATION_FINISHED_EVENT, changed);
        document.addEventListener('uor-attachments-changed', changed);
        return () => {
            document.removeEventListener(GENERATION_FINISHED_EVENT, changed);
            document.removeEventListener('uor-attachments-changed', changed);
        };
    }, []);

    useEffect(() => () => {
        mountedRef.current = false;
        for (const draft of imageDraftsRef.current) revokePreview(draft);
    }, []);

    const selected = state.selectedSource;
    const activeGeneration = state.session.activeGeneration;
    const residentMatches = selected?.kind === 'artifact'
        && state.artifact.phase === 'ready'
        && state.residentArtifact?.sourceId === selected.id;
    const prediction = !activeGeneration && residentMatches
        && state.residentArtifact?.canGenerate === false
        && state.residentArtifact?.canPredict === true;
    const generating = state.session.phase === 'generating';
    const messages = sessions.getConversationMessages();
    const storedAttachments = ui.getAttachments();
    const attachments = [...storedAttachments.map((attachment) => draftFromStoredAttachment(attachment)), ...imageDrafts];
    const activeFile = files.activeFile();
    const unavailable = !activeGeneration && residentMatches
        && (state.residentArtifact?.engineSupported === false
            || (state.residentArtifact?.canGenerate === false && state.residentArtifact?.canPredict !== true));
    const predictionMode = activeGeneration ? activeGeneration.canStop === false : prediction && !unavailable;
    const selectionLabel = activeGeneration
        ? (predictionMode ? 'Local next-token prediction' : activeGeneration.source.kind === 'artifact' ? 'Local generation' : 'API chat')
        : !selected ? 'No source selected'
        : selected.kind === 'artifact' && !residentMatches ? 'Selected model; not ready'
        : selected.kind === 'api' && !store.canRun() ? 'Invalid API endpoint'
        : unavailable ? 'Generation unavailable'
        : predictionMode ? 'Local next-token prediction'
        : selected.kind === 'artifact' ? 'Local generation' : 'API chat';

    imageDraftsRef.current = imageDrafts;

    function send(prompt: string): boolean {
        if (!generation.send(prompt)) return false;
        refresh();
        return true;
    }

    function budgetItems() {
        return [
            ...ui.getAttachments().map((attachment) => ({
                name: attachment.name,
                size: attachmentSize(attachment),
                kind: attachment.kind === 'image' ? 'image' as const : attachment.source === 'file' ? 'document' as const : 'text' as const,
            })),
            ...imageDraftsRef.current.map((attachment) => ({ name: attachment.name, size: attachment.bytes.byteLength, kind: 'image' as const })),
        ];
    }

    function removeStoredOrImage(id: string) {
        const image = imageDraftsRef.current.find((attachment) => attachment.id === id);
        if (image) {
            revokePreview(image);
            setImageDrafts((current) => current.filter((attachment) => attachment.id !== id));
            return;
        }
        if (id.startsWith('stored:')) {
            ui.removeAttachment(id.slice('stored:'.length));
            refresh();
        }
    }

    function clearAllAttachments() {
        for (const attachment of imageDraftsRef.current) revokePreview(attachment);
        setImageDrafts([]);
        ui.clearAttachments();
        refresh();
    }

    function addBudgetedAttachment(entry: Attachment): boolean {
        const plan = planAttachmentBatch([{ name: entry.name, size: attachmentSize(entry), type: entry.kind === 'image' ? 'image/png' : 'text/plain' }], budgetItems());
        for (const error of plan.errors) ui.toast(error, 'error');
        if (!plan.accepted.length) return false;
        const previous = imageDraftsRef.current.find((attachment) => attachment.name === entry.name);
        if (previous) {
            revokePreview(previous);
            setImageDrafts((current) => current.filter((attachment) => attachment.id !== previous.id));
        }
        ui.addAttachment(entry);
        refresh();
        return true;
    }

    async function handleAttachFiles(filesToAttach: File[]) {
        const plan = planAttachmentBatch(filesToAttach, budgetItems());
        for (const error of plan.errors) ui.toast(error, 'error');
        if (!plan.accepted.length) return;
        const planned = new Map<string, { file: File; kind: 'image' | 'document' }>();
        for (const entry of plan.accepted) planned.set(entry.file.name, entry);
        const images = new Map<string, File>();
        const documents = new Map<string, File>();
        for (const entry of planned.values()) {
            if (entry.kind === 'image') images.set(entry.file.name, entry.file);
            else documents.set(entry.file.name, entry.file);
        }
        if (images.size) {
            const nextDrafts: AttachmentDraft[] = [];
            for (const [name, file] of images) {
                try {
                    const bytes = new Uint8Array(await file.arrayBuffer());
                    const previewUrl = typeof URL.createObjectURL === 'function' ? URL.createObjectURL(file) : undefined;
                    nextDrafts.push({ id: `image:${name}:${++imageSequence}`, name, kind: 'image', bytes, mediaType: file.type || 'application/octet-stream', previewUrl });
                } catch (error) {
                    ui.toast(`${name} could not be read: ${String(error)}`, 'error');
                }
            }
            if (nextDrafts.length && !mountedRef.current) {
                for (const draft of nextDrafts) revokePreview(draft);
                return;
            }
            if (nextDrafts.length) {
                const names = new Set(nextDrafts.map((attachment) => attachment.name));
                for (const previous of imageDraftsRef.current.filter((attachment) => names.has(attachment.name))) revokePreview(previous);
                for (const name of names) if (ui.getAttachments().some((attachment) => attachment.name === name)) ui.removeAttachment(name);
                setImageDrafts((current) => [...current.filter((attachment) => !names.has(attachment.name)), ...nextDrafts]);
            }
        }
        if (documents.size) {
            const existing = ui.getAttachments().filter((attachment) => attachment.kind !== 'image');
            try {
                const result = await readTextFiles(Array.from(documents.values()), {
                    existingAttachments: existing,
                    maxFileBytes: MAX_DOCUMENT_BYTES,
                    maxTotalBytes: MAX_TOTAL_ATTACHMENT_BYTES,
                }) as { attachments: Attachment[]; errors: string[] };
                for (const attachment of result.attachments) ui.addAttachment(attachment);
                for (const error of result.errors) ui.toast(error, 'error');
                const storedNames = new Set(result.attachments.map((attachment) => attachment.name));
                const replacedImages = imageDraftsRef.current.filter((attachment) => storedNames.has(attachment.name));
                for (const previous of replacedImages) revokePreview(previous);
                if (replacedImages.length) setImageDrafts((current) => current.filter((attachment) => !storedNames.has(attachment.name)));
            } catch (error) {
                ui.toast(`Attachments could not be read: ${String(error)}`, 'error');
            }
        }
        refresh();
    }

    useEffect(() => {
        if (controlledDraft === undefined || controlledDraft === previousControlledDraft.current) return;
        previousControlledDraft.current = controlledDraft;
        if (ownControlledDraft.current === controlledDraft) {
            ownControlledDraft.current = null;
            return;
        }
        setVisibleDraft(controlledDraft);
        if (controllerRef.current) controllerRef.current.reset(controlledDraft);
    }, [controlledDraft]);

    const modelLabel = selected ? selected.id : 'Choose model';

    return (
        <section aria-label="Agent composer" class="agent-workspace">
            <header class="agent-heading"><div><h1>Agent</h1><p>{selectionLabel}</p></div><span class="session-phase">Session: {state.session.phase}</span></header>
            {state.session.error && <p role="alert" class="workbench-alert">{state.session.error}</p>}
            {activeGeneration && <p class="generation-banner">Generating with {activeGeneration.source.id} ({activeGeneration.privacy})</p>}
            <AgentTranscript messages={messages} canRun={store.canRun()} copy={copy} insert={(content) => files.insertText(content, nextInsertName())} download={(content) => download('studio-message.md', content)} retry={(content) => generation.send(content)} />
            <details class="context-drawer" aria-label="Context attachments"><summary><span><Icon name="attach" />Context</span><span class="attachment-count">{attachments.length}</span></summary><div class="context-content">
                <div class="context-actions"><button type="button" disabled={!activeFile} onClick={() => { const file = files.activeFile(); if (file) addBudgetedAttachment({ name: file.path, content: file.content, source: 'editor' }); }}>Attach current file</button><button type="button" disabled={!attachments.length} onClick={clearAllAttachments}>Clear attachments</button></div>
                <label>Pasted context<textarea aria-label="Pasted context" value={pasted} onInput={(event) => setPasted(event.currentTarget.value)} /></label>
                <button type="button" disabled={!pasted.trim()} onClick={() => { if (addBudgetedAttachment({ name: 'pasted-context.txt', content: pasted, source: 'paste' })) setPasted(''); }}>Attach pasted text</button>
                <p class="composer-attachment-note">Images stay in this browser tab and are not sent for visual understanding.</p>
            </div></details>
            <details class="build-tasks" aria-label="Build task templates"><summary><span><Icon name="command" />Build tasks</span><span class="attachment-count">{LOCAL_TEMPLATES.length}</span></summary><div class="build-task-grid">{LOCAL_TEMPLATES.map(([kind, label]) => <button type="button" data-template={kind} onClick={() => { setDraft(ui.templatePrompt(kind)); }}>{label}</button>)}</div></details>
            <ComposerSurface
                value={visibleDraft}
                onChange={(value) => setVisibleDraft(value)}
                onDraftChange={(value) => notifyDraftChange(value)}
                attachments={attachments}
                onAttach={handleAttachFiles}
                onRemove={removeStoredOrImage}
                onSend={send}
                onStop={() => { generation.stop(); refresh(); }}
                mode={permissionMode}
                onMode={setPermissionMode}
                model={modelLabel}
                onModel={() => ui.setView('model')}
                environment="Local browser"
                onEnvironment={() => {}}
                disabled={!store.canRun()}
                runtimeAvailable={false}
                initialDraft={initialDraftRef.current}
                sendLabel={predictionMode ? 'Predict next tokens' : 'Send'}
                generating={generating}
                canStop={activeGeneration?.canStop !== false}
                editorFile={activeFile}
                artifacts={[]}
                templatePrompt={(kind) => ui.templatePrompt(kind)}
                onMentionAdd={(entry) => { addBudgetedAttachment(entry); }}
                onController={(controller) => { controllerRef.current = controller; }}
            />
        </section>
    );
}
