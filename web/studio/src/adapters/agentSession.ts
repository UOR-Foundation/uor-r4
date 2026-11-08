import type { LifecycleStore, SessionSummary } from '../state/lifecycle.js';
import { createGenerationAdapter } from './generation.js';
import type { SessionApi } from './workflow.js';

interface SessionSnapshot {
    activeId: string | null;
    items: SessionSummary[];
}

export interface AgentSessionController {
    refresh(): boolean;
    create(): SessionSummary | null;
    resume(id: string): SessionSummary | null;
    rename(id: string, title: string): boolean;
    duplicate(id: string): SessionSummary | null;
    remove(id: string): SessionSummary | null;
    export(id: string): { name: string; content: string } | null;
    send(prompt: string): boolean;
    stop(): void;
    dispose(): void;
}

export function createAgentSessionController(store: LifecycleStore, sessions: SessionApi): AgentSessionController {
    const generation = createGenerationAdapter(store, sessions);

    function read(): SessionSnapshot | null {
        try {
            const items = sessions.listSessions().map((item) => ({ ...item }));
            const activeId = sessions.activeSessionId();
            if (activeId !== null && !items.some((item) => item.id === activeId)) return null;
            return { activeId, items };
        } catch {
            return null;
        }
    }

    function refresh(): boolean {
        const snapshot = read();
        if (!snapshot) return false;
        const active = store.getState().session.activeGeneration;
        if (active && active.sessionId !== snapshot.activeId) {
            try {
                sessions.stopGeneration();
                store.dispatch({ type: 'session/interrupted', activeId: snapshot.activeId, items: snapshot.items, generationId: active.generationId });
            } catch {
                return false;
            }
            return store.getState().session.activeGeneration === null && store.getState().session.activeId === snapshot.activeId;
        }
        store.dispatch({ type: 'session/synchronized', activeId: snapshot.activeId, items: snapshot.items });
        return store.getState().session.activeId === snapshot.activeId;
    }

    function verifiedActive(expectedId?: string): SessionSummary | null {
        const snapshot = read();
        if (!snapshot?.activeId || (expectedId !== undefined && snapshot.activeId !== expectedId)) return null;
        const active = snapshot.items.find((item) => item.id === snapshot.activeId);
        if (!active) return null;
        store.dispatch({ type: 'session/synchronized', activeId: snapshot.activeId, items: snapshot.items });
        return store.getState().session.activeId === active.id ? active : null;
    }

    function create(): SessionSummary | null {
        if (store.getState().session.phase === 'generating') return null;
        try {
            return sessions.newSession() ? verifiedActive() : null;
        } catch {
            return null;
        }
    }

    function resume(id: string): SessionSummary | null {
        if (store.getState().session.phase === 'generating') return null;
        try {
            return sessions.selectSession(id) ? verifiedActive(id) : null;
        } catch {
            return null;
        }
    }

    function rename(id: string, title: string): boolean {
        if (store.getState().session.phase === 'generating') return false;
        try {
            if (!sessions.renameSession(id, title)) return false;
            const snapshot = read();
            if (!snapshot || snapshot.items.find((item) => item.id === id)?.title !== title) return false;
            store.dispatch({ type: 'session/synchronized', activeId: snapshot.activeId, items: snapshot.items });
            return true;
        } catch {
            return false;
        }
    }

    function duplicate(id: string): SessionSummary | null {
        if (store.getState().session.phase === 'generating') return null;
        try {
            const duplicateId = sessions.duplicateSession(id);
            if (!duplicateId) return null;
            const snapshot = read();
            const duplicate = snapshot?.items.find((item) => item.id === duplicateId);
            if (!snapshot || !duplicate) return null;
            store.dispatch({ type: 'session/synchronized', activeId: snapshot.activeId, items: snapshot.items });
            return duplicate;
        } catch {
            return null;
        }
    }

    function remove(id: string): SessionSummary | null {
        if (store.getState().session.phase === 'generating') return null;
        try {
            return sessions.deleteSession(id) ? verifiedActive() : null;
        } catch {
            return null;
        }
    }

    function exportSession(id: string): { name: string; content: string } | null {
        try {
            const item = read()?.items.find((candidate) => candidate.id === id);
            if (!item) return null;
            const content = sessions.exportSession(id);
            return content ? { name: `${item.title}.json`, content } : null;
        } catch {
            return null;
        }
    }

    refresh();

    return {
        refresh,
        create,
        resume,
        rename,
        duplicate,
        remove,
        export: exportSession,
        send(prompt) {
            if (!prompt.trim() || !store.canRun()) return false;
            if (sessions.activeSessionId() === null) {
                if (!create()) return false;
            } else if (!refresh()) {
                return false;
            }
            const sent = generation.send(prompt);
            refresh();
            return sent;
        },
        stop() {
            generation.stop();
            refresh();
        },
        dispose() {
            generation.dispose();
        },
    };
}
