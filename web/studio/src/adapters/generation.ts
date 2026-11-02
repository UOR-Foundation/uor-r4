import type { LifecycleStore } from '../state/lifecycle.js';
import type { SessionApi } from './workflow.js';

export const GENERATION_FINISHED_EVENT = 'uor-generation-finished';

export interface GenerationAdapter {
    send(prompt: string): boolean;
    stop(): void;
    dispose(): void;
}

export function createGenerationAdapter(store: LifecycleStore, sessions: SessionApi): GenerationAdapter {
    const finished = (event: Event) => {
        const detail = (event as CustomEvent<{ outcome?: 'complete' | 'stopped' | 'error' }>).detail;
        if (detail?.outcome === 'complete' || detail?.outcome === 'stopped' || detail?.outcome === 'error') {
            store.dispatch({ type: 'generation/finished', outcome: detail.outcome });
        }
    };
    document.addEventListener(GENERATION_FINISHED_EVENT, finished);

    return {
        send(prompt) {
            const selected = store.getState().selectedSource;
            if (!selected || !prompt.trim() || !store.canRun()) return false;
            if (store.getState().session.phase === 'new') store.dispatch({ type: 'session/activated' });
            store.dispatch({ type: 'generation/started', source: selected });
            if (store.getState().session.phase !== 'generating') return false;
            try {
                const accepted = sessions.sendPrompt(prompt);
                if (!accepted) store.dispatch({ type: 'generation/finished', outcome: 'error' });
                return accepted;
            } catch {
                store.dispatch({ type: 'generation/finished', outcome: 'error' });
                return false;
            }
        },
        stop() {
            if (store.getState().session.phase === 'generating') sessions.stopGeneration();
        },
        dispose() {
            document.removeEventListener(GENERATION_FINISHED_EVENT, finished);
        },
    };
}
