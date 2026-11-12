import { MvmClientError, MVM_LIMITS, createMvmRequest, parseMvmResponse, type MvmKind } from './protocol.js';
import { acquireSandboxKey, type SandboxLease } from './persistence.js';

export { MvmClientError } from './protocol.js';

export interface MvmWorkerLike {
    postMessage(message: unknown): void;
    addEventListener(type: 'message', listener: (event: { data: string }) => void): void;
    removeEventListener(type: 'message', listener: (event: { data: string }) => void): void;
    terminate(): void;
}

interface SessionStorageLike {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
}

export interface MvmClientOptions {
    workerFactory?: () => MvmWorkerLike;
    sessionStorage?: SessionStorageLike;
    locks?: LockManager;
}

export interface MvmExecResult {
    exitCode: number;
    stdout: string;
    stderr: string;
    instructionsRetired?: number;
    compiledElfMachine?: number;
    cowStateSha256?: string;
}

export interface MvmHistoryEntry {
    cmd: string;
    cwd: string;
    persisted: false;
}

interface PendingRequest {
    resolve(result: unknown): void;
    reject(error: Error): void;
}

function defaultWorkerFactory(): MvmWorkerLike {
    return new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
}

function secretLike(command: string): boolean {
    return /(?:GITHUB_TOKEN|GH_TOKEN|Authorization\s*:|github_pat_|ghp_|\b[A-Z][A-Z0-9_]*_TOKEN\s*=)/i.test(command);
}

export class MvmClient {
    private readonly workerFactory: () => MvmWorkerLike;
    private readonly sessionStorage?: SessionStorageLike;
    private readonly locks?: LockManager;
    private worker: MvmWorkerLike | null = null;
    private listener: ((event: { data: string }) => void) | null = null;
    private leasePromise: Promise<SandboxLease> | null = null;
    private queue: Promise<void> = Promise.resolve();
    private nextId = 1;
    private disposed = false;
    private readonly pending = new Map<number, PendingRequest>();
    private readonly commandHistory: MvmHistoryEntry[] = [];
    private readonly fileListeners = new Set<() => void>();

    constructor(options: MvmClientOptions = {}) {
        this.workerFactory = options.workerFactory ?? defaultWorkerFactory;
        this.sessionStorage = options.sessionStorage;
        this.locks = options.locks;
    }

    history(): MvmHistoryEntry[] {
        return this.commandHistory.map((entry) => ({ ...entry }));
    }

    onFilesChanged(listener: () => void): () => void {
        this.fileListeners.add(listener);
        return () => { this.fileListeners.delete(listener); };
    }

    private notifyFilesChanged(): void {
        for (const listener of this.fileListeners) listener();
    }

    async status(): Promise<Record<string, unknown>> {
        return this.send('status', {}) as Promise<Record<string, unknown>>;
    }

    async exec(cmd: string, cwd = '/workspace'): Promise<MvmExecResult> {
        if (secretLike(cmd)) {
            throw new MvmClientError('secret_input_refused', 'Secret-like terminal input is refused; use the approved GitHub credential path.');
        }
        const result = await this.send('exec', { cmd, cwd }) as MvmExecResult;
        this.commandHistory.push({ cmd, cwd, persisted: false });
        if (this.commandHistory.length > MVM_LIMITS.maxHistoryEntries) this.commandHistory.shift();
        this.notifyFilesChanged();
        return result;
    }

    async read(path: string): Promise<{ text: string; revision: number }> {
        return this.send('read', { path }) as Promise<{ text: string; revision: number }>;
    }

    async write(path: string, text: string, baseRevision?: number): Promise<unknown> {
        const result = await this.send('write', { path, text, ...(baseRevision === undefined ? {} : { baseRevision }) });
        this.notifyFilesChanged();
        return result;
    }

    async list(prefix = '/workspace'): Promise<Array<{ path: string; kind: 'file'; sizeBytes: number }>> {
        return this.send('list', { prefix }) as Promise<Array<{ path: string; kind: 'file'; sizeBytes: number }>>;
    }

    async reset(): Promise<void> {
        await this.send('reset', {});
        this.commandHistory.length = 0;
        this.nextId = 1;
        this.notifyFilesChanged();
    }

    async snapshot(): Promise<{ snapshotId: string; digest: string }> {
        return this.send('snapshot', {}) as Promise<{ snapshotId: string; digest: string }>;
    }

    async restore(snapshot: { snapshotId: string; digest: string }): Promise<unknown> {
        const result = await this.send('restore', snapshot);
        this.notifyFilesChanged();
        return result;
    }

    async compile(source: string): Promise<unknown> {
        return this.send('compile', { source });
    }

    async vsock(payload: Record<string, unknown>): Promise<unknown> {
        return this.send('vsock', payload);
    }

    async dispose(): Promise<void> {
        if (this.disposed) return;
        try {
            if (this.worker !== null) await this.send('dispose', {});
        } finally {
            this.disposed = true;
            if (this.worker !== null && this.listener !== null) {
                this.worker.removeEventListener('message', this.listener);
                this.worker.terminate();
            }
            for (const request of this.pending.values()) {
                request.reject(new MvmClientError('disposed', 'MVM client is disposed'));
            }
            this.pending.clear();
            this.worker = null;
            this.listener = null;
            if (this.leasePromise !== null) {
                const lease = await this.leasePromise;
                await lease.release();
            }
        }
    }

    private ensureWorker(): MvmWorkerLike {
        if (this.disposed) throw new MvmClientError('disposed', 'MVM client is disposed');
        if (this.worker !== null) return this.worker;
        const worker = this.workerFactory();
        const listener = (event: { data: string }) => this.receive(event.data);
        worker.addEventListener('message', listener);
        this.worker = worker;
        this.listener = listener;
        return worker;
    }

    private ensureReady(): Promise<SandboxLease> {
        this.ensureWorker();
        if (this.leasePromise === null) {
            const sessionStorage = this.sessionStorage ?? globalThis.sessionStorage;
            const locks = this.locks ?? globalThis.navigator?.locks;
            this.leasePromise = acquireSandboxKey({ sessionStorage, locks });
        }
        return this.leasePromise;
    }

    private receive(encoded: string): void {
        let response: ReturnType<typeof parseMvmResponse>;
        try {
            response = parseMvmResponse(encoded);
        } catch (error) {
            for (const request of this.pending.values()) request.reject(error as Error);
            this.pending.clear();
            return;
        }
        const pending = this.pending.get(response.id);
        if (pending === undefined) return;
        this.pending.delete(response.id);
        if (response.ok) pending.resolve(response.result);
        else pending.reject(new MvmClientError('worker_error', response.error));
    }

    private send(kind: MvmKind, payload: Record<string, unknown>): Promise<unknown> {
        const operation = async (): Promise<unknown> => {
            if (this.disposed) throw new MvmClientError('disposed', 'MVM client is disposed');
            const lease = await this.ensureReady();
            if (this.disposed) throw new MvmClientError('disposed', 'MVM client is disposed');
            const worker = this.ensureWorker();
            const id = this.nextId;
            this.nextId += 1;
            const encoded = createMvmRequest({
                id,
                kind,
                payload: { ...payload, sandboxKey: lease.key, persistent: lease.persistent },
            });
            const message = JSON.parse(encoded) as unknown;
            return new Promise((resolve, reject) => {
                this.pending.set(id, { resolve, reject });
                try {
                    worker.postMessage(message);
                } catch (error) {
                    this.pending.delete(id);
                    reject(error);
                }
            });
        };
        const result = this.queue.then(operation, operation);
        this.queue = result.then(() => undefined, () => undefined);
        return result;
    }
}
