import { useState } from 'preact/hooks';
import type { SessionApi, SessionSummary } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { Modal } from './Modal.js';

interface SessionListProps {
    store: LifecycleStore;
    sessions: SessionApi;
    download(name: string, content: string): void;
    onError(message: string): void;
    onChange?(): void;
}

export function SessionList({ store, sessions, download, onError, onChange }: SessionListProps) {
    const [, setRevision] = useState(0);
    const [renaming, setRenaming] = useState<SessionSummary | null>(null);
    const [deleting, setDeleting] = useState<SessionSummary | null>(null);
    const [title, setTitle] = useState('');
    const refresh = () => setRevision((value) => value + 1);
    const activate = () => { store.dispatch({ type: 'session/activated' }); refresh(); onChange?.(); };
    const items = sessions.listSessions();

    function rename(session: SessionSummary) {
        setRenaming(session);
        setTitle(session.title);
    }

    return (
        <section aria-label="Sessions">
            <h2>Sessions</h2>
            <button type="button" onClick={() => { if (sessions.newSession()) activate(); else onError('Could not create a session.'); }}>New session</button>
            {items.length === 0 && <p>No sessions yet</p>}
            <ul>
                {items.map((session) => (
                    <li>
                        <strong>{session.title}</strong>
                        <span>{session.messageCount} messages · {session.preview}</span>
                        <button type="button" onClick={() => { if (sessions.selectSession(session.id)) activate(); else onError('Could not select that session.'); }}>{`Select ${session.title}`}</button>
                        <button type="button" onClick={() => rename(session)}>{`Rename ${session.title}`}</button>
                        <button type="button" onClick={() => { sessions.duplicateSession(session.id); refresh(); }}>{`Duplicate ${session.title}`}</button>
                        <button type="button" onClick={() => {
                            const exported = sessions.exportSession(session.id);
                            if (exported) download(`${session.title}.json`, exported);
                            else onError('Session export is unavailable.');
                        }}>{`Export ${session.title}`}</button>
                        <button type="button" onClick={() => setDeleting(session)}>{`Delete ${session.title}`}</button>
                    </li>
                ))}
            </ul>
            {renaming && (
                <Modal title="Rename session" onClose={() => setRenaming(null)}>
                    <form onSubmit={(event) => {
                        event.preventDefault();
                        if (title.trim()) sessions.renameSession(renaming.id, title.trim());
                        setRenaming(null);
                        refresh();
                    }}>
                        <label>Session title<input aria-label="Session title" autofocus value={title} onInput={(event) => setTitle(event.currentTarget.value)} /></label>
                        <button type="submit">Save title</button>
                    </form>
                </Modal>
            )}
            {deleting && (
                <Modal title="Delete session" onClose={() => setDeleting(null)}>
                    <p>Delete {deleting.title}? This affects only the selected local session.</p>
                    <button type="button" autofocus onClick={() => setDeleting(null)}>Keep session</button>
                    <button type="button" onClick={() => {
                        if (sessions.deleteSession(deleting.id)) {
                            setDeleting(null);
                            activate();
                        } else {
                            onError('Could not delete that session. Stop the current reply first.');
                        }
                    }}>Confirm delete</button>
                </Modal>
            )}
        </section>
    );
}
