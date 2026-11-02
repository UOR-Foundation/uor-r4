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
        const detail = (event as CustomEvent<{ outcome?: 'complete' | 'stopped' | 'error'; generationId?: number; error?: string }>).detail;
        if (detail?.generationId === undefined) return;
        if (detail.outcome === 'complete' || detail.outcome === 'stopped' || detail.outcome === 'error') {
            store.dispatch({ type: 'generation/finished', outcome: detail.outcome, generationId: detail.generationId, error: detail.error });
        }
    };
    document.addEventListener(GENERATION_FINISHED_EVENT, finished);

    return {
        send(prompt) {
            const selected = store.getState().selectedSource;
            if (!selected || !prompt.trim() || !store.canRun()) return false;
            if (store.getState().session.phase === 'new') store.dispatch({ type: 'session/activated' });
            const prediction = selected.kind === 'artifact'
                && store.getState().residentArtifact?.canGenerate === false
                && store.getState().residentArtifact?.canPredict === true;
            const fail = () => {
                store.dispatch({ type: 'generation/started', source: selected });
                const active = store.getState().session.activeGeneration;
                if (active) store.dispatch({ type: 'generation/finished', outcome: 'error', generationId: active.generationId, error: 'Generation could not be started.' });
                return false;
            };
            try {
                const generationId = prediction ? sessions.predict(prompt) : sessions.sendPrompt(prompt);
                if (typeof generationId !== 'number' || !Number.isInteger(generationId) || generationId <= 0) return fail();
                store.dispatch({ type: 'generation/started', source: selected, generationId });
                return store.getState().session.phase === 'generating';
            } catch {
                return fail();
            }
        },
        stop() {
            const active = store.getState().session.activeGeneration;
            if (active?.canStop !== false) sessions.stopGeneration();
        },
        dispose() {
            document.removeEventListener(GENERATION_FINISHED_EVENT, finished);
        },
    };
}
