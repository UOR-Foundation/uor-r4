import { useEffect, useRef, useState } from 'preact/hooks';
import type { DesktopActions } from '../adapters/desktopActions.js';
import { desktopRepositoryPath } from '../adapters/desktopActions.js';
import type { WorkspaceApi } from '../adapters/workflow.js';
import { AgentClientError, type AgentClient, type ChangesSnapshot } from '../agent/client.js';
import type { DiffCommentRecord } from '../agent/store.js';
import { DiffReview, createUploadedSnapshot, type DiffComment, type DiffCommentDraft, type DiffSource, type DiffScope } from './DiffReview.js';
import { Icon } from './ui/Icon.js';

interface DesktopPanelProps { actions: DesktopActions; files: WorkspaceApi; idPrefix?: string }
interface ChangesPanelProps extends DesktopPanelProps {
    copy(text: string): void | Promise<void>;
    agentClient?: AgentClient | null;
    taskId?: string | null;
}

export function useWorkspacePath(files: WorkspaceApi) {
    const [path, setPath] = useState(() => desktopRepositoryPath(files));
    const [revision, setRevision] = useState(0);
    useEffect(() => {
        const changed = (event: Event) => { const rootPath = String((event as CustomEvent<{ rootPath?: string }>).detail?.rootPath || ''); setPath(rootPath); setRevision((value) => value + 1); };
        document.addEventListener('uor-files-changed', changed);
        return () => document.removeEventListener('uor-files-changed', changed);
    }, []);
    return { path, revision, updatePath: (value: string) => { setPath(value); setRevision((current) => current + 1); } };
}

function commentFromRecord(record: DiffCommentRecord): DiffComment {
    return {
        commentId: record.comment_id,
        taskId: record.task_id,
        scope: record.scope,
        path: record.path,
        line: record.line,
        body: record.body,
        headDigest: record.head_digest,
        createdAtMs: record.created_at_ms,
    };
}

export function ChangesPanel({ actions, files, copy, idPrefix = '', agentClient = null, taskId = null }: ChangesPanelProps) {
    const { path, revision, updatePath } = useWorkspacePath(files);
    const latestRevision = useRef(revision); latestRevision.current = revision;
    const operation = useRef(0); const [refreshing, setRefreshing] = useState(false); const [summary, setSummary] = useState(''); const [diff, setDiff] = useState(''); const [displayDiff, setDisplayDiff] = useState(''); const diffRef = useRef('');
    const [scope, setScope] = useState<DiffScope>(taskId === null ? 'branch' : 'last_turn');
    const [uploaded, setUploaded] = useState<ChangesSnapshot | null>(null);
    const [snapshot, setSnapshot] = useState<ChangesSnapshot | null>(null);
    const [source, setSource] = useState<DiffSource>(null);
    const [reviewLoading, setReviewLoading] = useState(false);
    const [reviewError, setReviewError] = useState<string | null>(null);
    const [comments, setComments] = useState<DiffComment[]>([]);
    const [commentError, setCommentError] = useState<string | null>(null);
    const [commentRevision, setCommentRevision] = useState(0);
    const reviewOperation = useRef(0);
    const id = (name: string) => `${idPrefix}${name}`;
    useEffect(() => { operation.current += 1; setRefreshing(false); setSummary(''); setDiff(''); setDisplayDiff(''); diffRef.current = ''; return () => { operation.current += 1; }; }, [revision]);

    useEffect(() => {
        const request = ++reviewOperation.current;
        if (scope === 'branch' && uploaded !== null) {
            setSnapshot(uploaded);
            setSource('upload');
            setReviewError(null);
            setReviewLoading(false);
            return () => { reviewOperation.current += 1; };
        }
        if (agentClient === null || taskId === null) {
            setSnapshot(null);
            setSource(null);
            setReviewError(null);
            setReviewLoading(false);
            return () => { reviewOperation.current += 1; };
        }
        setReviewLoading(true);
        setReviewError(null);
        void (async () => {
            try {
                const result = await agentClient.changesSnapshot({ taskId, scope });
                if (request !== reviewOperation.current) return;
                setSnapshot(result);
                setSource(scope === 'last_turn' ? 'browser' : 'host');
            } catch (error) {
                if (request !== reviewOperation.current) return;
                setSnapshot(null);
                setSource(null);
                setReviewError(
                    error instanceof AgentClientError && error.code === 'host_unavailable'
                        ? 'Branch changes require the local companion. Upload a diff to review one locally.'
                        : `Changes could not be loaded: ${error instanceof Error ? error.message : String(error)}`,
                );
            } finally {
                if (request === reviewOperation.current) setReviewLoading(false);
            }
        })();
        return () => { reviewOperation.current += 1; };
    }, [agentClient, taskId, scope, uploaded]);

    useEffect(() => {
        let active = true;
        if (agentClient === null || taskId === null) {
            setComments([]);
            setCommentError(null);
            return () => { active = false; };
        }
        void agentClient.readDiffComments(taskId).then(
            (records) => {
                if (!active) return;
                setComments(records.map(commentFromRecord));
                setCommentError(null);
            },
            (error: unknown) => {
                if (!active) return;
                setComments([]);
                setCommentError(`Local comments could not be loaded: ${error instanceof Error ? error.message : String(error)}`);
            },
        );
        return () => { active = false; };
    }, [agentClient, taskId, commentRevision]);

    async function refresh() {
        const repoPath = path.trim(); if (!repoPath || !actions.available || refreshing) return;
        const request = ++operation.current; const startedRevision = latestRevision.current; setRefreshing(true);
        try {
            const [status, report] = await Promise.all([actions.gitStatus(repoPath), actions.gitDiff(repoPath)]);
            if (request !== operation.current || startedRevision !== latestRevision.current) return;
            const changedFiles = (status.modified_files?.length || 0) + (status.untracked_files?.length || 0);
            const limits = [status.truncated || report.truncated ? 'truncated' : '', status.timed_out || report.timed_out ? 'timed out' : ''].filter(Boolean).join(' · ');
            setSummary(`${status.branch} · ${changedFiles} changed file${changedFiles === 1 ? '' : 's'}${limits ? ` · ${limits}` : ''}`);
            const rawDiff = String(report.diff || ''); diffRef.current = rawDiff; setDiff(rawDiff); setDisplayDiff(rawDiff + (report.truncated ? '\n[diff truncated]' : '') + (report.timed_out ? '\n[diff timed out]' : ''));
        } catch (error) {
            if (request !== operation.current || startedRevision !== latestRevision.current) return;
            setSummary(`Git inspection failed: ${String(error instanceof Error ? error.message : error)}`); setDiff(''); setDisplayDiff('No diff available.'); diffRef.current = '';
        } finally { if (request === operation.current) setRefreshing(false); }
    }
    async function copyDiff() { if (!diffRef.current) return; try { await copy(diffRef.current); } catch (error) { setSummary(`Could not copy the diff: ${String(error instanceof Error ? error.message : error)}`); } }
    async function upload(input: HTMLInputElement) {
        const selected = input.files === null ? [] : Array.from(input.files);
        input.value = '';
        if (selected.length === 0) return;
        try {
            const next = await createUploadedSnapshot(selected);
            setUploaded(next);
            setScope('branch');
        } catch (error) {
            setReviewError(`The uploaded file could not be read: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
    async function addComment(draft: DiffCommentDraft): Promise<void> {
        if (agentClient === null || taskId === null) throw new Error('Select a browser task before adding a local comment.');
        if (snapshot === null) throw new Error('No current snapshot is loaded; reload changes before commenting.');
        const record = await agentClient.addDiffComment({
            taskId,
            scope,
            path: draft.path,
            line: draft.line,
            body: draft.body,
            headDigest: snapshot.headDigest,
        });
        const saved = commentFromRecord(record);
        setComments((current) => [
            ...current.filter((item) => !(item.scope === saved.scope && item.path === saved.path && item.line === saved.line && item.headDigest === saved.headDigest)),
            saved,
        ]);
        setCommentError(null);
        setCommentRevision((value) => value + 1);
    }

    return <section aria-label="Changes" class="desktop-panel changes-panel"><header class="panel-heading"><div><h1>Changes</h1><p>Review the working tree before choosing what to ship.</p></div><span class="panel-icon"><Icon name="changes" /></span></header>
        <section class="changes-review" aria-label="Task changes review">
            <div class="diff-upload">
                <label for={id('diffUpload')}>Upload a diff or text file to review locally</label>
                <input id={id('diffUpload')} type="file" multiple accept=".diff,.patch,.txt,.md,text/plain" onChange={(event) => { void upload(event.currentTarget); }} />
                {uploaded !== null && <button id={id('btnClearUpload')} type="button" onClick={() => setUploaded(null)}>Remove uploaded diff</button>}
            </div>
            {commentError !== null && <p class="panel-summary" role="alert">{commentError}</p>}
            <DiffReview
                taskId={taskId}
                scope={scope}
                onScope={setScope}
                comments={comments}
                onComment={addComment}
                snapshot={snapshot}
                source={source}
                loading={reviewLoading}
                error={reviewError}
                commentDisabledReason={agentClient === null || taskId === null ? 'Select a browser task to add a local comment.' : null}
                idPrefix={id('diffReview-')}
            />
        </section>
        <p id={id('gitStatusSummary')} class="panel-summary">{summary || (actions.available ? (path.trim() ? 'Refresh to inspect repository changes.' : 'Open a project folder in Files before inspecting changes.') : 'Git requires the local companion. Uploaded files can still be reviewed locally.')}</p>
        <label>Repository path<input id={id('terminalCwd')} aria-label="Repository path" value={path} onInput={(event) => updatePath(event.currentTarget.value)} /></label>
        <div class="panel-actions"><button id={id('btnRefreshChanges')} type="button" disabled={!actions.available || refreshing || !path.trim()} onClick={() => void refresh()}>{refreshing ? 'Refreshing…' : 'Refresh changes'}</button><button id={id('btnCopyDiff')} type="button" disabled={!diff} onClick={() => void copyDiff()}>Copy diff</button></div>
        <pre id={id('gitDiffOutput')} class="desktop-output" aria-label="Git diff">{displayDiff}</pre></section>;
}

export { TerminalPanel } from './TerminalSession.js';
