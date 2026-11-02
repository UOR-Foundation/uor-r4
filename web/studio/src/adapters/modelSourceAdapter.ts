import { LOAD_PROGRESS_EVENT } from '../../assets/js/model_sources.js';
import type { LoadOperation, LoadResult, ModelSourcesApi } from '../../assets/js/model_sources.js';
import { LifecycleStore } from '../state/lifecycle.js';

export { LOAD_PROGRESS_EVENT };

export interface WorkerLike {
    postMessage(message: unknown): void;
}

export interface WorkerMessage {
    action: string;
    capabilities?: { schema?: string };
    error?: string;
}

export interface ModelSourceAdapter {
    loadPublished(idOrEntry: unknown): Promise<LoadResult>;
    loadFromUrl(rawUrl: string): Promise<LoadResult>;
    loadLocalFile(file: File): Promise<LoadResult>;
    loadBundled(): Promise<LoadResult>;
    selectApi(sourceId: string, endpoint: string): void;
    testApi(id?: string): Promise<unknown>;
    handleWorkerMessage(message: WorkerMessage): void;
}

function sourceKind(result: LoadResult): string {
    const kind = result.meta.kind;
    if (kind === 'bundled') return 'bundled';
    if (kind === 'url' || kind === 'published') return 'url';
    return 'file';
}

export function createModelSourceAdapter(
    store: LifecycleStore,
    sources: ModelSourcesApi,
    worker: WorkerLike,
): ModelSourceAdapter {
    function start(sourceId: string): LoadOperation {
        store.dispatch({ type: 'artifact/selected', sourceId });
        const operation = store.getState().operation;
        if (!operation) throw new Error('artifact selection did not create an operation');
        return operation;
    }

    async function finish(result: LoadResult, operation: LoadOperation): Promise<LoadResult> {
        if (result.stale) return result;
        worker.postMessage({
            action: 'load_model',
            payload: {
                bytes: result.bytes,
                fileName: result.fileName,
                source: sourceKind(result),
                sourceLabel: result.meta.label || result.meta.fileName || '',
                expectedSha256: result.expectedSha256,
                tokenizerArtifact: result.tokenizerArtifact,
                operationId: operation.operationId,
            },
        });
        store.dispatch({ type: 'artifact/finalizing', operation });
        return result;
    }

    document.addEventListener(LOAD_PROGRESS_EVENT, (event) => {
        const detail = (event as CustomEvent<{
            operationId?: number;
            loaded: number;
            total: number | null;
            phase: 'downloading' | 'finalizing';
        }>).detail;
        const operation = store.getState().operation;
        if (!operation || detail.operationId !== operation.operationId) return;
        if (detail.phase === 'downloading') {
            store.dispatch({ type: 'artifact/downloadProgress', operation, loaded: detail.loaded, total: detail.total });
        } else {
            store.dispatch({ type: 'artifact/finalizing', operation });
        }
    });

    return {
        async loadPublished(idOrEntry) {
            const operation = start(typeof idOrEntry === 'string' ? idOrEntry : 'published');
            return finish(await sources.loadPublished(idOrEntry, operation), operation);
        },
        async loadFromUrl(rawUrl) {
            const operation = start('url');
            return finish(await sources.loadFromUrl(rawUrl, operation), operation);
        },
        async loadLocalFile(file) {
            const operation = start('local-file');
            return finish(await sources.loadLocalFile(file, operation), operation);
        },
        async loadBundled() {
            const operation = start('bundled-demo');
            return finish(await sources.loadBundled(operation), operation);
        },
        selectApi(sourceId, endpoint) {
            store.dispatch({ type: 'api/selected', sourceId, endpoint });
        },
        async testApi(id) {
            store.dispatch({ type: 'api/testStarted' });
            try {
                const result = await sources.testApi(id);
                store.dispatch({ type: 'api/testFinished', ok: result.ok });
                return result;
            } catch (error) {
                store.dispatch({ type: 'api/testFinished', ok: false });
                throw error;
            }
        },
        handleWorkerMessage(message) {
            const operation = store.getState().operation;
            if (!operation) return;
            if (message.action === 'model_loaded') {
                store.dispatch({
                    type: 'artifact/ready',
                    operation,
                    artifact: { schema: message.capabilities?.schema || '' },
                });
            } else if (message.action === 'operation_error') {
                store.dispatch({ type: 'artifact/failed', operation, error: message.error || 'Operation failed.' });
            }
        },
    };
}
