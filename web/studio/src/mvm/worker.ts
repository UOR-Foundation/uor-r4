import initWasm, { mvm_dispatch } from '../../node_modules/.cache/uor-r4-mvm/uor_r4_mvm.js';
import { createMvmRequest } from './protocol.js';
import {
    createIdbSnapshotStorage,
    MemorySnapshotStorage,
    MvmPersistence,
    snapshotMetadata,
    type MvmSnapshotInput,
} from './persistence.js';

interface HostSnapshot {
    snapshotId: string;
    digest: string;
}

declare global {
    var __uorMvmHost: {
        saveSnapshot(key: string, state: MvmSnapshotInput): Promise<HostSnapshot>;
        loadSnapshot(key: string, snapshotId: string, digest: string): Promise<unknown>;
        loadCurrentSnapshot(key: string): Promise<unknown>;
    };
}

const persistenceByKey = new Map<string, Promise<MvmPersistence>>();
const persistentByKey = new Map<string, boolean>();
let wasmReady: Promise<void> | null = null;
let initializedKey: string | null = null;

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
        };
        const saved = await (await persistence(key)).save(normalized);
        return snapshotMetadata(saved);
    },
    async loadSnapshot(key, snapshotId, digest) {
        return (await persistence(key)).load({ snapshotId, digest });
    },
    async loadCurrentSnapshot(key) {
        return (await persistence(key)).loadOrNull();
    },
};

async function ensureWasm(): Promise<void> {
    if (wasmReady === null) {
        wasmReady = initWasm(new URL('../../node_modules/.cache/uor-r4-mvm/uor_r4_mvm_bg.wasm', import.meta.url)).then(() => undefined);
    }
    return wasmReady;
}

async function ensureInitialized(key: string, persistent: boolean): Promise<void> {
    if (initializedKey === key) return;
    persistentByKey.set(key, persistent);
    await ensureWasm();
    const encoded = JSON.stringify({ id: 1, kind: 'init', payload: { sandboxKey: key, persistent } });
    const response = JSON.parse(await mvm_dispatch(encoded)) as { ok: boolean; error?: string };
    if (!response.ok) throw new Error(response.error ?? 'MVM initialization failed');
    initializedKey = key;
}

self.addEventListener('message', (event: MessageEvent) => {
    void (async () => {
        let id = 1;
        try {
            const encoded = createMvmRequest(event.data);
            const request = JSON.parse(encoded) as { id: number; kind: string; payload: Record<string, unknown> };
            id = request.id;
            const key = String(request.payload.sandboxKey ?? '');
            const persistent = request.payload.persistent === true;
            await ensureInitialized(key, persistent);
            const result = await mvm_dispatch(encoded);
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
