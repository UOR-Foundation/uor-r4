import { describe, expect, it } from 'vitest';

import {
    acquireSandboxKey,
    MemorySnapshotStorage,
    MvmPersistence,
    snapshotMetadata,
    type MvmSnapshotInput,
} from './persistence.js';
import { sha256Text, virtualCommitHash, type VirtualGitSnapshot } from './virtualGit.js';

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

async function validGitSnapshot(): Promise<VirtualGitSnapshot> {
    const text = 'int main(void){return 0;}';
    const blob = await sha256Text(text);
    const body = {
        parents: [],
        message: 'initial',
        author: 'Test <test@invalid>',
        timestamp: '2026-01-02T03:04:05.000Z',
        tree: { '/workspace/src/a.c': blob },
    };
    const hash = await virtualCommitHash(body);
    return {
        version: 1,
        initialized: true,
        currentBranch: 'main',
        branches: { main: hash },
        commits: { [hash]: { hash, ...body } },
        staged: {},
        blobs: { [blob]: text },
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

    it('legacy_snapshot_without_git_keeps_its_original_digest', async () => {
        const backend = storage();
        const value = snapshot();
        const header = JSON.stringify({
            version: value.version,
            snapshotId: value.snapshotId,
            cwd: value.cwd,
            cowDigest: value.cowDigest,
            files: Object.entries(value.files),
            sectors: [],
            cow: value.cow,
        });
        const digestBytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(header));
        const digest = `sha256:${[...new Uint8Array(digestBytes)].map((byte) => byte.toString(16).padStart(2, '0')).join('')}`;
        backend.seed('tab-a', { ...value, digest });
        const persistence = new MvmPersistence(backend, 'tab-a');
        const restored = await persistence.load();
        expect(restored.digest).toBe(digest);
        expect(restored.git).toBeUndefined();
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

    it('git_state_round_trip_and_corrupt_history_are_verified', async () => {
        const backend = storage();
        const persistence = new MvmPersistence(backend, 'tab-a');
        const value = snapshot();
        value.git = await validGitSnapshot();
        const saved = await persistence.save(value);
        const restored = await persistence.load(snapshotMetadata(saved));
        expect(restored.git?.currentBranch).toBe('main');
        expect(Object.keys(restored.git?.commits ?? {})).toHaveLength(1);

        const corrupt = snapshot();
        corrupt.git = await validGitSnapshot();
        const hash = corrupt.git.branches.main!;
        corrupt.git.commits[hash]!.parents = [`sha256:${'f'.repeat(64)}`];
        const writes = backend.writeCount;
        await expect(persistence.save(corrupt)).rejects.toThrow(/parent|history/i);
        expect(backend.writeCount).toBe(writes);

        const badBlob = snapshot();
        badBlob.git = await validGitSnapshot();
        const blob = badBlob.git.commits[hash]!.tree['/workspace/src/a.c']!;
        badBlob.git.blobs[blob] = 'tampered';
        await expect(persistence.save(badBlob)).rejects.toThrow(/blob|hash|digest/i);
        expect(backend.writeCount).toBe(writes);
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
