export type SourceKind = 'artifact' | 'api';
export type ArtifactPhase = 'unselected' | 'selected' | 'downloading' | 'finalizing' | 'ready' | 'error';
export type ApiTestStatus = 'untested' | 'testing' | 'passed' | 'failed';
export type FilePhase = 'no-folder' | 'no-open-file' | 'clean' | 'dirty' | 'saving' | 'saved' | 'error';
export type SessionPhase = 'new' | 'active' | 'generating' | 'complete' | 'stopped' | 'error';
export type Privacy = 'localWASM' | 'loopbackAPI' | 'privateNetworkAPI' | 'remoteAPI';

export interface SourceRef {
    kind: SourceKind;
    id: string;
    endpoint?: string;
}

export interface LoadOperation {
    operationId: number;
    sourceId: string;
}

export interface DownloadProgress {
    loaded: number;
    total: number | null;
    percent: number | null;
}

export interface ResidentArtifact {
    sourceId: string;
    schema: string;
}

export interface ArtifactState {
    phase: ArtifactPhase;
    error: string | null;
}

export interface ApiState {
    testStatus: ApiTestStatus;
}

export interface FileState {
    phase: FilePhase;
    path: string | null;
    error: string | null;
}

export interface GenerationSnapshot {
    source: SourceRef;
    privacy: Privacy;
}

export interface SessionState {
    phase: SessionPhase;
    activeGeneration: GenerationSnapshot | null;
    lastGeneration: GenerationSnapshot | null;
}

export interface LifecycleState {
    selectedSource: SourceRef | null;
    operation: LoadOperation | null;
    artifact: ArtifactState;
    residentArtifact: ResidentArtifact | null;
    api: ApiState;
    progress: DownloadProgress | null;
    file: FileState;
    session: SessionState;
}

interface OperationAction {
    operation: LoadOperation;
}

export type LifecycleAction =
    | { type: 'artifact/selected'; sourceId: string }
    | ({ type: 'artifact/downloadStarted' } & OperationAction)
    | ({ type: 'artifact/downloadProgress'; loaded: number; total: number | null } & OperationAction)
    | ({ type: 'artifact/finalizing' } & OperationAction)
    | ({ type: 'artifact/ready'; artifact: Pick<ResidentArtifact, 'schema'> } & OperationAction)
    | ({ type: 'artifact/failed'; error: string } & OperationAction)
    | { type: 'api/selected'; sourceId: string; endpoint: string }
    | { type: 'api/testStarted' }
    | { type: 'api/testFinished'; ok: boolean }
    | { type: 'file/opened'; path: string }
    | { type: 'file/dirty' }
    | { type: 'file/saveStarted' }
    | { type: 'file/saveFinished' }
    | { type: 'file/saveFailed'; error: string }
    | { type: 'generation/started'; source: SourceRef }
    | { type: 'generation/finished'; outcome: 'complete' | 'stopped' | 'error' };

function initialState(): LifecycleState {
    return {
        selectedSource: null,
        operation: null,
        artifact: { phase: 'unselected', error: null },
        residentArtifact: null,
        api: { testStatus: 'untested' },
        progress: null,
        file: { phase: 'no-folder', path: null, error: null },
        session: { phase: 'new', activeGeneration: null, lastGeneration: null },
    };
}

export class LifecycleStore {
    private state: LifecycleState = initialState();
    private nextOperationId = 0;

    getState(): LifecycleState {
        return this.state;
    }

    canRun(): boolean {
        if (this.state.session.phase === 'generating' || !this.state.selectedSource) return false;
        if (this.state.selectedSource.kind === 'api') return true;
        return this.state.artifact.phase === 'ready';
    }

    canSave(): boolean {
        return this.state.file.path !== null && this.state.file.phase !== 'saving';
    }

    dispatch(action: LifecycleAction): void {
        switch (action.type) {
            case 'artifact/selected': {
                this.nextOperationId += 1;
                this.state = {
                    ...this.state,
                    selectedSource: { kind: 'artifact', id: action.sourceId },
                    operation: { operationId: this.nextOperationId, sourceId: action.sourceId },
                    artifact: { ...this.state.artifact, phase: 'selected', error: null },
                    progress: null,
                };
                return;
            }
            case 'artifact/downloadStarted':
                if (this.isCurrent(action.operation)) {
                    this.state = {
                        ...this.state,
                        artifact: { ...this.state.artifact, phase: 'downloading', error: null },
                        progress: null,
                    };
                }
                return;
            case 'artifact/downloadProgress':
                if (this.isCurrent(action.operation)) {
                    const total = action.total !== null && action.total > 0 ? action.total : null;
                    this.state = {
                        ...this.state,
                        artifact: { ...this.state.artifact, phase: 'downloading', error: null },
                        progress: {
                            loaded: action.loaded,
                            total,
                            percent: total === null ? null : Math.min(100, Math.floor((action.loaded * 100) / total)),
                        },
                    };
                }
                return;
            case 'artifact/finalizing':
                if (this.isCurrent(action.operation)) {
                    this.state = {
                        ...this.state,
                        artifact: { ...this.state.artifact, phase: 'finalizing', error: null },
                        progress: this.state.progress
                            ? { ...this.state.progress, percent: null }
                            : { loaded: 0, total: null, percent: null },
                    };
                }
                return;
            case 'artifact/ready':
                if (this.isCurrent(action.operation)) {
                    this.state = {
                        ...this.state,
                        artifact: { phase: 'ready', error: null },
                        residentArtifact: { sourceId: action.operation.sourceId, schema: action.artifact.schema },
                        progress: null,
                    };
                }
                return;
            case 'artifact/failed':
                if (this.isCurrent(action.operation)) {
                    this.state = {
                        ...this.state,
                        artifact: { ...this.state.artifact, phase: 'error', error: action.error },
                        progress: null,
                    };
                }
                return;
            case 'api/selected':
                this.state = {
                    ...this.state,
                    selectedSource: { kind: 'api', id: action.sourceId, endpoint: action.endpoint },
                    operation: null,
                    artifact: { ...this.state.artifact, phase: 'unselected', error: null },
                    api: { testStatus: 'untested' },
                    progress: null,
                };
                return;
            case 'api/testStarted':
                if (this.state.selectedSource?.kind === 'api') {
                    this.state = { ...this.state, api: { testStatus: 'testing' } };
                }
                return;
            case 'api/testFinished':
                if (this.state.selectedSource?.kind === 'api') {
                    this.state = { ...this.state, api: { testStatus: action.ok ? 'passed' : 'failed' } };
                }
                return;
            case 'file/opened':
                this.state = {
                    ...this.state,
                    file: { phase: 'clean', path: action.path, error: null },
                };
                return;
            case 'file/dirty':
                if (this.state.file.path !== null) {
                    this.state = { ...this.state, file: { ...this.state.file, phase: 'dirty', error: null } };
                }
                return;
            case 'file/saveStarted':
                if (this.state.file.path !== null && this.state.file.phase !== 'saving') {
                    this.state = { ...this.state, file: { ...this.state.file, phase: 'saving', error: null } };
                }
                return;
            case 'file/saveFinished':
                if (this.state.file.path !== null && this.state.file.phase === 'saving') {
                    this.state = { ...this.state, file: { ...this.state.file, phase: 'saved', error: null } };
                }
                return;
            case 'file/saveFailed':
                if (this.state.file.path !== null) {
                    this.state = { ...this.state, file: { ...this.state.file, phase: 'error', error: action.error } };
                }
                return;
            case 'generation/started': {
                const selected = this.state.selectedSource;
                if (!this.canRun() || !selected || selected.kind !== action.source.kind || selected.id !== action.source.id) return;
                const generation = {
                    source: { ...action.source },
                    privacy: action.source.kind === 'api' ? classifyEndpoint(action.source.endpoint) : 'localWASM' as const,
                };
                this.state = {
                    ...this.state,
                    session: { phase: 'generating', activeGeneration: generation, lastGeneration: this.state.session.lastGeneration },
                };
                return;
            }
            case 'generation/finished': {
                const active = this.state.session.activeGeneration;
                if (!active) return;
                this.state = {
                    ...this.state,
                    session: { phase: action.outcome, activeGeneration: null, lastGeneration: active },
                };
                return;
            }
        }
    }

    private isCurrent(operation: LoadOperation): boolean {
        const current = this.state.operation;
        const selected = this.state.selectedSource;
        return Boolean(
            current
            && selected?.kind === 'artifact'
            && current.operationId === operation.operationId
            && current.sourceId === operation.sourceId
            && selected.id === operation.sourceId,
        );
    }
}

export function classifyEndpoint(input?: string): Privacy {
    if (input === undefined) return 'localWASM';
    const hostname = new URL(input).hostname.toLowerCase().replace(/^\[|\]$/g, '');
    if (hostname === 'localhost' || hostname === '::1' || hostname.startsWith('127.')) return 'loopbackAPI';
    if (
        hostname === '::'
        || hostname.startsWith('fc')
        || hostname.startsWith('fd')
        || hostname.startsWith('fe80')
        || hostname.startsWith('10.')
        || hostname.startsWith('192.168.')
        || hostname.startsWith('169.254.')
        || /^172\.(1[6-9]|2[0-9]|3[0-1])\./.test(hostname)
    ) {
        return 'privateNetworkAPI';
    }
    return 'remoteAPI';
}
