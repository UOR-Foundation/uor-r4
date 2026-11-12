import { describe, expect, it } from 'vitest';

import {
    acquireSandboxKey,
    MemorySnapshotStorage,
    MvmPersistence,
    snapshotMetadata,
    type MvmSnapshotInput,
} from './persistence.js';

const DIGEST_A = `sha256:${'a'.repeat(64)}`;

function snapshot(): MvmSnapshotInput {
    return {
        version: 1,
        snapshotId: DIGEST_A,
        cwd: '/workspace',
        cowDigest: DIGEST_A,
        files: { 'src/a.c': { text: 'int main(void){return 0;}', revision: 3 } },
        sectors: [],
    };
}

function storage() {
    return new MemorySnapshotStorage();
}

describe('MVM persistence', () => {
    it('mvm_cow_state_survives_reload', async () => {
        const backend = storage();
        const first = new MvmPersistence(backend, 'tab-a');
        const saved = await first.save(snapshot());
        expect(saved.snapshotId).toBe(DIGEST_A);
        expect(saved.digest).toMatch(/^sha256:[0-9a-f]{64}$/);
        const second = new MvmPersistence(backend, 'tab-a');
        const restored = await second.load({ snapshotId: saved.snapshotId, digest: saved.digest });
        expect(restored.files['src/a.c']).toEqual({ text: 'int main(void){return 0;}', revision: 3 });
        expect(restored.digest).toBe(saved.digest);
    });

    it('mvm_corrupt_quota_partial_and_unknown_version_are_refused', async () => {
        const backend = storage();
        const persistence = new MvmPersistence(backend, 'tab-a');
        const saved = await persistence.save(snapshot());
        backend.seed('tab-a', { ...snapshot(), digest: saved.digest, version: 2 });
        await expect(persistence.load()).rejects.toThrow(/version/i);
        expect(backend.writeCount).toBe(1);

        backend.seed('tab-a', { ...snapshot(), digest: saved.digest, files: { 'src/a.c': { text: 'tampered', revision: 3 } } });
        await expect(persistence.load()).rejects.toThrow(/digest|corrupt/i);
        await expect(persistence.save(snapshot())).rejects.toThrow(/digest|corrupt/i);
        expect(backend.writeCount).toBe(1);

        const valid = new MvmPersistence(storage(), 'tab-a');
        valid.storage.failNextPut(new DOMException('full', 'QuotaExceededError'));
        await expect(valid.save(snapshot())).rejects.toThrow(/quota|write|full/i);
        expect(valid.storage.writeCount).toBe(0);
    });

    it('duplicated_tab_gets_distinct_sandbox_or_refuses', async () => {
        const session = new Map([['uor-r4.studio.mvm.tab', 'duplicated-key']]);
        const locks = {
            request: async (_name: string, _options: unknown, callback: (lock: null) => unknown) => callback(null),
        };
        const sessionStorage = {
            getItem: (key: string) => session.get(key) ?? null,
            setItem: (key: string, value: string) => { session.set(key, value); },
        };
        await expect(acquireSandboxKey({ sessionStorage, locks: locks as never })).rejects.toThrow(/another tab|ownership/i);
        const fresh = await acquireSandboxKey({ sessionStorage, locks: locks as never, freshOnConflict: true, randomUUID: () => 'fresh-key' });
        expect(fresh.key).toBe('fresh-key');
        expect(fresh.key).not.toBe('duplicated-key');
        await fresh.release();
    });

    it('mvm_snapshot_restore_respects_message_cap_at_256_sectors', async () => {
        const backend = storage();
        const persistence = new MvmPersistence(backend, 'tab-a');
        const value = snapshot();
        value.sectors = Array.from({ length: 256 }, (_, id) => ({ id, bytes: new Uint8Array(65_536).fill(id % 251) }));
        const saved = await persistence.save(value);
        const metadata = snapshotMetadata(saved);
        expect(new TextEncoder().encode(JSON.stringify(metadata)).byteLength).toBeLessThan(2 * 1024 * 1024);
        expect(metadata).toEqual({ snapshotId: saved.snapshotId, digest: saved.digest });
        const restored = await persistence.load(metadata);
        expect(restored.sectors).toHaveLength(256);
        expect(restored.sectors[255].bytes).toHaveLength(65_536);
    });

    it('mvm_history_and_credentials_are_not_persisted', async () => {
        const backend = storage();
        const persistence = new MvmPersistence(backend, 'tab-a');
        const value = snapshot();
        const saved = await persistence.save(value);
        const raw = JSON.stringify(backend.readRaw('tab-a'));
        expect(raw).not.toContain('history');
        expect(raw).not.toContain('GITHUB_TOKEN');
        expect(raw).not.toContain('Authorization');
        expect(saved.digest).not.toBe(DIGEST_A);
    });
});


describe('MVM persistence representation guard', () => {
    it('mvm_map_and_set_are_refused_not_silently_dropped', async () => {
        const persistence = new MvmPersistence(storage(), 'tab-a');
        const value = snapshot();
        value.cow = {
            inodes: new Map([['/workspace/a.txt', { mode: 0o100644, kind: 'regular' }]]) as never,
            dirs: ['/'],
            cow_sectors: [],
            dirty_sectors: [],
            next_sector: 0,
            next_pid: 100,
        };
        await expect(persistence.save(value)).rejects.toThrow(/plain objects|Map or Set/i);
    });
});
