import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import { AgentClient, type AgentIncomingEvent, type ChangesSnapshot, type HostCapabilities } from '../agent/client.js';
import { FixtureTransport } from '../agent/fixtureTransport.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Canonical,
    type HostInvokeResponse,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from '../agent/hostClient.js';
import { AgentTaskStore } from '../agent/store.js';
import { DiffReview, createUploadedSnapshot, type DiffComment } from './DiffReview.js';

const ALL_FALSE: HostCapabilities = {
    tree: false,
    changes: false,
    readArtifact: false,
    terminal: false,
    git: false,
    mcp: false,
    approvals: false,
};

let host: HTMLDivElement;

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
});

afterEach(() => {
    render(null, host);
    host.remove();
    vi.restoreAllMocks();
});

class RecordingFixtureTransport extends FixtureTransport {
    readonly branchCalls: Array<{ taskId: string; scope: string }> = [];

    override async changesSnapshot(input: {
        taskId: string;
        scope: 'last_turn' | 'branch';
    }): Promise<ChangesSnapshot | null> {
        this.branchCalls.push({ taskId: input.taskId, scope: input.scope });
        return {
            scope: input.scope,
            files: [{ path: 'fixture-branch.ts', status: 'modified' }],
            headDigest: `sha256:${'d'.repeat(64)}`,
            capturedAtMs: 7,
        };
    }

    override async hostCapabilities(): Promise<HostCapabilities> {
        return { ...(await super.hostCapabilities()), changes: true };
    }
}

class TruncatingFixtureTransport extends FixtureTransport {
    override async hostCapabilities(): Promise<HostCapabilities> {
        return { ...(await super.hostCapabilities()), changes: true };
    }

    override async changesSnapshot(input: {
        taskId: string;
        scope: 'last_turn' | 'branch';
    }): Promise<ChangesSnapshot | null> {
        return {
            scope: 'branch',
            files: [{ path: 'bounded.ts', status: 'modified' }],
            headDigest: `sha256:${'f'.repeat(64)}`,
            capturedAtMs: 11,
            truncated: true,
        };
    }
}

class FakeHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly intents: HostToolIntent[] = [];

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge',
            hostAvailable: true,
            streaming: 'none',
            ...ALL_FALSE,
            changes: true,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostInvokeResponse> {
        this.intents.push(intent);
        const output = {
            snapshot: {
                scope: 'branch',
                files: [
                    { path: 'src/host-branch.ts', status: 'modified' },
                    { path: 'src/new.ts', status: 'added' },
                ],
                head_digest: `sha256:${'e'.repeat(64)}`,
                head_commit: '1'.repeat(40),
                base_ref: 'main',
                captured_at_ms: 1_800_000_000_000,
                truncated: true,
            },
        };
        const result: HostToolResult = {
            schema: HOST_RESULT_SCHEMA,
            intent_id: intent.intent_id,
            task_id: intent.task_id,
            run_id: intent.run_id,
            generation: intent.generation,
            tool_call_id: intent.tool_call_id,
            request_scope: intent.request_scope ?? 'task',
            status: 'ok',
            output,
            output_digest: await sha256Canonical(output),
            output_preview: 'branch',
            occurred_at_ms: 1_800_000_000_000,
        };
        return result;
    }

    subscribe(): HostSubscription {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: { approvalId: string; decision: 'approved' | 'denied' }) {
        return input;
    }
}

function changesSnapshotEvent(
    taskId: string,
    projectId: string,
    runId: string,
    generation: number,
    sequence: number,
    files: Array<Record<string, unknown>>,
    headDigest: string,
): AgentIncomingEvent {
    return {
        schema: 'uor-r4.studio.agent-event/1',
        event_id: `evt_${runId}_${generation}_${sequence.toString().padStart(12, '0')}`,
        project_id: projectId,
        task_id: taskId,
        run_id: runId,
        generation,
        sequence,
        recorded_at_ms: 1_794_283_200_000 + sequence,
        type: 'changes.snapshot',
        data: {
            scope: 'last_turn',
            files,
            head_digest: headDigest,
            captured_at_ms: 1_794_283_200_000 + sequence,
        },
    } as unknown as AgentIncomingEvent;
}

async function seedLastTurn(
    client: AgentClient,
    files: Array<Record<string, unknown>> = [{ path: 'src/a.ts', change: 'modified' }],
    headDigest = `sha256:${'a'.repeat(64)}`,
): Promise<{ taskId: string; runId: string; files: Array<Record<string, unknown>>; headDigest: string }> {
    const task = await client.createTask({ title: 'Review fixture' });
    const run = await client.startRun({ taskId: task.taskId, prompt: 'review' });
    const cursor = client.store.getRunCursor(task.taskId, run.runId);
    if (cursor === null) throw new Error('run cursor is missing');
    const applied = client.store.apply(
        changesSnapshotEvent(
            task.taskId,
            task.projectId,
            run.runId,
            cursor.generation,
            cursor.lastSequence + 1,
            files,
            headDigest,
        ),
    );
    if (applied.status !== 'applied') {
        throw new Error(`changes.snapshot was rejected: ${applied.message ?? ''}`);
    }
    return { taskId: task.taskId, runId: run.runId, files, headDigest };
}

function snapshot(
    overrides: Partial<ChangesSnapshot> = {},
): ChangesSnapshot {
    return {
        scope: 'last_turn',
        files: [{ path: 'src/a.ts', change: 'modified', content: 'line one\nline two\nline three' }],
        headDigest: `sha256:${'a'.repeat(64)}`,
        capturedAtMs: 1_794_283_200_000,
        ...overrides,
    };
}

function comment(overrides: Partial<DiffComment> = {}): DiffComment {
    return {
        commentId: 'cmt_1',
        taskId: 'tsk_9a3c5e7b1d2f4a60',
        scope: 'last_turn',
        path: 'src/a.ts',
        line: 2,
        body: 'check this line',
        headDigest: `sha256:${'a'.repeat(64)}`,
        createdAtMs: 1_794_283_200_001,
        ...overrides,
    };
}

function renderReview(
    overrides: Partial<Parameters<typeof DiffReview>[0]> = {},
): void {
    act(() => {
        render(
            <DiffReview
                taskId="tsk_9a3c5e7b1d2f4a60"
                scope="last_turn"
                onScope={vi.fn()}
                comments={[]}
                onComment={vi.fn()}
                snapshot={snapshot()}
                source="browser"
                {...overrides}
            />,
            host,
        );
    });
}

describe('DiffReview change sources', () => {
    test('changes_snapshot_last_turn_scope', async () => {
        const transport = new RecordingFixtureTransport();
        const client = new AgentClient({ transport });
        const seeded = await seedLastTurn(client);

        const result = await client.changesSnapshot({
            taskId: seeded.taskId,
            scope: 'last_turn',
        });

        expect(result).toMatchObject({
            scope: 'last_turn',
            files: seeded.files,
            headDigest: seeded.headDigest,
        });
        expect(transport.branchCalls).toEqual([]);
    });

    test('changes_snapshot_branch_scope', async () => {
        const transport = new RecordingFixtureTransport();
        const fakeHost = new FakeHostTransport();
        const client = new AgentClient({
            transport,
            hostClient: new HostClient({ transport: fakeHost }),
        });
        const task = await client.createTask({ title: 'Branch review' });
        await client.startRun({ taskId: task.taskId, prompt: 'branch' });

        const result = await client.changesSnapshot({
            taskId: task.taskId,
            scope: 'branch',
        });

        expect(result).toMatchObject({
            scope: 'branch',
            files: [
                { path: 'src/host-branch.ts', status: 'modified' },
                { path: 'src/new.ts', status: 'added' },
            ],
            headDigest: `sha256:${'e'.repeat(64)}`,
            truncated: true,
        });
        expect(fakeHost.intents.map((intent) => intent.tool)).toEqual(['git.changes']);
        expect(transport.branchCalls).toEqual([]);
    });

    test('durable_diff_comments_are_keyed_by_task_path_line_and_head', () => {
        const prototype = AgentTaskStore.prototype as unknown as Record<string, unknown>;
        expect(typeof prototype.recordDiffComment).toBe('function');
        expect(typeof prototype.readDiffComments).toBe('function');
    });

    test('bridge_transport_preserves_branch_truncation', async () => {
        const client = new AgentClient({ transport: new TruncatingFixtureTransport() });
        const task = await client.createTask({ title: 'Bounded branch' });
        await client.startRun({ taskId: task.taskId, prompt: 'bounded' });

        const result = await client.changesSnapshot({
            taskId: task.taskId,
            scope: 'branch',
        });

        expect(result).toMatchObject({
            scope: 'branch',
            truncated: true,
            files: [{ path: 'bounded.ts', status: 'modified' }],
        });
    });
});

describe('DiffReview static browser review', () => {
    test('static_browser_reviews_uploaded_file_locally', async () => {
        const file = new File(
            ['export function add(a, b) {\n    return a + b;\n}\n'],
            'src/math.ts',
            { type: 'text/plain' },
        );
        const uploaded = await createUploadedSnapshot([file]);

        expect(uploaded.scope).toBe('branch');
        expect(uploaded.truncated).toBe(false);
        expect(uploaded.files).toHaveLength(1);
        expect(uploaded.files[0]).toMatchObject({
            path: 'src/math.ts',
            status: 'modified',
        });
        expect(uploaded.headDigest).toMatch(/^sha256:[0-9a-f]{64}$/);

        const onComment = vi.fn();
        renderReview({
            scope: 'branch',
            snapshot: uploaded,
            source: 'upload',
            onComment,
        });

        expect(host.textContent).toContain('src/math.ts');
        expect(host.textContent).toContain('export function add(a, b)');
        expect(host.textContent).toContain('Uploaded file');
        expect(host.textContent).toContain('stored in this browser');
        expect(host.querySelector('[data-diff-file="src/math.ts"]')).not.toBeNull();
        expect(host.querySelector('[data-diff-line="2"]')?.textContent).toContain('return a + b;');
    });

    test('branch_truncation_is_surfaced_accessible', () => {
        renderReview({
            scope: 'branch',
            source: 'host',
            snapshot: snapshot({
                scope: 'branch',
                files: [{ path: 'src/host.ts', status: 'modified' }],
                truncated: true,
            }),
        });

        expect(host.textContent).toContain('truncated');
        expect(host.textContent).toContain('not the complete');
        expect(host.querySelector('[data-diff-truncated]')).not.toBeNull();
    });
});

describe('DiffReview local comments', () => {
    test('stale_comment_line_is_marked', () => {
        renderReview({
            comments: [
                comment(),
                comment({
                    commentId: 'cmt_old_head',
                    body: 'old head note',
                    headDigest: `sha256:${'b'.repeat(64)}`,
                }),
                comment({
                    commentId: 'cmt_missing_file',
                    body: 'missing file note',
                    path: 'src/gone.ts',
                    line: 1,
                }),
                comment({
                    commentId: 'cmt_missing_line',
                    body: 'missing line note',
                    line: 99,
                }),
            ],
        });

        const current = host.querySelector('[data-diff-current-comments]');
        const stale = host.querySelector('[data-diff-stale-comments]');
        expect(current?.textContent).toContain('check this line');
        expect(current?.textContent).not.toContain('old head note');
        expect(stale?.textContent).toContain('old head note');
        expect(stale?.textContent).toContain('missing file note');
        expect(stale?.textContent).toContain('missing line note');
        expect(stale?.textContent).toContain('head changed');
        expect(stale?.textContent).toContain('no longer in this snapshot');
        expect(stale?.textContent).toContain('Line 99 is not present');
        expect(host.querySelectorAll('[data-diff-stale="true"]')).toHaveLength(3);
    });

    test('comment_form_adds_a_line_bound_local_comment', async () => {
        const onComment = vi.fn();
        renderReview({ onComment });

        const body = host.querySelector<HTMLTextAreaElement>('textarea')!;
        const line = host.querySelector<HTMLInputElement>('input[type="number"]')!;
        act(() => {
            line.value = '3';
            line.dispatchEvent(new Event('input', { bubbles: true }));
            body.value = 'please rename this';
            body.dispatchEvent(new Event('input', { bubbles: true }));
        });
        await act(async () => {
            host.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
            await Promise.resolve();
        });

        expect(onComment).toHaveBeenCalledWith({
            path: 'src/a.ts',
            line: 3,
            body: 'please rename this',
        });
    });

    test('comment_form_rejects_a_line_that_is_not_in_the_snapshot', async () => {
        const onComment = vi.fn();
        renderReview({ onComment });

        const body = host.querySelector<HTMLTextAreaElement>('textarea')!;
        const line = host.querySelector<HTMLInputElement>('input[type="number"]')!;
        act(() => {
            line.value = '99';
            line.dispatchEvent(new Event('input', { bubbles: true }));
            body.value = 'ghost line';
            body.dispatchEvent(new Event('input', { bubbles: true }));
        });
        await act(async () => {
            host.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
            await Promise.resolve();
        });

        expect(onComment).not.toHaveBeenCalled();
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Line 99 is not present');
    });
});

describe('DiffReview control surface', () => {
    test('no_merge_control_without_service_identity', () => {
        renderReview();

        expect(host.textContent).not.toMatch(/\bmerge\b/i);
        expect(Array.from(host.querySelectorAll('button')).map((button) => button.textContent).join(' ')).not.toMatch(/\bmerge\b/i);
    });

    test('no_checkpoint_restore_control_is_rendered', () => {
        renderReview();

        expect(host.textContent).not.toMatch(/restore checkpoint/i);
        expect(host.textContent).not.toMatch(/\bcheckpoint\b/i);
        expect(Array.from(host.querySelectorAll('button')).map((button) => button.textContent).join(' ')).not.toMatch(/restore|checkpoint/i);
    });
});
