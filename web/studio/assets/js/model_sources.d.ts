export const LOAD_PROGRESS_EVENT: 'uor-model-load-progress';

export interface LoadOperation {
    operationId: number;
    sourceId: string;
}

export interface LoadResult {
    stale?: boolean;
    bytes: Uint8Array;
    fileName: string;
    expectedSha256?: string;
    tokenizerArtifact?: unknown;
    meta: {
        kind?: string;
        label?: string;
        fileName?: string;
        schema: string;
    };
}

export interface ModelSourcesApi {
    setActive(next: { kind: string; id: string }): unknown;
    loadPublished(idOrEntry: unknown, operation?: LoadOperation): Promise<LoadResult>;
    loadFromUrl(rawUrl: string, operation?: LoadOperation): Promise<LoadResult>;
    loadLocalFile(file: File, operation?: LoadOperation): Promise<LoadResult>;
    loadBundled(operation?: LoadOperation): Promise<LoadResult>;
    testApi(id?: string): Promise<{ ok: boolean; detail?: string; models?: unknown[] }>;
}
