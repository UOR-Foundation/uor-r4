import { MVM_LIMITS } from './protocol.js';
import {
    assertVirtualGitHashes,
    validateVirtualGitSnapshot,
    type VirtualGitSnapshot,
} from './virtualGit.js';

export interface StoredCowSector {
    id: number;
    bytes: Uint8Array;
}

export interface MvmPersistedCowView {
    inodes: Record<string, unknown>;
    dirs: string[];
    cow_sectors: StoredCowSector[];
    dirty_sectors: number[];
    next_sector: number;
    next_pid: number;
}

export interface MvmSnapshotInput {
    version: 1;
    snapshotId: string;
    cwd: string;
    cowDigest: string;
    files: Record<string, { text: string; revision: number }>;
    sectors: StoredCowSector[];
    cow?: MvmPersistedCowView;
    git?: VirtualGitSnapshot;
}

export interface MvmSnapshot extends MvmSnapshotInput {
    digest: string;
}

export interface SnapshotStorage {
    get(key: string): Promise<unknown>;
    put(key: string, value: unknown): Promise<void>;
    delete(key: string): Promise<void>;
}

export class MemorySnapshotStorage implements SnapshotStorage {
    private readonly values = new Map<string, unknown>();
    private failure: Error | DOMException | null = null;
    writeCount = 0;

    async get(key: string): Promise<unknown> {
        const value = this.values.get(key);
        return value === undefined ? undefined : clone(value);
    }

    async put(key: string, value: unknown): Promise<void> {
        if (this.failure !== null) {
            const failure = this.failure;
            this.failure = null;
            throw failure;
        }
        this.values.set(key, clone(value));
        this.writeCount += 1;
    }

    async delete(key: string): Promise<void> {
        this.values.delete(key);
    }

    seed(key: string, value: unknown): void {
        this.values.set(key, clone(value));
    }

    readRaw(key: string): unknown {
        return this.values.get(key);
    }

    failNextPut(error: Error | DOMException): void {
        this.failure = error;
    }
}

function isObject(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function clone<T>(value: T): T {
    if (value instanceof Map || value instanceof Set) {
        throw new Error('MVM persistence requires plain objects, not Map or Set');
    }
    if (value instanceof Uint8Array) return value.slice() as T;
    if (Array.isArray(value)) return value.map((entry) => clone(entry)) as T;
    if (isObject(value)) {
        return Object.fromEntries(Object.entries(value).map(([key, entry]) => [key, clone(entry)])) as T;
    }
    return value;
}

function textBytes(text: string): number {
    return new TextEncoder().encode(text).byteLength;
}

function hex(bytes: Uint8Array): string {
    return [...bytes].map((value) => value.toString(16).padStart(2, '0')).join('');
}

async function sha256(bytes: Uint8Array): Promise<string> {
    const buffer = new Uint8Array(bytes).buffer as ArrayBuffer;
    return `sha256:${hex(new Uint8Array(await crypto.subtle.digest('SHA-256', buffer)))}`;
}

async function digestInput(input: MvmSnapshotInput): Promise<string> {
    const files = Object.entries(input.files).sort(([a], [b]) => a.localeCompare(b));
    const sectors = [...input.sectors].sort((a, b) => a.id - b.id);
    const git = input.git === undefined ? null : validateVirtualGitSnapshot(input.git);
    if (git !== null) await assertVirtualGitHashes(git);
    const header = JSON.stringify({
        version: input.version,
        snapshotId: input.snapshotId,
        cwd: input.cwd,
        cowDigest: input.cowDigest,
        files,
        sectors: await Promise.all(sectors.map(async (sector) => ({ id: sector.id, digest: await sha256(sector.bytes) }))),
        cow: input.cow,
        ...(git === null ? {} : { git }),
    });
    return sha256(new TextEncoder().encode(header));
}

function assertDigest(value: string, label: string): void {
    if (!/^sha256:[0-9a-f]{64}$/.test(value)) throw new Error(`${label} digest is invalid`);
}

function validateInput(input: MvmSnapshotInput): void {
    if (!isObject(input) || input.version !== 1) throw new Error('unsupported MVM persistence version');
    assertDigest(input.snapshotId, 'snapshot');
    assertDigest(input.cowDigest, 'CoW');
    if (typeof input.cwd !== 'string' || input.cwd.length === 0 || input.cwd.length > 4096) throw new Error('cwd is invalid');
    if (!isObject(input.files) || Object.keys(input.files).length > MVM_LIMITS.maxFiles) throw new Error('virtual file count is invalid or exceeds its bound');
    let total = 0;
    for (const [path, file] of Object.entries(input.files)) {
        if (path.length === 0 || !isObject(file) || typeof file.text !== 'string' || !Number.isSafeInteger(file.revision) || file.revision < 0) {
            throw new Error(`virtual file is invalid: ${path}`);
        }
        const bytes = textBytes(file.text);
        if (bytes > MVM_LIMITS.maxFileBytes) throw new Error(`virtual file exceeds its byte bound: ${path}`);
        total += bytes;
    }
    if (total > MVM_LIMITS.maxWorkspaceBytes) throw new Error('virtual workspace exceeds its byte bound');
    if (!Array.isArray(input.sectors) || input.sectors.length > MVM_LIMITS.maxCowSectors) throw new Error('CoW sector count exceeds its bound');
    let previous = -1;
    for (const sector of input.sectors) {
        if (!Number.isSafeInteger(sector.id) || sector.id < 0 || sector.id <= previous || !(sector.bytes instanceof Uint8Array) || sector.bytes.byteLength !== MVM_LIMITS.maxCowSectorBytes) {
            throw new Error('CoW sector is invalid');
        }
        previous = sector.id;
    }
    if (input.cow !== undefined) {
        if (!isObject(input.cow) || !Array.isArray(input.cow.cow_sectors)) throw new Error('complete CoW state is invalid');
        if (input.cow.inodes instanceof Map || input.cow.inodes instanceof Set) throw new Error('MVM persistence requires plain objects, not Map or Set');
        if (!isObject(input.cow.inodes) || Object.keys(input.cow.inodes).length === 0) throw new Error('complete CoW inode map is empty or invalid');
        if (!Array.isArray(input.cow.dirs) || !Array.isArray(input.cow.dirty_sectors)) throw new Error('complete CoW directory metadata is invalid');
        if (!Number.isSafeInteger(input.cow.next_sector) || !Number.isSafeInteger(input.cow.next_pid)) throw new Error('complete CoW counters are invalid');
        if (input.cow.cow_sectors.length > MVM_LIMITS.maxCowSectors) throw new Error('complete CoW sector count exceeds its bound');
        let previousCow = -1;
        for (const sector of input.cow.cow_sectors) {
            if (!Number.isSafeInteger(sector.id) || sector.id < 0 || sector.id <= previousCow || !(sector.bytes instanceof Uint8Array) || sector.bytes.byteLength !== MVM_LIMITS.maxCowSectorBytes) {
                throw new Error('complete CoW sector is invalid');
            }
            previousCow = sector.id;
        }
    }
    if (input.git !== undefined) validateVirtualGitSnapshot(input.git);
    for (const forbidden of ['history', 'commands', 'token', 'githubToken', 'authorization']) {
        if (forbidden in input) throw new Error(`forbidden persistence field: ${forbidden}`);
    }
}

export class MvmPersistence<S extends SnapshotStorage = SnapshotStorage> {
    constructor(readonly storage: S, readonly key: string) {}

    async save(input: MvmSnapshotInput): Promise<MvmSnapshot> {
        await this.loadOrNull();
        validateInput(input);
        const digest = await digestInput(input);
        const record: MvmSnapshot = { ...clone(input), digest };
        try {
            await this.storage.put(this.key, record);
        } catch (error) {
            const name = error instanceof DOMException ? error.name : 'write_failed';
            throw new Error(`MVM persistence write failed (${name})`);
        }
        return this.load({ snapshotId: record.snapshotId, digest });
    }

    async load(expected?: { snapshotId: string; digest: string }): Promise<MvmSnapshot> {
        const raw = await this.storage.get(this.key);
        if (raw === undefined) throw new Error('MVM persistence record was not found');
        if (!isObject(raw) || typeof raw.digest !== 'string') throw new Error('MVM persistence record is corrupt');
        const input = { ...raw } as unknown as MvmSnapshotInput;
        validateInput(input);
        const computed = await digestInput(input);
        if (computed !== raw.digest) throw new Error('MVM persistence digest mismatch');
        if (expected !== undefined && (expected.snapshotId !== raw.snapshotId || expected.digest !== raw.digest)) {
            throw new Error('MVM snapshot identity mismatch');
        }
        return clone(raw) as unknown as MvmSnapshot;
    }

    async loadOrNull(): Promise<MvmSnapshot | null> {
        const raw = await this.storage.get(this.key);
        if (raw === undefined) return null;
        return this.load();
    }

    async clear(): Promise<void> {
        await this.storage.delete(this.key);
    }
}

export function snapshotMetadata(snapshot: MvmSnapshot): { snapshotId: string; digest: string } {
    return { snapshotId: snapshot.snapshotId, digest: snapshot.digest };
}

interface SessionStorageLike {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
}

interface SandboxKeyOptions {
    sessionStorage: SessionStorageLike;
    locks?: LockManager;
    freshOnConflict?: boolean;
    randomUUID?: () => string;
}

export interface SandboxLease {
    key: string;
    persistent: boolean;
    release(): Promise<void>;
}

const SESSION_KEY = 'uor-r4.studio.mvm.tab';

export async function acquireSandboxKey(options: SandboxKeyOptions): Promise<SandboxLease> {
    const random = options.randomUUID ?? (() => crypto.randomUUID());
    const hinted = options.sessionStorage.getItem(SESSION_KEY);
    const key = hinted ?? random();
    if (options.locks === undefined) {
        return { key: `${key}.ephemeral`, persistent: false, release: async () => undefined };
    }
    const locks = options.locks;
    const acquired = new Promise<{ key: string; releaseHold: () => void } | null>((resolve, reject) => {
        let settled = false;
        const succeed = (value: { key: string; releaseHold: () => void } | null) => {
            if (!settled) {
                settled = true;
                resolve(value);
            }
        };
        const fail = (error: unknown) => {
            if (!settled) {
                settled = true;
                reject(error);
            }
        };
        void locks.request(`uor-r4.studio.mvm:${key}`, { mode: 'exclusive', ifAvailable: true }, (lock) => {
            if (lock === null) {
                succeed(null);
                return undefined;
            }
            let releaseHold: () => void = () => {};
            const held = new Promise<void>((resolve) => { releaseHold = resolve; });
            succeed({ key, releaseHold: () => releaseHold() });
            return held;
        }).catch(fail);
    });
    const lease = await acquired;
    if (lease === null) {
        if (options.freshOnConflict === true) {
            return { key: random(), persistent: false, release: async () => undefined };
        }
        throw new Error('Another tab owns this MVM sandbox; ownership was not duplicated');
    }
    options.sessionStorage.setItem(SESSION_KEY, lease.key);
    return {
        key: lease.key,
        persistent: true,
        release: async () => { lease.releaseHold(); },
    };
}

function requestResult<T>(request: IDBRequest<T>): Promise<T> {
    return new Promise((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error ?? new Error('IndexedDB request failed'));
    });
}

function transactionDone(transaction: IDBTransaction): Promise<void> {
    return new Promise((resolve, reject) => {
        transaction.oncomplete = () => resolve();
        transaction.onabort = () => reject(transaction.error ?? new Error('IndexedDB transaction aborted'));
        transaction.onerror = () => reject(transaction.error ?? new Error('IndexedDB transaction failed'));
    });
}

export async function createIdbSnapshotStorage(factory: IDBFactory): Promise<SnapshotStorage> {
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
        const request = factory.open('uor-r4.studio.mvm', 1);
        request.onupgradeneeded = (event) => {
            if (event.oldVersion !== 0) {
                reject(new Error('Unsupported MVM persistence version'));
                return;
            }
            request.result.createObjectStore('sandboxes');
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error ?? new Error('Unable to open MVM persistence'));
        request.onblocked = () => reject(new Error('MVM persistence open is blocked by another version'));
    });
    return {
        async get(key: string): Promise<unknown> {
            const transaction = db.transaction('sandboxes', 'readonly');
            const result = await requestResult(transaction.objectStore('sandboxes').get(key));
            await transactionDone(transaction);
            return result;
        },
        async put(key: string, value: unknown): Promise<void> {
            const transaction = db.transaction('sandboxes', 'readwrite');
            await requestResult(transaction.objectStore('sandboxes').put(value, key));
            await transactionDone(transaction);
        },
        async delete(key: string): Promise<void> {
            const transaction = db.transaction('sandboxes', 'readwrite');
            await requestResult(transaction.objectStore('sandboxes').delete(key));
            await transactionDone(transaction);
        },
    };
}
