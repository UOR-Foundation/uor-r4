import initWasm, { mvm_dispatch } from '../../node_modules/.cache/uor-r4-mvm/uor_r4_mvm.js';
import { createMvmRequest, type MvmKind } from './protocol.js';
import {
    createIdbSnapshotStorage,
    MemorySnapshotStorage,
    MvmPersistence,
    snapshotMetadata,
    type MvmSnapshotInput,
} from './persistence.js';
import { VirtualGit, type VirtualGitSnapshot } from './virtualGit.js';
import { executeVirtualCommand } from './commands.js';

interface HostSnapshot {
    snapshotId: string;
    digest: string;
}

declare global {
    var __uorMvmHost: {
        saveSnapshot(key: string, state: MvmSnapshotInput): Promise<HostSnapshot>;
        loadSnapshot(key: string, snapshotId: string, digest: string): Promise<unknown>;
        loadCurrentSnapshot(key: string): Promise<unknown>;
        clearSnapshot(key: string): Promise<void>;
    };
}

const persistenceByKey = new Map<string, Promise<MvmPersistence>>();
const persistentByKey = new Map<string, boolean>();
let wasmReady: Promise<void> | null = null;
let initializedKey: string | null = null;
let currentKey = '';
let restoredGit: VirtualGitSnapshot | undefined;
let git = new VirtualGit(workerFiles());

function persistence(key: string): Promise<MvmPersistence> {
    let existing = persistenceByKey.get(key);
    if (existing === undefined) {
        const persistent = persistentByKey.get(key) ?? true;
        existing = (async () => {
            const storage = persistent
                ? await createIdbSnapshotStorage(indexedDB)
                : new MemorySnapshotStorage();
            return new MvmPersistence(storage, key);
        })();
        persistenceByKey.set(key, existing);
    }
    return existing;
}

globalThis.__uorMvmHost = {
    async saveSnapshot(key, state) {
        const normalized = {
            ...state,
            sectors: state.sectors.map((sector) => ({
                ...sector,
                bytes: sector.bytes instanceof Uint8Array ? sector.bytes : new Uint8Array(sector.bytes),
            })),
            cow: state.cow === undefined ? undefined : {
                ...state.cow,
                cow_sectors: state.cow.cow_sectors.map((sector) => ({
                    ...sector,
                    bytes: sector.bytes instanceof Uint8Array ? sector.bytes : new Uint8Array(sector.bytes),
                })),
            },
            git: git.serialize(),
        };
        const saved = await (await persistence(key)).save(normalized);
        return snapshotMetadata(saved);
    },
    async loadSnapshot(key, snapshotId, digest) {
        const value = await (await persistence(key)).load({ snapshotId, digest });
        restoredGit = value.git;
        return value;
    },
    async loadCurrentSnapshot(key) {
        const value = await (await persistence(key)).loadOrNull();
        restoredGit = value?.git;
        return value;
    },
    async clearSnapshot(key) {
        await (await persistence(key)).clear();
    },
};

async function ensureWasm(): Promise<void> {
    if (wasmReady === null) {
        wasmReady = initWasm(new URL('../../node_modules/.cache/uor-r4-mvm/uor_r4_mvm_bg.wasm', import.meta.url)).then(() => undefined);
    }
    return wasmReady;
}

async function workerDispatch(kind: MvmKind, payload: Record<string, unknown>): Promise<unknown> {
    const encoded = createMvmRequest({ id: 1, kind, payload });
    const encodedResult = await mvm_dispatch(encoded) as string;
    const result = JSON.parse(encodedResult) as { ok: boolean; result?: unknown; error?: string };
    if (!result.ok) throw new Error(result.error ?? 'MVM request failed');
    return result.result;
}

function workerFiles() {
    return {
        async list(prefix = '/workspace') {
            return await workerDispatch('list', {
                prefix,
                sandboxKey: currentKey,
                persistent: persistentByKey.get(currentKey) ?? true,
            }) as Array<{ path: string; kind: 'file'; sizeBytes: number }>;
        },
        async read(path: string) {
            return await workerDispatch('read', {
                path,
                sandboxKey: currentKey,
                persistent: persistentByKey.get(currentKey) ?? true,
            }) as { text: string; revision: number };
        },
        async write(path: string, text: string) {
            return workerDispatch('write', {
                path,
                text,
                sandboxKey: currentKey,
                persistent: persistentByKey.get(currentKey) ?? true,
            });
        },
        async remove(path: string) {
            if (path.includes("'")) throw new Error('guest path contains an unsupported quote');
            return workerDispatch('exec', {
                cmd: `rm '${path}'`,
                cwd: '/workspace',
                sandboxKey: currentKey,
                persistent: persistentByKey.get(currentKey) ?? true,
            });
        },
    };
}

async function ensureInitialized(key: string, persistent: boolean): Promise<void> {
    if (initializedKey === key) return;
    persistentByKey.set(key, persistent);
    restoredGit = undefined;
    git = new VirtualGit(workerFiles());
    await ensureWasm();
    const encoded = JSON.stringify({ id: 1, kind: 'init', payload: { sandboxKey: key, persistent } });
    const response = JSON.parse(await mvm_dispatch(encoded)) as { ok: boolean; error?: string };
    if (!response.ok) throw new Error(response.error ?? 'MVM initialization failed');
    await git.restore(restoredGit);
    restoredGit = undefined;
    initializedKey = key;
}

self.addEventListener('message', (event: MessageEvent) => {
    void (async () => {
        let id = 1;
        try {
            const encoded = createMvmRequest(event.data);
            const request = JSON.parse(encoded) as { id: number; kind: MvmKind; payload: Record<string, unknown> };
            id = request.id;
            const key = String(request.payload.sandboxKey ?? '');
            const persistent = request.payload.persistent === true;
            currentKey = key;
            await ensureInitialized(key, persistent);
            let result: string;
            if (request.kind === 'exec') {
                const special = await executeVirtualCommand(
                    git,
                    String(request.payload.cmd ?? ''),
                    String(request.payload.cwd ?? '/workspace'),
                );
                if (special.handled) {
                    if (special.persist) {
                        await workerDispatch('snapshot', { sandboxKey: key, persistent });
                        git.markPersisted();
                    }
                    result = JSON.stringify({ id, ok: true, result: special.result });
                } else {
                    result = await mvm_dispatch(encoded);
                }
            } else {
                result = await mvm_dispatch(encoded);
            }
            const parsed = JSON.parse(result) as { ok: boolean };
            if (parsed.ok && request.kind === 'restore') {
                await git.restore(restoredGit);
                restoredGit = undefined;
            }
            if (parsed.ok && request.kind === 'reset') {
                initializedKey = null;
                persistenceByKey.delete(key);
                persistentByKey.delete(key);
                git = new VirtualGit(workerFiles());
                restoredGit = undefined;
            }
            if (parsed.ok && request.kind === 'dispose') {
                git = new VirtualGit(workerFiles());
                restoredGit = undefined;
            }
            self.postMessage(result);
        } catch (error) {
            self.postMessage(JSON.stringify({
                id,
                ok: false,
                error: error instanceof Error ? error.message : String(error),
            }));
        }
    })();
});
