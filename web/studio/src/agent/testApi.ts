import { AgentClient } from './client.js';
import { AgentTaskStore } from './store.js';
import { AgentStoreError } from './storeTypes.js';
import { AGENT_EVENT_SCHEMA } from './generated/agentContract.js';
import {
    createRunQueuedEvent,
    createRunStartedEvent,
    fixtureEventId,
} from './fixtures.js';
import {
    createAgentStore,
    type AgentIncomingEvent,
    type AgentStore,
    type RunView,
    type TaskView,
} from '../state/agent.js';

const TERMINAL_STATES = new Set(['finished', 'failed', 'cancelled', 'expired']);

interface StoreFaults {
    abort_next_append: boolean;
}

interface FixtureSummary {
    taskId: string;
    projectId: string;
    runId: string;
    generation: number;
    eventIds: string[];
    digests: string[];
}

interface CursorView {
    taskId: string;
    runId: string;
    generation: number;
    sequence: number;
    eventId: string;
    digest: string;
    runState: string;
    statusProjection: string | null;
    taskState: string;
    messageCount: number;
    lastMessageText: string | null;
}

interface ActiveRunView {
    runId: string;
    generation: number;
    lastSequence: number;
    lastEventId: string | null;
    state: string;
    statusProjection: string | null;
}

interface RestoreView {
    phase: 'idle' | 'loading' | 'ready' | 'incomplete' | 'error';
    sendEnabled: boolean;
    canStartRun: boolean;
    error: { code: string; message: string } | null;
    incomplete: Array<{
        taskId: string | null;
        runId: string | null;
        code: string;
        message: string;
    }>;
    activeRun: ActiveRunView | null;
}

interface TaskViewSummary {
    taskId: string;
    title: string;
    pinned: boolean;
    archived: boolean;
    state: string;
    latestRunId: string | null;
    runs: Array<{ runId: string; lastSequence: number; state: string }>;
}

interface Outcome {
    ok: boolean;
    code?: string;
    message?: string;
    status?: string;
    storage?: string | null;
    record?: unknown;
    records?: unknown;
}

export interface StudioTestApi {
    reset(): Promise<void>;
    open(): Promise<RestoreView>;
    beginOpen(): Promise<void>;
    whenOpen(): Promise<RestoreView>;
    restoreState(): RestoreView;
    createTaskAndEvents(options?: {
        mode?: 'message' | 'cancelling' | 'finished';
    }): Promise<FixtureSummary>;
    createSecondRun(): Promise<FixtureSummary>;
    cursor(): CursorView | null;
    taskView(): TaskViewSummary | null;
    writeSnapshot(): Promise<Outcome>;
    readSnapshot(): Promise<Outcome>;
    appendChanges(): Promise<Outcome>;
    appendResume(): Promise<Outcome>;
    recordChanges(headDigest?: string): Promise<Outcome>;
    readChanges(): Promise<Outcome>;
    pin(pinned: boolean): Promise<Outcome>;
    archive(archived: boolean): Promise<Outcome>;
    acquireLease(writerId: string, nowMs?: number): Promise<Outcome>;
    heartbeatLease(writerId: string, nowMs?: number): Promise<Outcome>;
    releaseLease(writerId: string): Promise<Outcome>;
    lease(): Promise<Outcome>;
    armAbortNextAppend(): void;
    appendAssistant(
        text: string,
        options?: { writerId?: string; nowMs?: number },
    ): Promise<Outcome>;
    exportNdjson(): Promise<string>;
    importNdjson(text: string): Promise<Outcome>;
}

interface TestState {
    client: AgentClient | null;
    taskStore: AgentTaskStore | null;
    store: AgentStore | null;
    nowMs: number;
    phase: RestoreView['phase'];
    error: { code: string; message: string } | null;
    incomplete: RestoreView['incomplete'];
    fixture: FixtureSummary | null;
    openPromise: Promise<void> | null;
    openGate: (() => void) | null;
    faults: StoreFaults;
}

let state: TestState = freshState();

function freshState(): TestState {
    return {
        client: null,
        taskStore: null,
        store: null,
        nowMs: 1_794_283_200_000,
        phase: 'idle',
        error: null,
        incomplete: [],
        fixture: null,
        openPromise: null,
        openGate: null,
        faults: { abort_next_append: false },
    };
}

function randomHex(bytes: number): string {
    const values = new Uint8Array(bytes);
    crypto.getRandomValues(values);
    return [...values]
        .map((value) => value.toString(16).padStart(2, '0'))
        .join('');
}

function ensureClient(): AgentClient {
    if (state.client !== null) return state.client;
    const store = createAgentStore();
    const taskStore = new AgentTaskStore({
        store,
        now: () => state.nowMs,
        faults: state.faults,
    });
    const client = new AgentClient({ store, taskStore });
    state.store = store;
    state.taskStore = taskStore;
    state.client = client;
    return client;
}

function requireClient(): AgentClient {
    if (state.client === null) throw new Error('studio test client is not initialized');
    return state.client;
}

function requireTaskStore(): AgentTaskStore {
    if (state.taskStore === null) throw new Error('studio test store is not initialized');
    return state.taskStore;
}

function currentTask(): TaskView | null {
    const client = state.client;
    if (client === null) return null;
    const snapshot = client.store.snapshot();
    return (
        snapshot.selectedTask ??
        snapshot.tasks.find((task) => task.taskId === state.fixture?.taskId) ??
        snapshot.tasks[0] ??
        null
    );
}

function currentRun(task: TaskView | null): RunView | null {
    if (task === null) return null;
    if (task.latestRunId !== null) {
        return task.runs.find((run) => run.runId === task.latestRunId) ?? null;
    }
    return task.runs.length === 0 ? null : task.runs[task.runs.length - 1]!;
}

function currentFixture(): FixtureSummary {
    if (state.fixture !== null) return state.fixture;
    const task = currentTask();
    const run = currentRun(task);
    if (task === null || run === null) {
        throw new Error('no task/run fixture is present');
    }
    const fixture: FixtureSummary = {
        taskId: task.taskId,
        projectId: task.projectId,
        runId: run.runId,
        generation: run.generation,
        eventIds: run.events.map((event) => event.eventId),
        digests: run.lastDigest === null ? [] : [run.lastDigest],
    };
    state.fixture = fixture;
    return fixture;
}

function errorCode(error: unknown): string {
    return error instanceof AgentStoreError ? error.code : 'internal';
}

function errorMessage(error: unknown): string {
    if (error instanceof AgentStoreError) {
        const prefix = `${error.code}: `;
        return error.message.startsWith(prefix)
            ? error.message.slice(prefix.length)
            : error.message;
    }
    return error instanceof Error ? error.message : String(error);
}

function failure(error: unknown): Outcome {
    return { ok: false, code: errorCode(error), message: errorMessage(error) };
}

function appendOutcome(result: {
    status: string;
    code: string | null;
    message: string | null;
    storage: string | null;
}): Outcome {
    return {
        ok: result.status !== 'rejected',
        status: result.status,
        ...(result.code === null ? {} : { code: result.code }),
        ...(result.message === null ? {} : { message: result.message }),
        storage: result.storage,
    };
}

function restoreState(): RestoreView {
    const task = currentTask();
    const run = currentRun(task);
    const activeRun =
        run !== null && !TERMINAL_STATES.has(run.state)
            ? {
                  runId: run.runId,
                  generation: run.generation,
                  lastSequence: run.lastSequence,
                  lastEventId: run.lastEventId,
                  state: run.state,
                  statusProjection: run.statusProjection,
              }
            : null;
    const incompleteForTask =
        task !== null && state.incomplete.some((entry) => entry.taskId === task.taskId);
    const canStartRun =
        state.phase === 'ready' && !incompleteForTask && activeRun === null;
    return {
        phase: state.phase,
        sendEnabled: canStartRun,
        canStartRun,
        error: state.error,
        incomplete: state.incomplete,
        activeRun,
    };
}

async function performOpen(): Promise<void> {
    const client = ensureClient();
    state.phase = 'loading';
    state.error = null;
    state.incomplete = [];
    try {
        const report = await requireTaskStore().open();
        state.incomplete = report.incomplete.map((entry) => ({
            taskId: entry.task_id,
            runId: entry.run_id,
            code: entry.code,
            message: entry.message,
        }));
        state.phase = state.incomplete.length > 0 ? 'incomplete' : 'ready';
    } catch (error) {
        state.phase = 'error';
        state.error = { code: errorCode(error), message: errorMessage(error) };
    }
    void client;
}

async function open(): Promise<RestoreView> {
    if (state.phase === 'ready' || state.phase === 'incomplete') {
        return restoreState();
    }
    if (state.openGate !== null) {
        const release = state.openGate;
        state.openGate = null;
        release();
    }
    ensureClient();
    await performOpen();
    return restoreState();
}

function eventFor(
    fixture: FixtureSummary,
    sequence: number,
    generation: number,
    type: string,
    data: Record<string, unknown>,
): AgentIncomingEvent {
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id: fixtureEventId(fixture.runId, generation, sequence),
        project_id: fixture.projectId,
        task_id: fixture.taskId,
        run_id: fixture.runId,
        generation,
        sequence,
        recorded_at_ms: state.nowMs + sequence,
        type,
        data,
    } as unknown as AgentIncomingEvent;
}

async function appendFixtureEvent(
    fixture: FixtureSummary,
    type: string,
    data: Record<string, unknown>,
    options: { writerId?: string; nowMs?: number } = {},
): Promise<Outcome & { digest?: string | null }> {
    const task = requireClient().store.snapshot().tasks.find(
        (candidate) => candidate.taskId === fixture.taskId,
    );
    const run = task?.runs.find((candidate) => candidate.runId === fixture.runId) ?? null;
    if (run === null) return { ok: false, code: 'run_not_found', message: 'run is missing' };
    const event = eventFor(
        fixture,
        run.lastSequence + 1,
        run.generation,
        type,
        data,
    );
    const result = await requireTaskStore().appendEvent({
        event,
        writer_id: options.writerId,
        now_ms: options.nowMs,
    });
    if (result.status === 'applied') {
        fixture.eventIds.push(event.event_id);
        if (result.digest !== null) fixture.digests.push(result.digest);
    }
    return appendOutcome(result);
}

async function createTaskAndEvents(options: {
    mode?: 'message' | 'cancelling' | 'finished';
} = {}): Promise<FixtureSummary> {
    if (state.phase !== 'ready') {
        const opened = await open();
        if (opened.phase !== 'ready') {
            throw new Error(`studio test store is ${opened.phase}: ${opened.error?.message ?? ''}`);
        }
    }
    const taskStore = requireTaskStore();
    const task = await taskStore.createTask({ title: 'Fixture task' });
    const runId = `run_${randomHex(8)}`;
    const fixture: FixtureSummary = {
        taskId: task.task_id,
        projectId: task.project_id,
        runId,
        generation: 1,
        eventIds: [],
        digests: [],
    };
    const queued = createRunQueuedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: 1,
        recordedAtMs: state.nowMs,
        queuedAtMs: state.nowMs,
    });
    const queuedResult = await taskStore.appendEvent({ event: queued });
    if (queuedResult.status !== 'applied') {
        throw new Error(`queued fixture was rejected: ${queuedResult.message ?? ''}`);
    }
    fixture.eventIds.push(queued.event_id);
    fixture.digests.push(queuedResult.digest!);

    const started = createRunStartedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: 1,
        recordedAtMs: state.nowMs + 1,
        startedAtMs: state.nowMs + 1,
        mode: 'agent',
        modelId: 'replay',
        modelKind: 'fixture',
        environmentKind: 'browser-fixture',
    });
    const startedResult = await taskStore.appendEvent({ event: started });
    if (startedResult.status !== 'applied') {
        throw new Error(`started fixture was rejected: ${startedResult.message ?? ''}`);
    }
    fixture.eventIds.push(started.event_id);
    fixture.digests.push(startedResult.digest!);

    const mode = options.mode ?? 'message';
    if (mode === 'message') {
        const message = eventFor(fixture, 2, 1, 'assistant.message', {
            message_id: `msg_${randomHex(8)}`,
            text: 'replayed fixture message',
        });
        const messageResult = await taskStore.appendEvent({ event: message });
        if (messageResult.status !== 'applied') {
            throw new Error(`message fixture was rejected: ${messageResult.message ?? ''}`);
        }
        fixture.eventIds.push(message.event_id);
        fixture.digests.push(messageResult.digest!);
    } else if (mode === 'cancelling') {
        const status = eventFor(fixture, 2, 1, 'run.status', {
            status: 'cancelling',
        });
        const statusResult = await taskStore.appendEvent({ event: status });
        if (statusResult.status !== 'applied') {
            throw new Error(`cancelling fixture was rejected: ${statusResult.message ?? ''}`);
        }
        fixture.eventIds.push(status.event_id);
        fixture.digests.push(statusResult.digest!);
    } else {
        const finished = eventFor(fixture, 2, 1, 'run.finished', {
            status: 'finished',
            finished_at_ms: state.nowMs + 2,
            duration_ms: 2,
        });
        const finishedResult = await taskStore.appendEvent({ event: finished });
        if (finishedResult.status !== 'applied') {
            throw new Error(`finished fixture was rejected: ${finishedResult.message ?? ''}`);
        }
        fixture.eventIds.push(finished.event_id);
        fixture.digests.push(finishedResult.digest!);
    }

    state.fixture = fixture;
    requireClient().store.selectTask(fixture.taskId);
    return fixture;
}

async function createSecondRun(): Promise<FixtureSummary> {
    const task = currentTask();
    const first = currentRun(task);
    if (task === null || first === null) {
        throw new Error('a first run is required before the second run');
    }
    if (!TERMINAL_STATES.has(first.state)) {
        throw new Error('the first run must be terminal before a new run');
    }
    const runId = `run_${randomHex(8)}`;
    const fixture: FixtureSummary = {
        taskId: task.taskId,
        projectId: task.projectId,
        runId,
        generation: first.generation + 1,
        eventIds: [],
        digests: [],
    };
    const queued = createRunQueuedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: fixture.generation,
        recordedAtMs: state.nowMs + 10,
        queuedAtMs: state.nowMs + 10,
    });
    const queuedResult = await requireTaskStore().appendEvent({ event: queued });
    if (queuedResult.status !== 'applied') {
        throw new Error(`second queued fixture was rejected: ${queuedResult.message ?? ''}`);
    }
    fixture.eventIds.push(queued.event_id);
    fixture.digests.push(queuedResult.digest!);
    const started = createRunStartedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: fixture.generation,
        recordedAtMs: state.nowMs + 11,
        startedAtMs: state.nowMs + 11,
        mode: 'agent',
        modelId: 'replay',
        modelKind: 'fixture',
        environmentKind: 'browser-fixture',
    });
    const startedResult = await requireTaskStore().appendEvent({ event: started });
    if (startedResult.status !== 'applied') {
        throw new Error(`second started fixture was rejected: ${startedResult.message ?? ''}`);
    }
    fixture.eventIds.push(started.event_id);
    fixture.digests.push(startedResult.digest!);
    state.fixture = fixture;
    requireClient().store.selectTask(fixture.taskId);
    return fixture;
}

function cursorView(): CursorView | null {
    if (state.phase === 'loading' || state.phase === 'error' || state.phase === 'idle') {
        return null;
    }
    const task = currentTask();
    const run = currentRun(task);
    if (task === null || run === null || run.lastEventId === null || run.lastDigest === null) {
        return null;
    }
    return {
        taskId: task.taskId,
        runId: run.runId,
        generation: run.generation,
        sequence: run.lastSequence,
        eventId: run.lastEventId,
        digest: run.lastDigest,
        runState: run.state,
        statusProjection: run.statusProjection,
        taskState: task.state,
        messageCount: run.messages.length,
        lastMessageText: run.messages.length === 0
            ? null
            : run.messages[run.messages.length - 1]!.text,
    };
}

function taskView(): TaskViewSummary | null {
    const task = currentTask();
    if (task === null) return null;
    return {
        taskId: task.taskId,
        title: task.title,
        pinned: task.pinnedLocal,
        archived: task.archived,
        state: task.state,
        latestRunId: task.latestRunId,
        runs: task.runs.map((run) => ({
            runId: run.runId,
            lastSequence: run.lastSequence,
            state: run.state,
        })),
    };
}

async function updateFlag(kind: 'pin' | 'archive', value: boolean): Promise<Outcome> {
    try {
        const fixture = currentFixture();
        const record =
            kind === 'pin'
                ? await requireTaskStore().pinTask(fixture.taskId, value)
                : await requireTaskStore().archiveTask(fixture.taskId, value);
        return { ok: true, record };
    } catch (error) {
        return failure(error);
    }
}

function importFixtureFromStore(): void {
    const task = currentTask();
    const run = currentRun(task);
    if (task === null || run === null) {
        state.fixture = null;
        return;
    }
    state.fixture = {
        taskId: task.taskId,
        projectId: task.projectId,
        runId: run.runId,
        generation: run.generation,
        eventIds: run.events.map((event) => event.eventId),
        digests: [],
    };
}

export function installStudioTestApi(): void {
    if (window.__studioTest !== undefined) return;
    window.__studioTest = {
        async reset() {
            state.client?.dispose();
            if (state.taskStore !== null) {
                await state.taskStore.deleteDatabase();
            } else {
                const temporary = new AgentTaskStore({
                    store: createAgentStore(),
                    now: () => Date.now(),
                });
                await temporary.deleteDatabase();
            }
            state = freshState();
        },
        open,
        async beginOpen() {
            ensureClient();
            state.phase = 'loading';
            state.error = null;
            state.incomplete = [];
            const gate = new Promise<void>((resolve) => {
                state.openGate = resolve;
            });
            state.openPromise = (async () => {
                await gate;
                await performOpen();
                state.openPromise = null;
            })();
        },
        async whenOpen() {
            if (state.openGate !== null) {
                const release = state.openGate;
                state.openGate = null;
                release();
            }
            if (state.openPromise !== null) {
                await state.openPromise;
            }
            return restoreState();
        },
        restoreState,
        createTaskAndEvents,
        createSecondRun,
        cursor: cursorView,
        taskView,
        async writeSnapshot() {
            try {
                const fixture = currentFixture();
                const record = await requireTaskStore().writeSnapshot(
                    fixture.taskId,
                    fixture.runId,
                );
                return { ok: true, record };
            } catch (error) {
                return failure(error);
            }
        },
        async readSnapshot() {
            try {
                const fixture = currentFixture();
                const result = await requireTaskStore().readSnapshot(
                    fixture.taskId,
                    fixture.runId,
                );
                return result.ok
                    ? { ok: true, record: result.record }
                    : { ok: false, code: result.code, message: result.message };
            } catch (error) {
                return failure(error);
            }
        },
        async appendChanges() {
            const fixture = currentFixture();
            return appendFixtureEvent(fixture, 'changes.snapshot', {
                scope: 'last_turn',
                files: [{ path: 'src/a.ts', status: 'modified' }],
                head_digest: 'turn:3:fixture',
                captured_at_ms: state.nowMs + 3,
            });
        },
        async appendResume() {
            const fixture = currentFixture();
            const run = currentRun(currentTask());
            if (run === null || run.lastDigest === null) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'no verified run cursor is available to resume',
                };
            }
            const event = eventFor(
                fixture,
                run.lastSequence + 1,
                run.generation + 1,
                'run.resumed',
                {
                    resumed_at_ms: state.nowMs + run.lastSequence + 1,
                    previous_generation: run.generation,
                    snapshot_digest: run.lastDigest,
                },
            );
            const result = await requireTaskStore().appendEvent({ event });
            if (result.status === 'applied') {
                fixture.eventIds.push(event.event_id);
                if (result.digest !== null) fixture.digests.push(result.digest);
            }
            return appendOutcome(result);
        },
        async recordChanges(headDigest?: string) {
            try {
                const fixture = currentFixture();
                const view = requireTaskStore().store.getRun(
                    fixture.taskId,
                    fixture.runId,
                );
                const latest = view?.changesSnapshots[view.changesSnapshots.length - 1];
                const head = headDigest ?? latest?.headDigest;
                if (head === undefined) {
                    return {
                        ok: false,
                        code: 'invalid_arguments',
                        message: 'no changes.snapshot is present',
                    };
                }
                const record = await requireTaskStore().recordChanges({
                    task_id: fixture.taskId,
                    run_id: fixture.runId,
                    head_digest: head,
                    files: latest?.files ?? [],
                    captured_at_ms: latest?.capturedAtMs ?? state.nowMs,
                });
                return { ok: true, record };
            } catch (error) {
                return failure(error);
            }
        },
        async readChanges() {
            try {
                const fixture = currentFixture();
                const records = await requireTaskStore().readChanges(
                    fixture.taskId,
                    fixture.runId,
                );
                return { ok: true, record: { records } };
            } catch (error) {
                return failure(error);
            }
        },
        pin: (pinned: boolean) => updateFlag('pin', pinned),
        archive: (archived: boolean) => updateFlag('archive', archived),
        async acquireLease(writerId: string, nowMs?: number) {
            try {
                const fixture = currentFixture();
                return {
                    ok: true,
                    record: await requireTaskStore().acquireLease(
                        fixture.taskId,
                        writerId,
                        nowMs,
                    ),
                };
            } catch (error) {
                return failure(error);
            }
        },
        async heartbeatLease(writerId: string, nowMs?: number) {
            try {
                const fixture = currentFixture();
                return {
                    ok: true,
                    record: await requireTaskStore().heartbeatLease(
                        fixture.taskId,
                        writerId,
                        nowMs,
                    ),
                };
            } catch (error) {
                return failure(error);
            }
        },
        async releaseLease(writerId: string) {
            try {
                const fixture = currentFixture();
                const result = await requireTaskStore().releaseLease(
                    fixture.taskId,
                    writerId,
                );
                return { ok: true, record: result };
            } catch (error) {
                return failure(error);
            }
        },
        async lease() {
            try {
                const fixture = currentFixture();
                return {
                    ok: true,
                    record: await requireTaskStore().readLease(fixture.taskId),
                };
            } catch (error) {
                return failure(error);
            }
        },
        armAbortNextAppend() {
            state.faults.abort_next_append = true;
        },
        async appendAssistant(
            text: string,
            options: { writerId?: string; nowMs?: number } = {},
        ) {
            const fixture = currentFixture();
            return appendFixtureEvent(
                fixture,
                'assistant.message',
                {
                    message_id: `msg_${randomHex(8)}`,
                    text,
                },
                options,
            );
        },
        async exportNdjson() {
            return requireTaskStore().exportNdjson();
        },
        async importNdjson(text: string) {
            try {
                const summary = await requireTaskStore().importNdjson(text);
                importFixtureFromStore();
                return { ok: true, record: summary };
            } catch (error) {
                return failure(error);
            }
        },
    };
}

declare global {
    interface Window {
        __studioTest?: StudioTestApi;
    }
}
