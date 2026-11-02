import { describe, expect, it, vi } from 'vitest';
import { createCopyForwardPlan, MIGRATION_KEYS, readMigrationSnapshot } from './migration.js';

const ALL_KEYS = [
    'uor.studio.conversations.v1',
    'uor.studio.activeConversation.v1',
    'uor.studio.editor.v1',
    'uor.studio.modelSources.v1',
    'uor.studio.settings.v1',
    'uor.studio.ui.v2',
    'uor.studio.ui.v1',
    'uor.studio.attachments.v1',
    'uor.studio.tabs.v2',
    'uor.studio.recent.v2',
    'uor.studio.theme',
];

function storageWith(entries: Record<string, string>) {
    const values = new Map(Object.entries(entries));
    return {
        getItem: vi.fn((key: string) => values.has(key) ? values.get(key)! : null),
        setItem: vi.fn(),
    };
}

describe('migration snapshot', () => {
    it('inventories every current and legacy key', () => {
        expect(MIGRATION_KEYS).toEqual(ALL_KEYS);
        expect(new Set(MIGRATION_KEYS).size).toBe(ALL_KEYS.length);
    });

    it('reads each key once and tolerates unknown fields', () => {
        const storage = storageWith({
            'uor.studio.settings.v1': JSON.stringify({ theme: 'dark', futureSetting: { nested: true } }),
            'uor.studio.ui.v2': JSON.stringify({ nav: 'collapsed', futureFlag: 42 }),
        });
        const snapshot = readMigrationSnapshot(storage);
        expect(storage.getItem).toHaveBeenCalledTimes(ALL_KEYS.length);
        for (const key of ALL_KEYS) expect(storage.getItem).toHaveBeenCalledWith(key);
        expect(snapshot.find((entry) => entry.key === 'uor.studio.settings.v1')?.value).toEqual({ theme: 'dark', futureSetting: { nested: true } });
        expect(snapshot.find((entry) => entry.key === 'uor.studio.ui.v2')?.value).toEqual({ nav: 'collapsed', futureFlag: 42 });
        expect(storage.setItem).not.toHaveBeenCalled();
    });

    it('preserves malformed raw input without throwing or logging its contents', () => {
        const sentinel = 'sentinel-not-a-secret';
        const storage = storageWith({ 'uor.studio.settings.v1': `{not-json-${sentinel}` });
        const snapshot = readMigrationSnapshot(storage);
        const entry = snapshot.find((item) => item.key === 'uor.studio.settings.v1');
        expect(entry?.raw).toBe(`{not-json-${sentinel}`);
        expect(entry?.value).toBeNull();
        expect(entry?.error).toBe('Invalid JSON');
        expect(entry?.error).not.toContain(sentinel);
    });

    it('represents missing values without an error', () => {
        const snapshot = readMigrationSnapshot(storageWith({}));
        expect(snapshot).toHaveLength(ALL_KEYS.length);
        for (const entry of snapshot) {
            expect(entry.raw).toBeNull();
            expect(entry.value).toBeNull();
            expect(entry.error).toBeNull();
        }
    });
});

describe('copy-forward plan', () => {
    it('is deterministic and idempotent', () => {
        const storage = storageWith({ 'uor.studio.theme': 'dark' });
        const snapshot = readMigrationSnapshot(storage);
        expect(createCopyForwardPlan(snapshot)).toEqual(createCopyForwardPlan(snapshot));
        expect(storage.getItem).toHaveBeenCalledTimes(ALL_KEYS.length);
        expect(storage.setItem).not.toHaveBeenCalled();
    });

    it('preserves raw rollback data for migratable keys', () => {
        const raw = JSON.stringify({ title: 'Draft', futureField: [1, 2, 3] });
        const plan = createCopyForwardPlan(readMigrationSnapshot(storageWith({ 'uor.studio.conversations.v1': raw })));
        const entry = plan.find((item) => item.sourceKey === 'uor.studio.conversations.v1');
        expect(entry?.raw).toBe(raw);
        expect(entry?.value).toEqual({ title: 'Draft', futureField: [1, 2, 3] });
        expect(entry?.destinationKey).not.toBe('uor.studio.conversations.v1');
    });

    it('never creates a second provider-secret store or migration marker', () => {
        const raw = JSON.stringify({ active: { kind: 'api', id: 'remote' }, api: [{ id: 'remote', apiKey: 'sentinel-not-a-secret' }] });
        const plan = createCopyForwardPlan(readMigrationSnapshot(storageWith({ 'uor.studio.modelSources.v1': raw })));
        const live = plan.find((entry) => entry.sourceKey === 'uor.studio.modelSources.v1');
        expect(live?.destinationKey).toBeNull();
        expect(live?.migratable).toBe(false);
        expect(plan.filter((entry) => entry.destinationKey !== null).map((entry) => entry.destinationKey)).not.toContain('uor.studio.modelSources.v1');
        expect(plan.every((entry) => !('migrationMarker' in entry))).toBe(true);
        expect(plan.every((entry) => !('payload' in entry))).toBe(true);
        expect(JSON.stringify(plan)).not.toContain('sentinel-not-a-secret');
    });

    it('preserves the raw theme as a valid value and plan', () => {
        const snapshot = readMigrationSnapshot(storageWith({ 'uor.studio.theme': 'dark' }));
        const entry = snapshot.find((item) => item.key === 'uor.studio.theme');
        expect(entry?.value).toBe('dark');
        expect(entry?.error).toBeNull();
        const plan = createCopyForwardPlan(snapshot);
        expect(plan.find((item) => item.sourceKey === 'uor.studio.theme')?.value).toBe('dark');
    });

    it('marks malformed entries non-copyable instead of erasing them', () => {
        const snapshot = readMigrationSnapshot(storageWith({ 'uor.studio.settings.v1': '{bad-json' }));
        const plan = createCopyForwardPlan(snapshot);
        const entry = plan.find((item) => item.sourceKey === 'uor.studio.settings.v1');
        expect(entry?.migratable).toBe(false);
        expect(entry?.destinationKey).toBeNull();
        expect(entry?.raw).toBe('{bad-json');
    });
});
