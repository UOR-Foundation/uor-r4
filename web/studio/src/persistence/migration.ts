export const MIGRATION_KEYS = [
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
] as const;

export type MigrationKey = typeof MIGRATION_KEYS[number];

export interface MigrationEntry {
    key: MigrationKey;
    sourceKey: MigrationKey;
    raw: string | null;
    value: unknown;
    error: string | null;
    destinationKey: string | null;
    migratable: boolean;
}

export interface MigrationStorage {
    getItem(key: string): string | null;
}

const DESTINATION_KEYS: Record<Exclude<MigrationKey, 'uor.studio.modelSources.v1'>, string> = {
    'uor.studio.conversations.v1': 'uor.studio.lifecycle.conversations.v2',
    'uor.studio.activeConversation.v1': 'uor.studio.lifecycle.activeConversation.v2',
    'uor.studio.editor.v1': 'uor.studio.lifecycle.editor.v2',
    'uor.studio.settings.v1': 'uor.studio.lifecycle.settings.v2',
    'uor.studio.ui.v2': 'uor.studio.lifecycle.ui.v3',
    'uor.studio.ui.v1': 'uor.studio.lifecycle.ui.v2',
    'uor.studio.attachments.v1': 'uor.studio.lifecycle.attachments.v2',
    'uor.studio.tabs.v2': 'uor.studio.lifecycle.tabs.v3',
    'uor.studio.recent.v2': 'uor.studio.lifecycle.recent.v3',
    'uor.studio.theme': 'uor.studio.lifecycle.theme.v1',
};

function isLiveCompatibilityKey(key: MigrationKey): boolean {
    return key === 'uor.studio.modelSources.v1';
}

export function readMigrationSnapshot(storage: MigrationStorage): MigrationEntry[] {
    return MIGRATION_KEYS.map((key) => {
        const raw = storage.getItem(key);
        if (raw === null) {
            return {
                key,
                sourceKey: key,
                raw: null,
                value: null,
                error: null,
                destinationKey: null,
                migratable: !isLiveCompatibilityKey(key),
            };
        }
        try {
            return {
                key,
                sourceKey: key,
                raw,
                value: JSON.parse(raw) as unknown,
                error: null,
                destinationKey: null,
                migratable: !isLiveCompatibilityKey(key),
            };
        } catch {
            return {
                key,
                sourceKey: key,
                raw,
                value: null,
                error: 'Invalid JSON',
                destinationKey: null,
                migratable: !isLiveCompatibilityKey(key),
            };
        }
    });
}

export function createCopyForwardPlan(snapshot: MigrationEntry[]): MigrationEntry[] {
    return snapshot.map((entry) => ({
        ...entry,
        value: entry.value === undefined ? null : structuredClone(entry.value),
        destinationKey: entry.migratable
            ? DESTINATION_KEYS[entry.sourceKey as Exclude<MigrationKey, 'uor.studio.modelSources.v1'>]
            : null,
    }));
}
