import type { LifecycleStore } from '../state/lifecycle.js';
import type { WorkspaceApi } from './workflow.js';

export async function saveActiveFile(store: LifecycleStore, files: WorkspaceApi, onError: (message: string) => void): Promise<boolean> {
    const path = store.getState().file.path;
    if (!path || !store.canSave()) return false;
    store.dispatch({ type: 'file/saveStarted', path });
    if (store.getState().file.phase !== 'saving') return false;
    try {
        const saved = await files.saveActive();
        if (store.getState().file.path !== path) return saved;
        if (saved) {
            store.dispatch({ type: 'file/saveFinished', path });
        } else {
            store.dispatch({ type: 'file/saveFailed', path, error: 'Save did not complete.' });
            onError('Save did not complete.');
        }
        return saved;
    } catch (error) {
        if (store.getState().file.path === path) {
            const message = error instanceof Error ? error.message : String(error);
            store.dispatch({ type: 'file/saveFailed', path, error: message });
            onError(message);
        }
        return false;
    }
}
