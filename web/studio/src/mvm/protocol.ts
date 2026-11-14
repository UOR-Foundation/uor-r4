export const MVM_LIMITS = Object.freeze({
    maxFiles: 512,
    maxFileBytes: 1024 * 1024,
    maxWorkspaceBytes: 16 * 1024 * 1024,
    maxCommandBytes: 64 * 1024,
    maxProtocolBytes: 2 * 1024 * 1024,
    maxOutputBytes: 1024 * 1024,
    maxCowSectors: 256,
    maxCowSectorBytes: 65_536,
    maxGitCommits: 256,
    maxHistoryEntries: 200,
});

export const MVM_KINDS = ['init', 'status', 'exec', 'read', 'write', 'list', 'snapshot', 'restore', 'compile', 'vsock', 'reset', 'dispose'] as const;
export type MvmKind = (typeof MVM_KINDS)[number];

export interface MvmRequest {
    id: number;
    kind: MvmKind;
    payload: Record<string, unknown>;
}

export type MvmResponse =
    | { id: number; ok: true; result: unknown }
    | { id: number; ok: false; error: string };

export class MvmClientError extends Error {
    constructor(readonly code: string, message: string) {
        super(message);
        this.name = 'MvmClientError';
    }
}

function isObject(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function utf8Length(value: string): number {
    return new TextEncoder().encode(value).byteLength;
}

function requireBound(value: string, maximum: number, label: string): void {
    if (utf8Length(value) > maximum) {
        throw new MvmClientError('limit_exceeded', `${label} exceeds the ${maximum}-byte bound`);
    }
}

export function createMvmRequest(request: MvmRequest): string {
    if (!isObject(request) || !Number.isSafeInteger(request.id) || request.id < 1) {
        throw new MvmClientError('invalid_request', 'request id must be a positive safe integer');
    }
    if (!MVM_KINDS.includes(request.kind)) {
        throw new MvmClientError('invalid_request', 'request kind is not allowlisted');
    }
    if (!isObject(request.payload)) {
        throw new MvmClientError('invalid_request', 'request payload must be an object');
    }
    for (const key of Object.keys(request)) {
        if (!['id', 'kind', 'payload'].includes(key)) {
            throw new MvmClientError('invalid_request', `unexpected request key: ${key}`);
        }
    }
    if (typeof request.payload.cmd === 'string') requireBound(request.payload.cmd, MVM_LIMITS.maxCommandBytes, 'command');
    if (typeof request.payload.text === 'string') requireBound(request.payload.text, MVM_LIMITS.maxFileBytes, 'file');
    if (typeof request.payload.source === 'string') requireBound(request.payload.source, MVM_LIMITS.maxFileBytes, 'source');
    const encoded = JSON.stringify(request);
    requireBound(encoded, MVM_LIMITS.maxProtocolBytes, 'protocol message');
    return encoded;
}

export function parseMvmResponse(encoded: string): MvmResponse {
    requireBound(encoded, MVM_LIMITS.maxProtocolBytes, 'protocol response');
    let value: unknown;
    try {
        value = JSON.parse(encoded);
    } catch {
        throw new MvmClientError('invalid_response', 'response is not valid JSON');
    }
    if (!isObject(value) || !Number.isSafeInteger(value.id) || (value.id as number) < 1 || typeof value.ok !== 'boolean') {
        throw new MvmClientError('invalid_response', 'response shape is invalid');
    }
    for (const key of Object.keys(value)) {
        if (!['id', 'ok', 'result', 'error'].includes(key)) {
            throw new MvmClientError('invalid_response', `unexpected response key: ${key}`);
        }
    }
    if (value.ok === true) {
        if (!('result' in value) || 'error' in value) {
            throw new MvmClientError('invalid_response', 'successful response must contain only result');
        }
        return { id: value.id as number, ok: true, result: value.result };
    }
    if (typeof value.error !== 'string' || 'result' in value) {
        throw new MvmClientError('invalid_response', 'failed response must contain only an error string');
    }
    return { id: value.id as number, ok: false, error: value.error };
}
