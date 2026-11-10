import { AgentClient, AgentClientError, type TaskSummary } from './client.js';
import { FixtureTransport } from './fixtureTransport.js';
import { AgentTaskStore } from './store.js';
import { AgentRuntime } from './runtime.js';
import { ReplayModelPort } from './modelPort.js';
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
    abort_next_task_write: boolean;
    fail_import_publish: boolean;
    fail_import_cleanup: boolean;
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
    unresolvedRunId: string | null;
    hold: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
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
    reopen(): Promise<RestoreView>;
    memoryCursor(): CursorView | null;
    beginOpen(): Promise<void>;
    whenOpen(): Promise<RestoreView>;
    restoreState(): RestoreView;
    createTaskAndEvents(options?: {
        mode?: 'message' | 'cancelling' | 'finished';
    }): Promise<FixtureSummary>;
    createSecondRun(): Promise<FixtureSummary>;
    createClientRun(): Promise<FixtureSummary>;
    createQueuedOnlyTask(): Promise<FixtureSummary>;
    targetFixture(taskId: string, runId: string, generation?: number): void;
    listUnloadedActiveTask(mode: 'running' | 'cancelling'): Promise<Outcome>;
    listActiveSummary(
        taskId: string,
        latestRunId: string | null,
        mode: 'running' | 'cancelling',
    ): Promise<Outcome>;
    appendQueuedSafe(taskId: string, runId: string): Promise<Outcome>;
    createTaskAt(createdAtMs: number): Promise<Outcome>;
    resumeViaClient(): Promise<Outcome>;
    recordChangesWithFiles(files: Array<Record<string, unknown>>): Promise<Outcome>;
    seedSmallEvents(total: number): Promise<{
        events: number;
        sequence: number;
        durationMs: number;
        heapUsed: number | null;
    }>;
    scaleMetrics(): {
        events: number;
        sequence: number;
        durationMs: number;
        heapUsed: number | null;
    } | null;
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
    seedBackgroundHoldForTest(): Promise<Outcome>;
    startRunOnHeldTaskForTest(taskId: string): Promise<Outcome>;
    resolveBackgroundHoldForTest(
        taskId: string,
        runId: string,
        toolCallId: string,
    ): Promise<Outcome>;
    acquireLease(writerId: string, nowMs?: number): Promise<Outcome>;
    heartbeatLease(writerId: string, nowMs?: number): Promise<Outcome>;
    releaseLease(writerId: string): Promise<Outcome>;
    lease(): Promise<Outcome>;
    armAbortNextAppend(): void;
    armAbortNextTaskWrite(): void;
    armFailImportPublish(): void;
    armFailImportCleanup(): void;
    disposeDuringImport(): void;
    beginHeldQueuedAppend(taskId: string, runId: string): Promise<void>;
    awaitAppendHeld(): Promise<void>;
    releaseHeldAppend(): void;
    finishHeldAppend(): Promise<Outcome>;
    beginHeldTaskWrite(taskId: string): Promise<void>;
    awaitTaskWriteHeld(): Promise<void>;
    releaseTaskWrite(): void;
    finishHeldTaskWrite(): Promise<Outcome>;
    beginImport(text: string, options?: { holdPublish?: boolean }): void;
    awaitImportPublishGate(): Promise<void>;
    releaseImportPublishGate(): void;
    whenImport(): Promise<Outcome>;
    advanceImportClock(ms: number): Promise<Outcome>;
    freezeImportClock(ms: number): Promise<Outcome>;
    appendAssistantSafe(
        text: string,
        options?: { writerId?: string; nowMs?: number },
    ): Promise<Outcome>;
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
    scale: {
        events: number;
        sequence: number;
        durationMs: number;
        heapUsed: number | null;
    } | null;
    importPromise: Promise<Outcome> | null;
    importGateRelease: (() => void) | null;
    appendGateRelease: (() => void) | null;
    appendGateReached: Promise<void> | null;
    heldAppend: Promise<Outcome> | null;
    taskWriteGateRelease: (() => void) | null;
    taskWriteGateReached: Promise<void> | null;
    heldTaskWrite: Promise<Outcome> | null;
    importPublishGateRelease: (() => void) | null;
    importPublishGateReached: Promise<void> | null;
    timers: TestTimer[];
    nextTimerHandle: number;
}

interface TestTimer {
    handle: number;
    periodMs: number;
    nextAtMs: number;
    callback: () => void | Promise<void>;
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
        faults: {
            abort_next_append: false,
            abort_next_task_write: false,
            fail_import_publish: false,
            fail_import_cleanup: false,
        },
        scale: null,
        importPromise: null,
        importGateRelease: null,
        appendGateRelease: null,
        appendGateReached: null,
        heldAppend: null,
        taskWriteGateRelease: null,
        taskWriteGateReached: null,
        heldTaskWrite: null,
        importPublishGateRelease: null,
        importPublishGateReached: null,
        timers: [],
        nextTimerHandle: 1,
    };
}

function randomHex(bytes: number): string {
    const values = new Uint8Array(bytes);
    crypto.getRandomValues(values);
    return [...values]
        .map((value) => value.toString(16).padStart(2, '0'))
        .join('');
}

function testSetInterval(
    handler: () => void | Promise<void>,
    timeout: number,
): number {
    const periodMs = Math.max(1, timeout);
    const handle = state.nextTimerHandle;
    state.nextTimerHandle += 1;
    state.timers.push({
        handle,
        periodMs,
        nextAtMs: state.nowMs + periodMs,
        callback: handler,
    });
    return handle;
}

function testClearInterval(handle: unknown): void {
    const index = state.timers.findIndex((timer) => timer.handle === handle);
    if (index >= 0) state.timers.splice(index, 1);
}

class SummaryTransport extends FixtureTransport {
    constructor(private readonly taskSummary: TaskSummary) {
        super();
    }

    override async listTasks(): Promise<TaskSummary[]> {
        return [this.taskSummary];
    }
}

function ensureClient(): AgentClient {
    if (state.client !== null) return state.client;
    const store = createAgentStore();
    const taskStore = new AgentTaskStore({
        store,
        now: () => state.nowMs,
        faults: state.faults,
        timers: {
            setInterval: testSetInterval,
            clearInterval: testClearInterval,
        },
    });
    const client = new AgentClient({
        transport: new FixtureTransport(),
        store,
        taskStore,
    });
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
    if (error instanceof AgentStoreError || error instanceof AgentClientError) {
        return error.code;
    }
    return 'internal';
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
    return {
        ok: false,
        code: errorCode(error),
        message: errorMessage(error),
        ...(error instanceof AgentStoreError && error.storage !== null
            ? { storage: error.storage }
            : {}),
    };
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
    const latestLoaded =
        task !== null &&
        task.latestRunId !== null &&
        run !== null &&
        run.runId === task.latestRunId;
    const unresolvedRun =
        task !== null &&
        (task.unresolvedRunId !== null ||
            (task.latestRunId !== null && !latestLoaded));
    const canStartRun =
        state.phase === 'ready' &&
        task?.state !== 'waiting_for_background' &&
        !incompleteForTask &&
        activeRun === null &&
        !unresolvedRun;
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
    if (
        (state.phase === 'ready' || state.phase === 'incomplete') &&
        state.taskStore?.isOpen === true
    ) {
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

async function ensureReady(): Promise<void> {
    if (state.phase === 'ready') return;
    const opened = await open();
    if (opened.phase !== 'ready') {
        throw new Error(`studio test store is ${opened.phase}: ${opened.error?.message ?? ''}`);
    }
}

async function createQueuedOnlyTask(): Promise<FixtureSummary> {
    await ensureReady();
    const taskStore = requireTaskStore();
    const task = await taskStore.createTask({ title: 'Queued only task' });
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
    const result = await taskStore.appendEvent({ event: queued });
    if (result.status !== 'applied') {
        throw new Error(`queued-only fixture was rejected: ${result.message ?? ''}`);
    }
    fixture.eventIds.push(queued.event_id);
    fixture.digests.push(result.digest!);
    state.fixture = fixture;
    requireClient().store.selectTask(fixture.taskId);
    return fixture;
}

async function listUnloadedActiveTask(
    mode: 'running' | 'cancelling',
): Promise<Outcome> {
    await ensureReady();
    const taskId = `tsk_${randomHex(8)}`;
    const runId = `run_${randomHex(8)}`;
    const summary: TaskSummary = {
        taskId,
        projectId: 'prj_0000000000000001',
        title: `Unloaded ${mode} task`,
        repo: null,
        branch: null,
        createdAtMs: state.nowMs,
        archived: false,
        pinned: false,
        latestRunId: runId,
        state: mode,
    };
    const client = new AgentClient({
        transport: new SummaryTransport(summary),
        store: requireClient().store,
        taskStore: requireTaskStore(),
    });
    await client.listTasks({ includeArchived: true });
    requireClient().store.selectTask(taskId);
    state.fixture = {
        taskId,
        projectId: summary.projectId,
        runId,
        generation: 1,
        eventIds: [],
        digests: [],
    };
    return { ok: true, record: summary };
}

async function listActiveSummary(
    taskId: string,
    latestRunId: string | null,
    mode: 'running' | 'cancelling',
): Promise<Outcome> {
    await ensureReady();
    const task = requireClient()
        .store.snapshot()
        .tasks.find((candidate) => candidate.taskId === taskId);
    if (task === undefined) {
        return { ok: false, code: 'task_not_found', message: 'task is not loaded' };
    }
    const summary: TaskSummary = {
        taskId: task.taskId,
        projectId: task.projectId,
        title: task.title,
        repo: task.repo,
        branch: task.branch,
        createdAtMs: task.createdAtMs,
        archived: task.archived,
        pinned: task.pinnedLocal,
        latestRunId,
        unresolvedRunId: null,
        state: mode,
    };
    const client = new AgentClient({
        transport: new SummaryTransport(summary),
        store: requireClient().store,
        taskStore: requireTaskStore(),
    });
    try {
        await client.listTasks({ includeArchived: true });
        return { ok: true, record: summary };
    } catch (error) {
        return failure(error);
    }
}

async function appendQueuedSafe(taskId: string, runId: string): Promise<Outcome> {
    try {
        const fixture: FixtureSummary = {
            taskId,
            projectId: 'prj_0000000000000001',
            runId,
            generation: 1,
            eventIds: [],
            digests: [],
        };
        const event = createRunQueuedEvent({
            projectId: fixture.projectId,
            taskId,
            runId,
            generation: 1,
            recordedAtMs: state.nowMs,
            queuedAtMs: state.nowMs,
        });
        const result = await requireTaskStore().appendEvent({ event });
        return appendOutcome(result);
    } catch (error) {
        return failure(error);
    }
}

async function appendAssistantSafe(
    text: string,
    options: { writerId?: string; nowMs?: number } = {},
): Promise<Outcome> {
    try {
        const fixture = currentFixture();
        const task = requireClient()
            .store.snapshot()
            .tasks.find((candidate) => candidate.taskId === fixture.taskId);
        const run =
            task?.runs.find((candidate) => candidate.runId === fixture.runId) ??
            null;
        const event = eventFor(
            fixture,
            run === null ? 3 : run.lastSequence + 1,
            run?.generation ?? fixture.generation,
            'assistant.message',
            { message_id: `msg_${randomHex(8)}`, text },
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
    } catch (error) {
        return failure(error);
    }
}

async function createClientRun(): Promise<FixtureSummary> {
    await ensureReady();
    const client = requireClient();
    const task = await client.createTask({ title: 'Client resume fixture' });
    const run = await client.startRun({
        taskId: task.taskId,
        prompt: 'resume fixture',
    });
    const deadline = Date.now() + 10_000;
    let cursor = client.store.getRunCursor(task.taskId, run.runId);
    while (
        (cursor === null || cursor.lastSequence < 1) &&
        Date.now() < deadline
    ) {
        await new Promise((resolve) => setTimeout(resolve, 10));
        cursor = client.store.getRunCursor(task.taskId, run.runId);
    }
    if (cursor === null || cursor.lastSequence < 1 || cursor.lastDigest === null) {
        throw new Error('client fixture run did not reach run.started');
    }
    const fixture: FixtureSummary = {
        taskId: task.taskId,
        projectId: task.projectId,
        runId: run.runId,
        generation: cursor.generation,
        eventIds: [
            fixtureEventId(run.runId, cursor.generation, 0),
            fixtureEventId(run.runId, cursor.generation, 1),
        ],
        digests: [cursor.lastDigest],
    };
    state.fixture = fixture;
    client.store.selectTask(task.taskId);
    return fixture;
}

async function createTaskAt(createdAtMs: number): Promise<Outcome> {
    try {
        await ensureReady();
        const record = await requireTaskStore().createTask({
            title: 'timestamp task',
            created_at_ms: createdAtMs,
        });
        return { ok: true, record };
    } catch (error) {
        return failure(error);
    }
}

async function resumeViaClient(): Promise<Outcome> {
    try {
        const fixture = currentFixture();
        const snapshot = await requireClient().resumeTask({
            taskId: fixture.taskId,
            runId: fixture.runId,
        });
        return { ok: true, record: snapshot };
    } catch (error) {
        return failure(error);
    }
}

async function recordChangesWithFiles(
    files: Array<Record<string, unknown>>,
): Promise<Outcome> {
    try {
        const fixture = currentFixture();
        const view = requireTaskStore().store.getRun(
            fixture.taskId,
            fixture.runId,
        );
        const latest = view?.changesSnapshots[view.changesSnapshots.length - 1];
        if (latest === undefined) {
            return {
                ok: false,
                code: 'invalid_arguments',
                message: 'no changes.snapshot is present',
            };
        }
        const record = await requireTaskStore().recordChanges({
            task_id: fixture.taskId,
            run_id: fixture.runId,
            head_digest: latest.headDigest,
            files,
            captured_at_ms: latest.capturedAtMs,
        });
        return { ok: true, record };
    } catch (error) {
        return failure(error);
    }
}

async function seedSmallEvents(total: number): Promise<{
    events: number;
    sequence: number;
    durationMs: number;
    heapUsed: number | null;
}> {
    await ensureReady();
    if (state.fixture === null) await createTaskAndEvents();
    if (!Number.isSafeInteger(total) || total < 3) {
        throw new Error('scale target must be at least 3 events');
    }
    const fixture = currentFixture();
    const taskStore = requireTaskStore();
    const client = requireClient();
    const startedAt = performance.now();
    let cursor = client.store.getRunCursor(fixture.taskId, fixture.runId);
    if (cursor === null) throw new Error('scale run is missing');
    while (cursor.lastSequence < total - 1) {
        const sequence = cursor.lastSequence + 1;
        if (sequence >= 10_000) {
            throw new Error('scale target exceeds max_events_per_run');
        }
        const event = eventFor(
            fixture,
            sequence,
            cursor.generation,
            'assistant.message',
            {
                message_id: `msg_${sequence.toString().padStart(8, '0')}`,
                text: 'x',
            },
        );
        const result = await taskStore.appendEvent({ event });
        if (result.status !== 'applied') {
            throw new Error(
                `scale append ${sequence} was rejected: ${result.code ?? ''} ${result.message ?? ''}`,
            );
        }
        fixture.eventIds.push(event.event_id);
        if (result.digest !== null) fixture.digests.push(result.digest);
        const next = client.store.getRunCursor(fixture.taskId, fixture.runId);
        if (next === null || next.lastSequence !== sequence) {
            throw new Error('scale cursor did not advance');
        }
        cursor = next;
    }
    const durationMs = performance.now() - startedAt;
    const heapUsed =
        (performance as unknown as { memory?: { usedJSHeapSize: number } })
            .memory?.usedJSHeapSize ?? null;
    const result = {
        events: cursor.lastSequence + 1,
        sequence: cursor.lastSequence,
        durationMs,
        heapUsed,
    };
    state.scale = result;
    return result;
}

function memoryCursorView(): CursorView | null {
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

function cursorView(): CursorView | null {
    if (state.phase === 'loading' || state.phase === 'error' || state.phase === 'idle') {
        return null;
    }
    return memoryCursorView();
}

const HOLD_TOOL_ID = 'tool_hold_probe';

function runtimeForTest(): AgentRuntime {
    const client = requireClient();
    return new AgentRuntime({
        store: client.store,
        taskStore: requireTaskStore(),
        modelPort: new ReplayModelPort(),
        now: () => state.nowMs,
        timers: {
            setInterval: testSetInterval,
            clearInterval: testClearInterval,
            setTimeout: (handler, timeout) =>
                globalThis.setTimeout(handler, timeout),
            clearTimeout: (handle) =>
                globalThis.clearTimeout(handle as never),
        },
        writerId: `runtime-test-${randomHex(8)}`,
    });
}

async function seedBackgroundHoldForTest(): Promise<Outcome> {
    try {
        const fixture = await createTaskAndEvents({ mode: 'cancelling' });
        const failed = await appendFixtureEvent(fixture, 'run.failed', {
            code: 'stop_unacknowledged',
            message: 'probe stop was not acknowledged',
            retryable: false,
            stop_outcome: 'ack_timeout',
            in_flight_tool_id: HOLD_TOOL_ID,
        });
        if (failed.ok !== true) return failed;
        const records = await requireTaskStore().listTasks({
            includeArchived: true,
        });
        const record = records.find(
            (candidate) => candidate.task_id === fixture.taskId,
        );
        if (record === undefined) {
            throw new Error('hold probe task record is missing');
        }
        await requireTaskStore().writeTaskRecord({
            ...record,
            hold: {
                reason: 'waiting_for_background',
                run_id: fixture.runId,
                tool_call_id: HOLD_TOOL_ID,
            },
        });
        return {
            ok: true,
            record: {
                taskId: fixture.taskId,
                runId: fixture.runId,
                toolCallId: HOLD_TOOL_ID,
            },
        };
    } catch (error) {
        return failure(error);
    }
}

async function startRunOnHeldTaskForTest(taskId: string): Promise<Outcome> {
    const runtime = runtimeForTest();
    try {
        await runtime.open();
        const run = await runtime.startRun({
            taskId,
            prompt: 'must not start on a hold',
        });
        return { ok: true, record: run };
    } catch (error) {
        return failure(error);
    } finally {
        await runtime.dispose();
    }
}

async function resolveBackgroundHoldForTest(
    taskId: string,
    runId: string,
    toolCallId: string,
): Promise<Outcome> {
    const runtime = runtimeForTest();
    try {
        await runtime.open();
        const receipt = await runtime.resolveBackground({
            taskId,
            runId,
            toolCallId,
        });
        return { ok: true, record: receipt };
    } catch (error) {
        return failure(error);
    } finally {
        await runtime.dispose();
    }
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
        unresolvedRunId: task.unresolvedRunId,
        hold: task.hold,
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
        async reopen() {
            if (state.taskStore === null) {
                return open();
            }
            state.taskStore.close();
            state.phase = 'idle';
            return open();
        },
        memoryCursor: memoryCursorView,
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
        createClientRun,
        createQueuedOnlyTask,
        targetFixture(taskId: string, runId: string, generation = 1) {
            state.fixture = {
                taskId,
                projectId: 'prj_0000000000000001',
                runId,
                generation,
                eventIds: [],
                digests: [],
            };
            requireClient().store.selectTask(taskId);
        },
        listUnloadedActiveTask,
        listActiveSummary,
        appendQueuedSafe,
        createTaskAt,
        resumeViaClient,
        recordChangesWithFiles,
        seedSmallEvents,
        scaleMetrics() {
            return state.scale;
        },
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
            const verification = requireClient().store.verifySnapshot({
                taskId: fixture.taskId,
                runId: fixture.runId,
                generation: run.generation,
                lastSequence: run.lastSequence,
                lastEventId: run.lastEventId!,
                headDigest: run.lastDigest,
            });
            if (!verification.ok) {
                return {
                    ok: false,
                    code: verification.code ?? 'snapshot_corrupt',
                    message: verification.message ?? 'snapshot verification failed',
                };
            }
            const result = await requireTaskStore().appendEvent({
                event,
                snapshot_anchor: {
                    task_id: fixture.taskId,
                    run_id: fixture.runId,
                    generation: run.generation,
                    last_sequence: run.lastSequence,
                    last_event_id: run.lastEventId!,
                    head_digest: run.lastDigest,
                },
            });
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
        seedBackgroundHoldForTest,
        startRunOnHeldTaskForTest,
        resolveBackgroundHoldForTest,
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
        armAbortNextTaskWrite() {
            state.faults.abort_next_task_write = true;
        },
        armFailImportPublish() {
            state.faults.fail_import_publish = true;
        },
        armFailImportCleanup() {
            state.faults.fail_import_cleanup = true;
        },
        disposeDuringImport() {
            state.client?.dispose();
        },
        async beginHeldQueuedAppend(taskId: string, runId: string) {
            await ensureReady();
            const gate = new Promise<void>((resolve) => {
                state.appendGateRelease = resolve;
            });
            state.appendGateReached = requireTaskStore().setAppendGate(gate);
            const fixture: FixtureSummary = {
                taskId,
                projectId: 'prj_0000000000000001',
                runId,
                generation: 1,
                eventIds: [],
                digests: [],
            };
            state.fixture = fixture;
            const queued = createRunQueuedEvent({
                projectId: fixture.projectId,
                taskId,
                runId,
                generation: 1,
                recordedAtMs: state.nowMs,
                queuedAtMs: state.nowMs,
            });
            state.heldAppend = (async () => {
                try {
                    const result = await requireTaskStore().appendEvent({
                        event: queued,
                    });
                    if (result.status === 'applied') {
                        fixture.eventIds.push(queued.event_id);
                        if (result.digest !== null) fixture.digests.push(result.digest);
                    }
                    return appendOutcome(result);
                } catch (error) {
                    return failure(error);
                }
            })();
        },
        async awaitAppendHeld() {
            if (state.appendGateReached !== null) {
                await state.appendGateReached;
            }
        },
        releaseHeldAppend() {
            const release = state.appendGateRelease;
            state.appendGateRelease = null;
            release?.();
        },
        async finishHeldAppend() {
            if (state.heldAppend === null) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'no held append is running',
                };
            }
            const result = await state.heldAppend;
            state.heldAppend = null;
            state.appendGateReached = null;
            return result;
        },
        async beginHeldTaskWrite(taskId: string) {
            await ensureReady();
            const gate = new Promise<void>((resolve) => {
                state.taskWriteGateRelease = resolve;
            });
            state.taskWriteGateReached = requireTaskStore().setTaskWriteGate(gate);
            const record = {
                task_id: taskId,
                project_id: 'prj_0000000000000001',
                title: 'Held task write',
                created_at_ms: state.nowMs,
                archived: false,
                pinned_local: false,
                latest_run_id: null,
            };
            state.heldTaskWrite = (async () => {
                try {
                    const written = await requireTaskStore().writeTaskRecord(record);
                    return { ok: true, record: written };
                } catch (error) {
                    return failure(error);
                }
            })();
        },
        async awaitTaskWriteHeld() {
            if (state.taskWriteGateReached !== null) {
                await state.taskWriteGateReached;
            }
        },
        releaseTaskWrite() {
            const release = state.taskWriteGateRelease;
            state.taskWriteGateRelease = null;
            release?.();
        },
        async finishHeldTaskWrite() {
            if (state.heldTaskWrite === null) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'no held task write is running',
                };
            }
            const result = await state.heldTaskWrite;
            state.heldTaskWrite = null;
            state.taskWriteGateReached = null;
            return result;
        },
        beginImport(text: string, options: { holdPublish?: boolean } = {}) {
            state.phase = 'loading';
            state.error = null;
            const gate = new Promise<void>((resolve) => {
                state.importGateRelease = resolve;
            });
            requireTaskStore().setImportGate(gate);
            if (options.holdPublish === true) {
                const publishGate = new Promise<void>((resolve) => {
                    state.importPublishGateRelease = resolve;
                });
                state.importPublishGateReached =
                    requireTaskStore().setImportPublishGate(publishGate);
            } else {
                state.importPublishGateRelease = null;
                state.importPublishGateReached = null;
            }
            state.importPromise = (async () => {
                try {
                    const summary = await requireTaskStore().importNdjson(text);
                    importFixtureFromStore();
                    state.phase = 'ready';
                    state.error = null;
                    return { ok: true, record: summary };
                } catch (error) {
                    state.phase = 'error';
                    state.error = { code: errorCode(error), message: errorMessage(error) };
                    return failure(error);
                }
            })();
        },
        async awaitImportPublishGate() {
            if (state.importPublishGateReached !== null) {
                await state.importPublishGateReached;
            }
        },
        releaseImportPublishGate() {
            const release = state.importPublishGateRelease;
            state.importPublishGateRelease = null;
            release?.();
        },
        async whenImport() {
            if (state.importGateRelease !== null) {
                const release = state.importGateRelease;
                state.importGateRelease = null;
                release();
            }
            if (state.importPromise === null) {
                return { ok: false, code: 'invalid_arguments', message: 'no import is running' };
            }
            const result = await state.importPromise;
            state.importPromise = null;
            return result;
        },
        async advanceImportClock(ms: number) {
            if (!Number.isFinite(ms) || ms < 0) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'ms must be a non-negative number',
                };
            }
            state.nowMs += ms;
            const due = state.timers.filter((timer) => timer.nextAtMs <= state.nowMs);
            for (const timer of due) {
                if (!state.timers.includes(timer)) continue;
                timer.nextAtMs += timer.periodMs;
                try {
                    await timer.callback();
                } catch (error) {
                    return failure(error);
                }
            }
            return {
                ok: true,
                record: { now_ms: state.nowMs, timers: state.timers.length },
            };
        },
        async freezeImportClock(ms: number) {
            if (!Number.isFinite(ms) || ms < 0) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'ms must be a non-negative number',
                };
            }
            state.nowMs += ms;
            return {
                ok: true,
                record: { now_ms: state.nowMs, timers: state.timers.length },
            };
        },
        appendAssistantSafe,
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
