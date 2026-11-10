import {
    AGENT_RUN_STATES,
    MAX_EVENT_BYTES,
    MAX_EVENTS_PER_RUN,
    type AgentErrorCode,
    type AgentEvent,
    type RunState,
} from './generated/agentContract.js';
import {
    CanonicalJsonError,
    canonicalJson,
    digestEvent,
    parseCanonicalJson,
} from './generated/agentCanonical.js';
import {
    AgentStore,
    validateAgentEvent,
    type AgentIncomingEvent,
    type AgentTaskRecord,
    type RunView,
} from '../state/agent.js';
import {
    AGENT_DB_NAME,
    AGENT_DB_STORES,
    AGENT_DB_VERSION,
    AgentStoreError,
    EXPORT_SCHEMA,
    LEASE_STALE_MS,
    MAX_EXPORT_BYTES,
    MAX_SNAPSHOT_BYTES,
    SNAPSHOT_SCHEMA,
    type AgentDbStoreName,
    type AppendEventStoreInput,
    type AppendStoreResult,
    type ChangeInput,
    type ChangeRecord,
    type CreateTaskStoreInput,
    type EventRecord,
    type ImportSummary,
    type LeaseRecord,
    type ListTaskOptions,
    type OpenReport,
    type ReadResult,
    type ReadRunResult,
    type RestoreIncomplete,
    type RunRecord,
    type SnapshotRecord,
    type TaskRecord,
} from './storeTypes.js';

const PROJECT_ID_PATTERN = /^prj_[0-9a-f]{16}$/;
const TASK_ID_PATTERN = /^tsk_[0-9a-f]{16}$/;
const RUN_ID_PATTERN = /^run_[0-9a-f]{16}$/;
const DIGEST_PATTERN = /^sha256:[0-9a-f]{64}$/;
const RUN_STATES: ReadonlySet<string> = new Set(AGENT_RUN_STATES);

interface StoreFaults {
    abort_next_append: boolean;
}

export interface AgentTaskStoreOptions {
    store: AgentStore;
    indexedDB?: IDBFactory;
    now?: () => number;
    writerId?: string;
    faults?: StoreFaults;
}

interface EventPair {
    record: EventRecord;
    key: IDBValidKey;
}

function byteLength(text: string): number {
    return new TextEncoder().encode(text).byteLength;
}

function isObject(value: unknown): value is Record<string, unknown> {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        return false;
    }
    const prototype = Object.getPrototypeOf(value);
    return prototype === Object.prototype || prototype === null;
}

function isNonNegativeInteger(value: unknown): value is number {
    return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
}

function isPositiveInteger(value: unknown): value is number {
    return typeof value === 'number' && Number.isSafeInteger(value) && value >= 1;
}

function isNonEmptyString(value: unknown): value is string {
    return typeof value === 'string' && value.length > 0;
}

function corrupt(message: string): never {
    throw new AgentStoreError('snapshot_corrupt', message);
}

function invalidArgument(message: string): never {
    throw new AgentStoreError('invalid_arguments', message);
}

function runKey(taskId: string, runId: string): string {
    return `${taskId}\u0000${runId}`;
}

function jsonClean<T>(value: T): T {
    return JSON.parse(JSON.stringify(value)) as T;
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

function storageError(error: unknown): AgentStoreError {
    if (error instanceof AgentStoreError) {
        return error;
    }
    return new AgentStoreError(
        'limit_exceeded',
        error instanceof Error ? error.message : 'indexeddb transaction failed',
        'indexeddb',
    );
}

function randomHex(bytes: number): string {
    const values = new Uint8Array(bytes);
    const crypto = globalThis.crypto;
    if (crypto === undefined || typeof crypto.getRandomValues !== 'function') {
        throw new AgentStoreError('runtime_unavailable', 'secure random ids are unavailable');
    }
    crypto.getRandomValues(values);
    return [...values]
        .map((value) => value.toString(16).padStart(2, '0'))
        .join('');
}

function requestResult<T>(request: IDBRequest<T>): Promise<T> {
    return new Promise<T>((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () =>
            reject(
                new AgentStoreError(
                    'limit_exceeded',
                    request.error?.name ?? 'indexeddb request failed',
                    'indexeddb',
                ),
            );
    });
}

function transactionDone(transaction: IDBTransaction): Promise<void> {
    return new Promise<void>((resolve, reject) => {
        let settled = false;
        const fail = () => {
            if (settled) return;
            settled = true;
            reject(
                new AgentStoreError(
                    'limit_exceeded',
                    transaction.error?.name ?? 'indexeddb transaction aborted',
                    'indexeddb',
                ),
            );
        };
        transaction.oncomplete = () => {
            if (settled) return;
            settled = true;
            resolve();
        };
        transaction.onabort = fail;
        transaction.onerror = fail;
    });
}

function openAgentDatabase(factory: IDBFactory): Promise<IDBDatabase> {
    return new Promise<IDBDatabase>((resolve, reject) => {
        const request = factory.open(AGENT_DB_NAME, AGENT_DB_VERSION);
        request.onupgradeneeded = () => {
            const db = request.result;
            const transaction = request.transaction;
            if (transaction === null) {
                reject(new AgentStoreError('internal', 'missing versionchange transaction'));
                return;
            }
            const ensure = (name: AgentDbStoreName): IDBObjectStore =>
                db.objectStoreNames.contains(name)
                    ? transaction.objectStore(name)
                    : db.createObjectStore(name);
            for (const name of AGENT_DB_STORES) {
                ensure(name);
            }
            const events = transaction.objectStore('events');
            if (!events.indexNames.contains('by_event_id')) {
                events.createIndex('by_event_id', 'event_id', { unique: true });
            }
        };
        request.onsuccess = () => {
            const db = request.result;
            db.onversionchange = () => db.close();
            resolve(db);
        };
        request.onerror = () =>
            reject(
                new AgentStoreError(
                    'runtime_unavailable',
                    request.error?.name ?? 'could not open the agent database',
                ),
            );
        request.onblocked = () =>
            reject(
                new AgentStoreError(
                    'runtime_unavailable',
                    'agent database upgrade is blocked by another connection',
                ),
            );
    });
}

function deleteAgentDatabase(factory: IDBFactory): Promise<void> {
    return new Promise<void>((resolve, reject) => {
        const request = factory.deleteDatabase(AGENT_DB_NAME);
        request.onsuccess = () => resolve();
        request.onerror = () =>
            reject(
                new AgentStoreError(
                    'runtime_unavailable',
                    request.error?.name ?? 'could not delete the agent database',
                ),
            );
        request.onblocked = () =>
            reject(
                new AgentStoreError(
                    'runtime_unavailable',
                    'agent database deletion is blocked by another connection',
                ),
            );
    });
}

function requireStoredTask(value: unknown): asserts value is TaskRecord {
    if (!isObject(value)) corrupt('task record is not an object');
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('task record task_id is invalid');
    }
    if (typeof value.project_id !== 'string' || !PROJECT_ID_PATTERN.test(value.project_id)) {
        corrupt('task record project_id is invalid');
    }
    if (typeof value.title !== 'string' || !isNonNegativeInteger(value.created_at_ms)) {
        corrupt('task record title or created_at_ms is invalid');
    }
    if (typeof value.archived !== 'boolean' || typeof value.pinned_local !== 'boolean') {
        corrupt('task record flags are invalid');
    }
    if (
        value.latest_run_id !== undefined &&
        value.latest_run_id !== null &&
        (typeof value.latest_run_id !== 'string' || !RUN_ID_PATTERN.test(value.latest_run_id))
    ) {
        corrupt('task record latest_run_id is invalid');
    }
    if (value.repo !== undefined && typeof value.repo !== 'string') {
        corrupt('task record repo is invalid');
    }
    if (value.branch !== undefined && typeof value.branch !== 'string') {
        corrupt('task record branch is invalid');
    }
    if (value.hold !== undefined) {
        if (
            !isObject(value.hold) ||
            value.hold.reason !== 'waiting_for_background' ||
            typeof value.hold.run_id !== 'string' ||
            !RUN_ID_PATTERN.test(value.hold.run_id) ||
            typeof value.hold.tool_call_id !== 'string'
        ) {
            corrupt('task record hold is invalid');
        }
    }
}

function requireStoredRun(value: unknown): asserts value is RunRecord {
    if (!isObject(value)) corrupt('run record is not an object');
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('run record task_id is invalid');
    }
    if (typeof value.run_id !== 'string' || !RUN_ID_PATTERN.test(value.run_id)) {
        corrupt('run record run_id is invalid');
    }
    if (!isPositiveInteger(value.generation)) {
        corrupt('run record generation is invalid');
    }
    if (typeof value.state !== 'string' || !RUN_STATES.has(value.state)) {
        corrupt('run record state is invalid');
    }
    if (
        value.retry_of_run_id !== null &&
        (typeof value.retry_of_run_id !== 'string' || !RUN_ID_PATTERN.test(value.retry_of_run_id))
    ) {
        corrupt('run record retry_of_run_id is invalid');
    }
    if (!isNonNegativeInteger(value.started_at_ms)) {
        corrupt('run record started_at_ms is invalid');
    }
    if (value.ended_at_ms !== undefined && !isNonNegativeInteger(value.ended_at_ms)) {
        corrupt('run record ended_at_ms is invalid');
    }
    if (!isNonNegativeInteger(value.last_sequence)) {
        corrupt('run record last_sequence is invalid');
    }
    if (
        value.last_event_id !== null &&
        typeof value.last_event_id !== 'string'
    ) {
        corrupt('run record last_event_id is invalid');
    }
    if (
        value.last_digest !== null &&
        (typeof value.last_digest !== 'string' || !DIGEST_PATTERN.test(value.last_digest))
    ) {
        corrupt('run record last_digest is invalid');
    }
}

function requireStoredEvent(value: unknown): asserts value is EventRecord {
    if (!isObject(value)) corrupt('event record is not an object');
    if (typeof value.event_id !== 'string' || value.event_id.length === 0) {
        corrupt('event record event_id is invalid');
    }
    if (
        typeof value.canonical !== 'string' ||
        byteLength(value.canonical) > MAX_EVENT_BYTES
    ) {
        corrupt('event record canonical text is invalid');
    }
    if (typeof value.digest !== 'string' || !DIGEST_PATTERN.test(value.digest)) {
        corrupt('event record digest is invalid');
    }
    if (
        value.prev_digest !== null &&
        (typeof value.prev_digest !== 'string' || !DIGEST_PATTERN.test(value.prev_digest))
    ) {
        corrupt('event record prev_digest is invalid');
    }
    if (!isNonNegativeInteger(value.recorded_at_ms)) {
        corrupt('event record recorded_at_ms is invalid');
    }
}

function requireStoredSnapshot(value: unknown): asserts value is SnapshotRecord {
    if (!isObject(value)) corrupt('snapshot record is not an object');
    if (value.schema !== SNAPSHOT_SCHEMA) {
        corrupt('snapshot schema is not supported');
    }
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('snapshot task_id is invalid');
    }
    if (typeof value.run_id !== 'string' || !RUN_ID_PATTERN.test(value.run_id)) {
        corrupt('snapshot run_id is invalid');
    }
    if (!isPositiveInteger(value.generation)) {
        corrupt('snapshot generation is invalid');
    }
    if (typeof value.last_event_id !== 'string' || value.last_event_id.length === 0) {
        corrupt('snapshot last_event_id is invalid');
    }
    if (!isNonNegativeInteger(value.last_sequence)) {
        corrupt('snapshot last_sequence is invalid');
    }
    if (typeof value.head_digest !== 'string' || !DIGEST_PATTERN.test(value.head_digest)) {
        corrupt('snapshot head_digest is invalid');
    }
    if (!isNonNegativeInteger(value.created_at_ms)) {
        corrupt('snapshot created_at_ms is invalid');
    }
}

function requireStoredChange(value: unknown): asserts value is ChangeRecord {
    if (!isObject(value)) corrupt('change record is not an object');
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('change record task_id is invalid');
    }
    if (typeof value.run_id !== 'string' || !RUN_ID_PATTERN.test(value.run_id)) {
        corrupt('change record run_id is invalid');
    }
    if (value.scope !== 'last_turn') {
        corrupt('change record scope is invalid');
    }
    if (!Array.isArray(value.files) || !value.files.every((file) => isObject(file))) {
        corrupt('change record files are invalid');
    }
    if (!isNonEmptyString(value.head_digest)) {
        corrupt('change record head_digest is invalid');
    }
    if (!isNonNegativeInteger(value.captured_at_ms)) {
        corrupt('change record captured_at_ms is invalid');
    }
}

function requireStoredLease(value: unknown): asserts value is LeaseRecord {
    if (!isObject(value)) corrupt('lease record is not an object');
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('lease task_id is invalid');
    }
    if (!isNonEmptyString(value.writer_id)) {
        corrupt('lease writer_id is invalid');
    }
    if (
        !isNonNegativeInteger(value.started_at_ms) ||
        !isNonNegativeInteger(value.heartbeat_at_ms)
    ) {
        corrupt('lease timestamps are invalid');
    }
}

function taskToAgentRecord(record: TaskRecord): AgentTaskRecord {
    return {
        taskId: record.task_id,
        projectId: record.project_id,
        title: record.title,
        repo: record.repo ?? null,
        branch: record.branch ?? null,
        createdAtMs: record.created_at_ms,
        archived: record.archived,
        pinnedLocal: record.pinned_local,
        latestRunId: record.latest_run_id ?? null,
    };
}

function viewToTaskRecord(view: {
    taskId: string;
    projectId: string;
    title: string;
    repo: string | null;
    branch: string | null;
    createdAtMs: number;
    archived: boolean;
    pinnedLocal: boolean;
    latestRunId: string | null;
}): TaskRecord {
    return {
        task_id: view.taskId,
        project_id: view.projectId,
        title: view.title,
        ...(view.repo === null ? {} : { repo: view.repo }),
        ...(view.branch === null ? {} : { branch: view.branch }),
        created_at_ms: view.createdAtMs,
        archived: view.archived,
        pinned_local: view.pinnedLocal,
        latest_run_id: view.latestRunId,
    };
}

function snapshotVerification(snapshot: SnapshotRecord): {
    taskId: string;
    runId: string;
    generation: number;
    lastSequence: number;
    lastEventId: string;
    headDigest: string;
} {
    return {
        taskId: snapshot.task_id,
        runId: snapshot.run_id,
        generation: snapshot.generation,
        lastSequence: snapshot.last_sequence,
        lastEventId: snapshot.last_event_id,
        headDigest: snapshot.head_digest,
    };
}

function snapshotMatchesEvent(
    snapshot: SnapshotRecord,
    event: AgentIncomingEvent,
    digest: string,
): boolean {
    return (
        snapshot.generation === event.generation &&
        snapshot.last_sequence === event.sequence &&
        snapshot.last_event_id === event.event_id &&
        snapshot.head_digest === digest
    );
}

export class AgentTaskStore {
    readonly store: AgentStore;

    private readonly factory: IDBFactory;
    private readonly clock: () => number;
    private readonly writerId: string;
    private readonly faults: StoreFaults;
    private db: IDBDatabase | null = null;
    private opened = false;
    private openPromise: Promise<OpenReport> | null = null;
    private lastReport: OpenReport | null = null;

    constructor(options: AgentTaskStoreOptions) {
        this.store = options.store;
        this.factory = options.indexedDB ?? globalThis.indexedDB;
        if (this.factory === undefined) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'IndexedDB is not available in this browser',
            );
        }
        this.clock = options.now ?? (() => Date.now());
        this.writerId = options.writerId ?? `writer-${randomHex(8)}`;
        this.faults = options.faults ?? { abort_next_append: false };
    }

    get isOpen(): boolean {
        return this.opened && this.db !== null;
    }

    get defaultWriterId(): string {
        return this.writerId;
    }

    async open(): Promise<OpenReport> {
        if (this.opened && this.lastReport !== null) {
            return this.lastReport;
        }
        if (this.openPromise !== null) {
            return this.openPromise;
        }
        this.openPromise = this.restore();
        try {
            const report = await this.openPromise;
            this.opened = true;
            this.lastReport = report;
            return report;
        } finally {
            this.openPromise = null;
        }
    }

    close(): void {
        this.db?.close();
        this.db = null;
        this.opened = false;
        this.openPromise = null;
        this.lastReport = null;
    }

    async deleteDatabase(): Promise<void> {
        this.close();
        await deleteAgentDatabase(this.factory);
    }

    async createTask(input: CreateTaskStoreInput): Promise<TaskRecord> {
        this.requireOpen();
        const title = input.title.trim();
        if (title.length === 0) invalidArgument('task title must not be empty');
        const projectId = input.project_id ?? 'prj_0000000000000001';
        if (!PROJECT_ID_PATTERN.test(projectId)) {
            invalidArgument('project_id must match prj_<16 lowercase hex>');
        }
        const taskId = input.task_id ?? `tsk_${randomHex(8)}`;
        if (!TASK_ID_PATTERN.test(taskId)) {
            invalidArgument('task_id must match tsk_<16 lowercase hex>');
        }
        if (this.store.snapshot().tasks.some((task) => task.taskId === taskId)) {
            invalidArgument('task_id already exists');
        }
        if (input.repo !== undefined && input.repo !== null && typeof input.repo !== 'string') {
            invalidArgument('repo must be a string');
        }
        if (input.branch !== undefined && input.branch !== null && typeof input.branch !== 'string') {
            invalidArgument('branch must be a string');
        }
        const record: TaskRecord = {
            task_id: taskId,
            project_id: projectId,
            title,
            ...(input.repo === undefined || input.repo === null ? {} : { repo: input.repo }),
            ...(input.branch === undefined || input.branch === null ? {} : { branch: input.branch }),
            created_at_ms: input.created_at_ms ?? this.clock(),
            archived: false,
            pinned_local: false,
            latest_run_id: null,
        };
        await this.persistTaskWithRollback(record);
        return record;
    }

    async writeTaskRecord(record: TaskRecord): Promise<TaskRecord> {
        this.requireOpen();
        requireStoredTask(record);
        const latest = record.latest_run_id ?? null;
        const hasRun =
            latest !== null && this.store.getRun(record.task_id, latest) !== null;
        const normalized: TaskRecord = {
            ...jsonClean(record),
            latest_run_id: hasRun ? latest : null,
        };
        await this.persistTaskWithRollback(normalized);
        return normalized;
    }

    async pinTask(taskId: string, pinned: boolean): Promise<TaskRecord> {
        return this.updateTaskFlag(taskId, (record) => {
            record.pinned_local = pinned;
        });
    }

    async archiveTask(taskId: string, archived: boolean): Promise<TaskRecord> {
        return this.updateTaskFlag(taskId, (record) => {
            record.archived = archived;
        });
    }

    async listTasks(options: ListTaskOptions = {}): Promise<TaskRecord[]> {
        this.requireOpen();
        const records = (await this.readAll<TaskRecord>('tasks'))
            .map((record) => {
                requireStoredTask(record);
                return record;
            })
            .filter((record) => options.includeArchived === true || !record.archived)
            .filter((record) => {
                const query = options.query?.trim().toLowerCase() ?? '';
                return query.length === 0 || record.title.toLowerCase().includes(query);
            })
            .map((record) => {
                const latest = record.latest_run_id ?? null;
                const exists =
                    latest !== null && this.store.getRun(record.task_id, latest) !== null;
                return {
                    ...record,
                    latest_run_id: exists ? latest : null,
                };
            });
        records.sort((left, right) =>
            left.created_at_ms === right.created_at_ms
                ? left.task_id.localeCompare(right.task_id)
                : left.created_at_ms - right.created_at_ms,
        );
        return records;
    }

    async readRun(taskId: string, runId: string): Promise<ReadRunResult> {
        this.requireOpen();
        const run = await this.readRunRecord(taskId, runId);
        if (run === null) {
            return { ok: false, code: 'run_not_found', message: 'run record is missing' };
        }
        const pairs = await this.readEventPairs(taskId, runId);
        if (pairs.length === 0) {
            return {
                ok: false,
                code: 'snapshot_missing',
                message: 'run record has no canonical event records',
            };
        }
        try {
            const verified = await this.verifyEventChain(taskId, runId, pairs);
            this.assertRunRecordMatches(run, verified);
            return {
                ok: true,
                run,
                events: pairs.map((pair) => pair.record),
            };
        } catch (error) {
            if (error instanceof AgentStoreError) {
                return { ok: false, code: error.code, message: errorMessage(error) };
            }
            return {
                ok: false,
                code: 'snapshot_corrupt',
                message: errorMessage(error),
            };
        }
    }

    async writeSnapshot(taskId: string, runId: string): Promise<SnapshotRecord> {
        this.requireOpen();
        const view = this.store.getRun(taskId, runId);
        if (view === null || view.lastEventId === null || view.lastDigest === null) {
            throw new AgentStoreError(
                'snapshot_missing',
                'run has no verified event chain to snapshot',
            );
        }
        const read = await this.readRun(taskId, runId);
        if (!read.ok) {
            throw new AgentStoreError(read.code, read.message);
        }
        if (
            read.run.last_event_id !== view.lastEventId ||
            read.run.last_sequence !== view.lastSequence ||
            read.run.last_digest !== view.lastDigest
        ) {
            throw new AgentStoreError(
                'snapshot_corrupt',
                'in-memory run cursor does not match the durable event chain',
            );
        }
        const snapshot = this.buildSnapshot(view);
        if (byteLength(canonicalJson(jsonClean(snapshot))) > MAX_SNAPSHOT_BYTES) {
            throw new AgentStoreError('limit_exceeded', 'snapshot exceeds max_snapshot_bytes');
        }
        const db = this.requireDb();
        const transaction = db.transaction(['snapshots', 'runs'], 'readwrite');
        try {
            const storedRun = (await requestResult(
                transaction.objectStore('runs').get([taskId, runId]),
            )) as RunRecord | undefined;
            if (
                storedRun === undefined ||
                storedRun.last_event_id !== snapshot.last_event_id ||
                storedRun.last_sequence !== snapshot.last_sequence ||
                storedRun.last_digest !== snapshot.head_digest
            ) {
                throw new AgentStoreError(
                    'snapshot_corrupt',
                    'durable run record moved before the snapshot was written',
                );
            }
            await requestResult(
                transaction
                    .objectStore('snapshots')
                    .put(jsonClean(snapshot), [taskId, runId]),
            );
            await requestResult(
                transaction.objectStore('runs').put(jsonClean(storedRun), [taskId, runId]),
            );
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
        const verification = this.store.verifySnapshot(snapshotVerification(snapshot));
        if (!verification.ok) {
            throw new AgentStoreError(
                verification.code ?? 'snapshot_corrupt',
                verification.message ?? 'snapshot verification failed',
            );
        }
        return snapshot;
    }

    async readSnapshot(taskId: string, runId: string): Promise<ReadResult<SnapshotRecord>> {
        this.requireOpen();
        const stored = (await this.readOne<SnapshotRecord>('snapshots', [taskId, runId])) as
            | SnapshotRecord
            | undefined;
        if (stored === undefined) {
            return { ok: false, code: 'snapshot_missing', message: 'snapshot record is missing' };
        }
        try {
            requireStoredSnapshot(stored);
        } catch (error) {
            return { ok: false, code: 'snapshot_corrupt', message: errorMessage(error) };
        }
        const read = await this.readRun(taskId, runId);
        if (!read.ok) {
            return {
                ok: false,
                code: read.code === 'snapshot_missing' ? 'snapshot_corrupt' : read.code,
                message: read.message,
            };
        }
        const pairs = await this.readEventPairs(taskId, runId);
        const matching = pairs.find((pair) => {
            const key = pair.key;
            return (
                Array.isArray(key) &&
                key.length === 4 &&
                key[2] === stored.generation &&
                key[3] === stored.last_sequence &&
                pair.record.event_id === stored.last_event_id &&
                pair.record.digest === stored.head_digest
            );
        });
        if (matching === undefined) {
            return {
                ok: false,
                code: 'snapshot_corrupt',
                message: 'snapshot position does not match the durable event chain',
            };
        }
        return { ok: true, record: stored };
    }

    async recordChanges(input: ChangeInput): Promise<ChangeRecord> {
        this.requireOpen();
        if (!TASK_ID_PATTERN.test(input.task_id)) invalidArgument('task_id is invalid');
        if (!RUN_ID_PATTERN.test(input.run_id)) invalidArgument('run_id is invalid');
        if (!isNonEmptyString(input.head_digest)) invalidArgument('head_digest must be non-empty');
        if (
            input.scope !== undefined &&
            input.scope !== 'last_turn'
        ) {
            invalidArgument('change scope must be last_turn');
        }
        if (!Array.isArray(input.files) || !input.files.every((file) => isObject(file))) {
            invalidArgument('change files must be an array of objects');
        }
        if (!isNonNegativeInteger(input.captured_at_ms)) {
            invalidArgument('captured_at_ms must be a non-negative safe integer');
        }
        const view = this.store.getRun(input.task_id, input.run_id);
        if (view === null) {
            throw new AgentStoreError('run_not_found', 'run is not present in AgentStore');
        }
        const bound = view.changesSnapshots.some(
            (snapshot) =>
                snapshot.scope === 'last_turn' &&
                snapshot.headDigest === input.head_digest,
        );
        if (!bound) {
            invalidArgument(
                'head_digest is not bound to an applied changes.snapshot event',
            );
        }
        const record: ChangeRecord = {
            task_id: input.task_id,
            run_id: input.run_id,
            scope: 'last_turn',
            files: jsonClean(input.files),
            head_digest: input.head_digest,
            captured_at_ms: input.captured_at_ms,
        };
        const transaction = this.requireDb().transaction('changes', 'readwrite');
        try {
            await requestResult(
                transaction
                    .objectStore('changes')
                    .put(jsonClean(record), [
                        record.task_id,
                        record.run_id,
                        record.head_digest,
                    ]),
            );
            await transactionDone(transaction);
        } catch (error) {
            throw storageError(error);
        }
        return record;
    }

    async readChanges(taskId: string, runId?: string): Promise<ChangeRecord[]> {
        this.requireOpen();
        const records = (await this.readAll<ChangeRecord>('changes'))
            .map((record) => {
                requireStoredChange(record);
                return record;
            })
            .filter(
                (record) =>
                    record.task_id === taskId &&
                    (runId === undefined || record.run_id === runId),
            );
        records.sort((left, right) =>
            left.captured_at_ms === right.captured_at_ms
                ? left.head_digest.localeCompare(right.head_digest)
                : left.captured_at_ms - right.captured_at_ms,
        );
        return records;
    }

    async acquireLease(
        taskId: string,
        writerId: string,
        nowMs = this.clock(),
    ): Promise<LeaseRecord> {
        this.requireOpen();
        this.requireWriter(writerId);
        if (!TASK_ID_PATTERN.test(taskId)) invalidArgument('task_id is invalid');
        const task = this.store.snapshot().tasks.find(
            (candidate) => candidate.taskId === taskId,
        );
        if (task === undefined) {
            throw new AgentStoreError('task_not_found', 'task is not present in AgentStore');
        }
        if (task.latestRunId !== null) {
            const read = await this.readRun(taskId, task.latestRunId);
            if (!read.ok) {
                throw new AgentStoreError(read.code, read.message);
            }
            const storedSnapshot = await this.readOne<SnapshotRecord>(
                'snapshots',
                [taskId, task.latestRunId],
            );
            if (storedSnapshot !== undefined) {
                const snapshotRead = await this.readSnapshot(
                    taskId,
                    task.latestRunId,
                );
                if (!snapshotRead.ok) {
                    throw new AgentStoreError(
                        snapshotRead.code,
                        snapshotRead.message,
                    );
                }
            }
        }
        if (!isNonNegativeInteger(nowMs)) invalidArgument('now_ms is invalid');
        const transaction = this.requireDb().transaction('leases', 'readwrite');
        try {
            const stores = transaction.objectStore('leases');
            const existing = (await requestResult(stores.get(taskId))) as LeaseRecord | undefined;
            if (existing !== undefined) {
                requireStoredLease(existing);
            }
            const activeOther =
                existing !== undefined &&
                existing.writer_id !== writerId &&
                nowMs - existing.heartbeat_at_ms <= LEASE_STALE_MS;
            if (activeOther) {
                throw new AgentStoreError(
                    'stale_generation',
                    'task lease is held by another active writer',
                );
            }
            const record: LeaseRecord =
                existing !== undefined && existing.writer_id === writerId
                    ? {
                          task_id: taskId,
                          writer_id: writerId,
                          started_at_ms: existing.started_at_ms,
                          heartbeat_at_ms: nowMs,
                      }
                    : {
                          task_id: taskId,
                          writer_id: writerId,
                          started_at_ms: nowMs,
                          heartbeat_at_ms: nowMs,
                      };
            await requestResult(stores.put(jsonClean(record), taskId));
            await transactionDone(transaction);
            return record;
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    async heartbeatLease(
        taskId: string,
        writerId: string,
        nowMs = this.clock(),
    ): Promise<LeaseRecord> {
        this.requireOpen();
        this.requireWriter(writerId);
        if (!isNonNegativeInteger(nowMs)) invalidArgument('now_ms is invalid');
        const transaction = this.requireDb().transaction('leases', 'readwrite');
        try {
            const stores = transaction.objectStore('leases');
            const existing = (await requestResult(stores.get(taskId))) as LeaseRecord | undefined;
            if (existing === undefined || existing.writer_id !== writerId) {
                throw new AgentStoreError(
                    'stale_generation',
                    'lease is missing or held by another writer',
                );
            }
            requireStoredLease(existing);
            const record: LeaseRecord = {
                ...existing,
                heartbeat_at_ms: nowMs,
            };
            await requestResult(stores.put(jsonClean(record), taskId));
            await transactionDone(transaction);
            return record;
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    async releaseLease(taskId: string, writerId: string): Promise<{ released: boolean }> {
        this.requireOpen();
        this.requireWriter(writerId);
        const transaction = this.requireDb().transaction('leases', 'readwrite');
        try {
            const stores = transaction.objectStore('leases');
            const existing = (await requestResult(stores.get(taskId))) as LeaseRecord | undefined;
            if (existing === undefined) {
                await transactionDone(transaction);
                return { released: false };
            }
            requireStoredLease(existing);
            if (existing.writer_id !== writerId) {
                throw new AgentStoreError(
                    'stale_generation',
                    'lease is held by another writer',
                );
            }
            await requestResult(stores.delete(taskId));
            await transactionDone(transaction);
            return { released: true };
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    async readLease(taskId: string): Promise<LeaseRecord | null> {
        this.requireOpen();
        const record = (await this.readOne<LeaseRecord>('leases', taskId)) as
            | LeaseRecord
            | undefined;
        if (record === undefined) return null;
        requireStoredLease(record);
        return record;
    }

    async appendEvent(input: AppendEventStoreInput): Promise<AppendStoreResult> {
        this.requireOpen();
        const event = input.event;
        const now = input.now_ms ?? this.clock();
        let canonical: string;
        let digest: string;
        try {
            canonical = canonicalJson(event);
            digest = await digestEvent(event as AgentEvent);
        } catch (error) {
            const code: AgentErrorCode =
                error instanceof CanonicalJsonError ? error.code : 'invalid_event';
            return {
                status: 'rejected',
                code,
                message: errorMessage(error),
                event_id: event.event_id,
                digest: null,
                storage: null,
            };
        }
        if (byteLength(canonical) > MAX_EVENT_BYTES) {
            return {
                status: 'rejected',
                code: 'limit_exceeded',
                message: 'event exceeds max_event_bytes',
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }
        if (input.digest !== undefined && input.digest !== digest) {
            return {
                status: 'rejected',
                code: 'invalid_event',
                message: 'provided digest does not match the recomputed canonical digest',
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }

        const checkpoint = this.store.checkpoint();
        let result;
        try {
            result = this.store.apply(event, digest);
        } catch (error) {
            this.store.restore(checkpoint);
            return {
                status: 'rejected',
                code: 'internal',
                message: errorMessage(error),
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }
        if (result.status !== 'applied') {
            return {
                ...result,
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }

        try {
            await this.commitEvent(event, canonical, digest, input, now);
        } catch (error) {
            this.store.restore(checkpoint);
            if (error instanceof AgentStoreError && error.storage === null) {
                return {
                    status: 'rejected',
                    code: error.code,
                    message: errorMessage(error),
                    event_id: event.event_id,
                    digest,
                    storage: null,
                };
            }
            return {
                status: 'rejected',
                code: 'limit_exceeded',
                message: errorMessage(error),
                event_id: event.event_id,
                digest,
                storage: 'indexeddb',
            };
        }
        return {
            status: 'applied',
            code: null,
            message: null,
            event_id: event.event_id,
            digest,
            storage: null,
        };
    }

    async exportNdjson(): Promise<string> {
        this.requireOpen();
        const tasks = (await this.readAll<TaskRecord>('tasks')).map((record) => {
            requireStoredTask(record);
            return record;
        });
        const runs = (await this.readAll<RunRecord>('runs')).map((record) => {
            requireStoredRun(record);
            return record;
        });
        const snapshots = (await this.readAll<SnapshotRecord>('snapshots')).map((record) => {
            requireStoredSnapshot(record);
            return record;
        });
        const changes = (await this.readAll<ChangeRecord>('changes')).map((record) => {
            requireStoredChange(record);
            return record;
        });
        const eventPairs = await this.readAllEventPairs();
        const eventGroups = new Map<string, EventPair[]>();
        for (const pair of eventPairs) {
            const key = this.pairRunKey(pair);
            const group = eventGroups.get(key);
            if (group === undefined) {
                eventGroups.set(key, [pair]);
            } else {
                group.push(pair);
            }
        }
        const runKeys = new Set(runs.map((run) => runKey(run.task_id, run.run_id)));
        for (const key of eventGroups.keys()) {
            if (!runKeys.has(key)) {
                corrupt('event records exist without a run record');
            }
        }
        const sortedRuns = [...runs].sort(
            (left, right) =>
                left.task_id.localeCompare(right.task_id) ||
                left.generation - right.generation ||
                left.run_id.localeCompare(right.run_id),
        );
        for (const run of sortedRuns) {
            const pairs = eventGroups.get(runKey(run.task_id, run.run_id)) ?? [];
            if (pairs.length === 0) {
                corrupt('run record has no canonical event records');
            }
            const verified = await this.verifyEventChain(run.task_id, run.run_id, pairs);
            this.assertRunRecordMatches(run, verified);
        }
        const runMap = new Map(
            sortedRuns.map((run) => [runKey(run.task_id, run.run_id), run]),
        );
        for (const snapshot of snapshots) {
            const run = runMap.get(runKey(snapshot.task_id, snapshot.run_id));
            if (run === undefined) {
                corrupt('snapshot record exists without a run record');
            }
            const pairs = eventGroups.get(runKey(snapshot.task_id, snapshot.run_id)) ?? [];
            const verified = await this.verifyEventChain(
                snapshot.task_id,
                snapshot.run_id,
                pairs,
            );
            const index = verified.events.findIndex(
                (event) =>
                    event.generation === snapshot.generation &&
                    event.sequence === snapshot.last_sequence,
            );
            if (
                index < 0 ||
                verified.events[index]!.event_id !== snapshot.last_event_id ||
                verified.digests[index] !== snapshot.head_digest
            ) {
                corrupt('snapshot position does not match the exported event chain');
            }
        }
        const lines: string[] = [];
        const push = (value: unknown) => {
            lines.push(canonicalJson(jsonClean(value)));
        };
        push({
            kind: 'header',
            schema: EXPORT_SCHEMA,
            exported_at_ms: this.clock(),
        });
        for (const task of [...tasks].sort(
            (left, right) =>
                left.created_at_ms - right.created_at_ms ||
                left.task_id.localeCompare(right.task_id),
        )) {
            push({ kind: 'task', record: task });
        }
        for (const run of sortedRuns) {
            push({ kind: 'run', record: run });
        }
        for (const pair of [...eventPairs].sort((left, right) =>
            this.comparePairs(left, right),
        )) {
            const record = pair.record;
            const event = await this.materializeEvent(record);
            push({
                kind: 'event',
                task_id: event.task_id,
                run_id: event.run_id,
                generation: event.generation,
                sequence: event.sequence,
                record,
            });
        }
        for (const snapshot of snapshots) {
            push({ kind: 'snapshot', record: snapshot });
        }
        for (const change of changes) {
            push({ kind: 'changes', record: change });
        }
        const text = `${lines.join('\n')}\n`;
        if (byteLength(text) > MAX_EXPORT_BYTES) {
            throw new AgentStoreError(
                'limit_exceeded',
                'agent export exceeds max_export_bytes',
            );
        }
        return text;
    }

    async importNdjson(text: string): Promise<ImportSummary> {
        this.requireOpen();
        if (typeof text !== 'string' || byteLength(text) > MAX_EXPORT_BYTES) {
            throw new AgentStoreError('limit_exceeded', 'import exceeds max_export_bytes');
        }
        const rawLines = text.split('\n');
        while (rawLines.length > 0 && rawLines[rawLines.length - 1] === '') {
            rawLines.pop();
        }
        if (rawLines.length === 0) invalidArgument('import contains no NDJSON records');
        const lines = rawLines.map((line) => {
            if (byteLength(line) > MAX_EVENT_BYTES + 65536) {
                throw new AgentStoreError('limit_exceeded', 'import line exceeds its bound');
            }
            let canonical: string;
            try {
                canonical = parseCanonicalJson(line);
            } catch (error) {
                throw new AgentStoreError(
                    'snapshot_corrupt',
                    `import line is not canonical JSON: ${errorMessage(error)}`,
                );
            }
            if (canonical !== line) {
                corrupt('import line is canonicalizable but not in canonical form');
            }
            try {
                return JSON.parse(canonical) as Record<string, unknown>;
            } catch {
                corrupt('import line could not be materialized');
            }
        });
        const header = lines[0]!;
        if (header.kind !== 'header' || header.schema !== EXPORT_SCHEMA) {
            invalidArgument('import must begin with the agent export header');
        }
        const tasks: TaskRecord[] = [];
        const runs: RunRecord[] = [];
        const snapshots: SnapshotRecord[] = [];
        const changes: ChangeRecord[] = [];
        const events: EventPair[] = [];
        const seenEventKeys = new Set<string>();
        const seenRunKeys = new Set<string>();
        const seenSnapshotKeys = new Set<string>();
        const seenChangeKeys = new Set<string>();
        for (let index = 1; index < lines.length; index += 1) {
            const line = lines[index]!;
            if (line.kind === 'task') {
                const record = line.record;
                requireStoredTask(record);
                tasks.push(record);
            } else if (line.kind === 'run') {
                const record = line.record;
                requireStoredRun(record);
                const key = runKey(record.task_id, record.run_id);
                if (seenRunKeys.has(key)) corrupt('import repeats a run record');
                seenRunKeys.add(key);
                runs.push(record);
            } else if (line.kind === 'snapshot') {
                const record = line.record;
                requireStoredSnapshot(record);
                const key = runKey(record.task_id, record.run_id);
                if (seenSnapshotKeys.has(key)) corrupt('import repeats a snapshot record');
                seenSnapshotKeys.add(key);
                snapshots.push(record);
            } else if (line.kind === 'changes') {
                const record = line.record;
                requireStoredChange(record);
                const key = `${record.task_id}\u0000${record.run_id}\u0000${record.head_digest}`;
                if (seenChangeKeys.has(key)) corrupt('import repeats a change record');
                seenChangeKeys.add(key);
                changes.push(record);
            } else if (line.kind === 'event') {
                const record = line.record;
                requireStoredEvent(record);
                const taskId = line.task_id;
                const runId = line.run_id;
                const generation = line.generation;
                const sequence = line.sequence;
                if (
                    typeof taskId !== 'string' ||
                    !TASK_ID_PATTERN.test(taskId) ||
                    typeof runId !== 'string' ||
                    !RUN_ID_PATTERN.test(runId) ||
                    !isPositiveInteger(generation) ||
                    !isNonNegativeInteger(sequence)
                ) {
                    corrupt('import event key is invalid');
                }
                const key = `${taskId}\u0000${runId}\u0000${generation}\u0000${sequence}`;
                if (seenEventKeys.has(key)) corrupt('import repeats an event key');
                seenEventKeys.add(key);
                events.push({
                    record,
                    key: [taskId, runId, generation, sequence],
                });
            } else {
                invalidArgument(`unknown import record kind ${String(line.kind)}`);
            }
        }

        const taskIds = new Set<string>();
        for (const task of tasks) {
            if (taskIds.has(task.task_id)) corrupt('import repeats a task record');
            taskIds.add(task.task_id);
        }
        const runMap = new Map(
            runs.map((run) => [runKey(run.task_id, run.run_id), run]),
        );
        for (const event of events) {
            const [taskId, runId] = event.key as [string, string, number, number];
            if (!runMap.has(runKey(taskId, runId))) {
                corrupt('import event belongs to a missing run record');
            }
        }
        const eventGroups = new Map<string, EventPair[]>();
        for (const event of events) {
            const key = this.pairRunKey(event);
            const group = eventGroups.get(key);
            if (group === undefined) {
                eventGroups.set(key, [event]);
            } else {
                group.push(event);
            }
        }
        const verifiedByRun = new Map<
            string,
            { events: AgentIncomingEvent[]; digests: string[] }
        >();
        for (const [key, run] of runMap) {
            const pairs = eventGroups.get(key) ?? [];
            if (pairs.length === 0) {
                corrupt('import run record has no event records');
            }
            const verified = await this.verifyEventChain(run.task_id, run.run_id, pairs);
            this.assertRunRecordMatches(run, verified);
            verifiedByRun.set(key, verified);
        }
        for (const change of changes) {
            const key = runKey(change.task_id, change.run_id);
            if (!runMap.has(key)) {
                corrupt('import change record belongs to a missing run record');
            }
            const verified = verifiedByRun.get(key);
            const bound = verified?.events.some((event) => {
                const data = event.data as Record<string, unknown>;
                return (
                    event.type === 'changes.snapshot' &&
                    data.scope === 'last_turn' &&
                    data.head_digest === change.head_digest
                );
            });
            if (bound !== true) {
                corrupt('import change record has no matching changes.snapshot event');
            }
        }
        for (const snapshot of snapshots) {
            const pairs = eventGroups.get(runKey(snapshot.task_id, snapshot.run_id)) ?? [];
            if (pairs.length === 0) {
                corrupt('import snapshot has no run event chain');
            }
            const verified = await this.verifyEventChain(
                snapshot.task_id,
                snapshot.run_id,
                pairs,
            );
            const index = verified.events.findIndex(
                (event) =>
                    event.generation === snapshot.generation &&
                    event.sequence === snapshot.last_sequence,
            );
            if (
                index < 0 ||
                verified.events[index]!.event_id !== snapshot.last_event_id ||
                verified.digests[index] !== snapshot.head_digest
            ) {
                corrupt('import snapshot position does not match the event chain');
            }
        }
        for (const task of tasks) {
            const latest = task.latest_run_id ?? null;
            if (latest !== null && !runMap.has(runKey(task.task_id, latest))) {
                corrupt('import task latest_run_id has no run record');
            }
        }

        const counts = await Promise.all(
            (['tasks', 'runs', 'events', 'snapshots', 'changes'] as AgentDbStoreName[]).map(
                (name) => this.countStore(name),
            ),
        );
        if (counts.some((count) => count > 0)) {
            throw new AgentStoreError(
                'duplicate_event',
                'import requires an empty agent store',
            );
        }

        const db = this.requireDb();
        const stores: AgentDbStoreName[] = [
            'tasks',
            'runs',
            'events',
            'snapshots',
            'changes',
        ];
        const transaction = db.transaction(stores, 'readwrite');
        try {
            for (const task of tasks) {
                await requestResult(
                    transaction.objectStore('tasks').put(jsonClean(task), task.task_id),
                );
            }
            for (const run of runs) {
                await requestResult(
                    transaction
                        .objectStore('runs')
                        .put(jsonClean(run), [run.task_id, run.run_id]),
                );
            }
            for (const event of events) {
                await requestResult(
                    transaction
                        .objectStore('events')
                        .put(jsonClean(event.record), event.key),
                );
            }
            for (const snapshot of snapshots) {
                await requestResult(
                    transaction
                        .objectStore('snapshots')
                        .put(jsonClean(snapshot), [snapshot.task_id, snapshot.run_id]),
                );
            }
            for (const change of changes) {
                await requestResult(
                    transaction.objectStore('changes').put(jsonClean(change), [
                        change.task_id,
                        change.run_id,
                        change.head_digest,
                    ]),
                );
            }
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }

        this.close();
        await this.open();
        return {
            tasks: tasks.length,
            runs: runs.length,
            events: events.length,
            snapshots: snapshots.length,
            changes: changes.length,
        };
    }

    private requireOpen(): void {
        if (!this.opened || this.db === null) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'durable task store is not open',
            );
        }
    }

    private requireDb(): IDBDatabase {
        this.requireOpen();
        return this.db!;
    }

    private requireWriter(writerId: string): void {
        if (!isNonEmptyString(writerId) || writerId.length > 128) {
            invalidArgument('writer_id must be a non-empty string');
        }
    }

    private async restore(): Promise<OpenReport> {
        const db = await openAgentDatabase(this.factory);
        this.db = db;
        const checkpoint = this.store.checkpoint();
        this.store.reset();
        try {
            const taskRecords = (await this.readAll<TaskRecord>('tasks', db)).map((record) => {
                requireStoredTask(record);
                return record;
            });
            const runRecords = (await this.readAll<RunRecord>('runs', db)).map((record) => {
                requireStoredRun(record);
                return record;
            });
            const snapshotRecords = (await this.readAll<SnapshotRecord>('snapshots', db)).map(
                (record) => {
                    requireStoredSnapshot(record);
                    return record;
                },
            );
            const eventKeys = await this.readAllKeys('events', db);
            const runKeys = new Set(
                runRecords.map((run) => runKey(run.task_id, run.run_id)),
            );
            const orphanRuns = new Map<string, { taskId: string; runId: string }>();
            for (const key of eventKeys) {
                if (!Array.isArray(key) || key.length !== 4) {
                    corrupt('event store key is not [task_id, run_id, generation, sequence]');
                }
                if (
                    typeof key[0] !== 'string' ||
                    typeof key[1] !== 'string' ||
                    typeof key[2] !== 'number' ||
                    typeof key[3] !== 'number'
                ) {
                    corrupt('event store key fields are invalid');
                }
                const composite = runKey(key[0], key[1]);
                if (!runKeys.has(composite)) {
                    orphanRuns.set(composite, { taskId: key[0], runId: key[1] });
                }
            }
            const runsByTask = new Map<string, RunRecord[]>();
            for (const run of runRecords) {
                const existing = runsByTask.get(run.task_id);
                if (existing === undefined) {
                    runsByTask.set(run.task_id, [run]);
                } else {
                    existing.push(run);
                }
            }
            const snapshotsByRun = new Map<string, SnapshotRecord>();
            for (const snapshot of snapshotRecords) {
                const composite = runKey(snapshot.task_id, snapshot.run_id);
                if (!runKeys.has(composite)) {
                    corrupt('snapshot record exists without a run record');
                }
                snapshotsByRun.set(composite, snapshot);
            }
            taskRecords.sort((left, right) =>
                left.created_at_ms === right.created_at_ms
                    ? left.task_id.localeCompare(right.task_id)
                    : left.created_at_ms - right.created_at_ms,
            );
            for (const task of taskRecords) {
                this.store.upsertTask({
                    ...taskToAgentRecord(task),
                    archived: false,
                    pinnedLocal: false,
                    latestRunId: null,
                });
            }
            const incomplete: RestoreIncomplete[] = [];
            const addIncomplete = (entry: RestoreIncomplete) => {
                if (
                    !incomplete.some(
                        (existing) =>
                            existing.task_id === entry.task_id &&
                            existing.run_id === entry.run_id &&
                            existing.code === entry.code,
                    )
                ) {
                    incomplete.push(entry);
                }
            };
            for (const orphan of orphanRuns.values()) {
                addIncomplete({
                    task_id: orphan.taskId,
                    run_id: orphan.runId,
                    code: 'snapshot_missing',
                    message: 'event records exist without a run record',
                });
            }
            const replayedRuns: RunRecord[] = [];
            for (const task of taskRecords) {
                const taskRuns = (runsByTask.get(task.task_id) ?? []).sort(
                    (left, right) =>
                        left.generation - right.generation ||
                        left.started_at_ms - right.started_at_ms ||
                        left.run_id.localeCompare(right.run_id),
                );
                for (const run of taskRuns) {
                    const composite = runKey(run.task_id, run.run_id);
                    const pairs = await this.readEventPairs(run.task_id, run.run_id, db);
                    if (pairs.length === 0) {
                        addIncomplete({
                            task_id: run.task_id,
                            run_id: run.run_id,
                            code: 'snapshot_missing',
                            message: 'run record has no canonical event records',
                        });
                        continue;
                    }
                    await this.verifyAndReplay(
                        run,
                        pairs,
                        snapshotsByRun.get(composite) ?? null,
                    );
                    replayedRuns.push(run);
                }
                const latest = task.latest_run_id ?? null;
                const latestExists =
                    latest !== null && taskRuns.some((run) => run.run_id === latest);
                if (latest !== null && !latestExists) {
                    addIncomplete({
                        task_id: task.task_id,
                        run_id: latest,
                        code: 'snapshot_missing',
                        message: 'latest_run_id has no run record',
                    });
                }
                this.store.upsertTask({
                    ...taskToAgentRecord(task),
                    archived: task.archived,
                    pinnedLocal: task.pinned_local,
                    latestRunId: latestExists ? latest : null,
                });
            }
            const snapshot = this.store.snapshot();
            return {
                tasks: taskRecords.map((task) => {
                    const view = snapshot.tasks.find(
                        (candidate) => candidate.taskId === task.task_id,
                    );
                    return {
                        ...task,
                        latest_run_id: view?.latestRunId ?? null,
                    };
                }),
                runs: replayedRuns,
                incomplete,
            };
        } catch (error) {
            this.store.restore(checkpoint);
            db.close();
            this.db = null;
            throw storageError(error);
        }
    }

    private async verifyAndReplay(
        run: RunRecord,
        pairs: EventPair[],
        snapshot: SnapshotRecord | null,
    ): Promise<void> {
        const verified = await this.verifyEventChain(run.task_id, run.run_id, pairs);
        let snapshotVerified = false;
        for (let index = 0; index < verified.events.length; index += 1) {
            const event = verified.events[index]!;
            const digest = verified.digests[index]!;
            if (event.type === 'run.resumed') {
                if (snapshot === null) {
                    throw new AgentStoreError(
                        'snapshot_missing',
                        'run.resumed requires a persisted verified snapshot',
                    );
                }
                const check = this.store.verifySnapshot(snapshotVerification(snapshot));
                if (!check.ok) {
                    throw new AgentStoreError(
                        check.code ?? 'snapshot_corrupt',
                        check.message ?? 'run.resumed snapshot verification failed',
                    );
                }
            }
            const applied = this.store.apply(event, digest);
            if (applied.status !== 'applied') {
                throw new AgentStoreError(
                    'snapshot_corrupt',
                    `replay rejected ${event.event_id}: ${applied.code ?? 'unknown'} ${applied.message ?? ''}`,
                );
            }
            if (snapshot !== null && snapshotMatchesEvent(snapshot, event, digest)) {
                const check = this.store.verifySnapshot(snapshotVerification(snapshot));
                if (!check.ok) {
                    throw new AgentStoreError(
                        check.code ?? 'snapshot_corrupt',
                        check.message ?? 'snapshot verification failed',
                    );
                }
                snapshotVerified = true;
            }
        }
        if (snapshot !== null && !snapshotVerified) {
            corrupt('snapshot position is not part of the replayed event chain');
        }
        const view = this.store.getRun(run.task_id, run.run_id);
        if (view === null) {
            corrupt('replayed run is missing from AgentStore');
        }
        this.assertRunRecordMatches(run, verified, view);
    }

    private assertRunRecordMatches(
        run: RunRecord,
        verified: { events: AgentIncomingEvent[]; digests: string[] },
        view?: RunView,
    ): void {
        const count = verified.events.length;
        const lastEvent = verified.events[count - 1]!;
        const lastDigest = verified.digests[count - 1]!;
        if (
            run.generation !== lastEvent.generation ||
            run.last_sequence !== lastEvent.sequence ||
            run.last_event_id !== lastEvent.event_id ||
            run.last_digest !== lastDigest
        ) {
            corrupt('run record cursor does not match the verified event chain');
        }
        if (view !== undefined) {
            if (
                run.generation !== view.generation ||
                run.state !== view.state ||
                run.last_sequence !== view.lastSequence ||
                run.last_event_id !== view.lastEventId ||
                run.last_digest !== view.lastDigest
            ) {
                corrupt('run record does not match the replayed run');
            }
        }
    }

    private async verifyEventChain(
        taskId: string,
        runId: string,
        pairs: EventPair[],
    ): Promise<{ events: AgentIncomingEvent[]; digests: string[] }> {
        const events: AgentIncomingEvent[] = [];
        const digests: string[] = [];
        const seen = new Set<string>();
        let previousDigest: string | null = null;
        let expectedSequence = 0;
        let generation: number | null = null;
        for (let index = 0; index < pairs.length; index += 1) {
            const pair = pairs[index]!;
            const key = pair.key;
            if (!Array.isArray(key) || key.length !== 4) {
                corrupt('event store key is not [task_id, run_id, generation, sequence]');
            }
            if (
                typeof key[0] !== 'string' ||
                typeof key[1] !== 'string' ||
                typeof key[2] !== 'number' ||
                typeof key[3] !== 'number'
            ) {
                corrupt('event store key fields are invalid');
            }
            requireStoredEvent(pair.record);
            const event = await this.materializeEvent(pair.record);
            if (event.task_id !== taskId || event.run_id !== runId) {
                corrupt('event identity does not match its per-run key');
            }
            if (event.generation !== key[2] || event.sequence !== key[3]) {
                corrupt('event generation or sequence does not match its key');
            }
            if (seen.has(event.event_id)) {
                corrupt('duplicate event_id in the per-run log');
            }
            seen.add(event.event_id);
            if (events.length === 0) {
                if (pair.record.prev_digest !== null) {
                    corrupt('first event must have a null prev_digest');
                }
                if (event.sequence !== 0 || event.type !== 'run.queued') {
                    corrupt('first event must be run.queued at sequence 0');
                }
                generation = event.generation;
            } else {
                if (pair.record.prev_digest !== previousDigest) {
                    corrupt('event prev_digest does not link to the prior event');
                }
                if (event.sequence !== expectedSequence) {
                    corrupt('event sequence is not contiguous');
                }
                if (event.generation !== generation) {
                    if (
                        generation === null ||
                        event.generation !== generation + 1 ||
                        event.type !== 'run.resumed'
                    ) {
                        corrupt('generation changes require run.resumed');
                    }
                    generation = event.generation;
                }
            }
            previousDigest = pair.record.digest;
            expectedSequence = event.sequence + 1;
            events.push(event);
            digests.push(pair.record.digest);
            if (events.length > MAX_EVENTS_PER_RUN) {
                throw new AgentStoreError('limit_exceeded', 'run exceeds max_events_per_run');
            }
        }
        return { events, digests };
    }

    private async materializeEvent(record: EventRecord): Promise<AgentIncomingEvent> {
        if (record.prev_digest !== null && !DIGEST_PATTERN.test(record.prev_digest)) {
            corrupt('event prev_digest is not a SHA-256 digest');
        }
        let canonical: string;
        try {
            canonical = parseCanonicalJson(record.canonical);
        } catch (error) {
            corrupt(`event canonical text is not canonical JSON: ${errorMessage(error)}`);
        }
        if (canonical !== record.canonical) {
            corrupt('event canonical text is not canonical');
        }
        let value: unknown;
        try {
            value = JSON.parse(canonical);
        } catch {
            corrupt('event canonical text could not be materialized');
        }
        const validation = validateAgentEvent(value);
        if (!validation.ok) {
            corrupt(`event violates the contract: ${validation.code} ${validation.message}`);
        }
        const event = validation.event;
        if (record.event_id !== event.event_id) {
            corrupt('event_id does not match the canonical event');
        }
        const digest = await digestEvent(event as AgentEvent);
        if (digest !== record.digest) {
            corrupt('event digest does not match the recomputed canonical digest');
        }
        return event;
    }

    private async commitEvent(
        event: AgentIncomingEvent,
        canonical: string,
        digest: string,
        input: AppendEventStoreInput,
        now: number,
    ): Promise<void> {
        const db = this.requireDb();
        const transaction = db.transaction(
            ['tasks', 'runs', 'events', 'snapshots', 'leases'],
            'readwrite',
        );
        try {
            const tasks = transaction.objectStore('tasks');
            const runs = transaction.objectStore('runs');
            const events = transaction.objectStore('events');
            const snapshots = transaction.objectStore('snapshots');
            const leases = transaction.objectStore('leases');

            const lease = (await requestResult(leases.get(event.task_id))) as
                | LeaseRecord
                | undefined;
            if (lease !== undefined) {
                requireStoredLease(lease);
                if (
                    lease.writer_id !== input.writer_id &&
                    now - lease.heartbeat_at_ms <= LEASE_STALE_MS
                ) {
                    throw new AgentStoreError(
                        'stale_generation',
                        'task lease is held by another active writer',
                    );
                }
            }

            const existing = (await requestResult(
                events.get([event.task_id, event.run_id, event.generation, event.sequence]),
            )) as EventRecord | undefined;
            if (existing !== undefined && existing.digest !== digest) {
                throw new AgentStoreError(
                    'duplicate_event',
                    'same event key already holds a different digest',
                );
            }
            const storedRun = (await requestResult(
                runs.get([event.task_id, event.run_id]),
            )) as RunRecord | undefined;
            if (storedRun !== undefined) {
                if (event.sequence !== storedRun.last_sequence + 1) {
                    throw new AgentStoreError(
                        'out_of_order',
                        'event sequence does not follow the durable run cursor',
                    );
                }
            } else if (event.type !== 'run.queued' || event.sequence !== 0) {
                throw new AgentStoreError(
                    'snapshot_missing',
                    'durable run record is missing for a non-initial event',
                );
            }
            const view = this.store.getRun(event.task_id, event.run_id);
            if (view === null) {
                throw new AgentStoreError('internal', 'applied run is missing from AgentStore');
            }
            const taskView = this.store
                .snapshot()
                .tasks.find((candidate) => candidate.taskId === event.task_id);
            const latestRunId = taskView?.latestRunId ?? event.run_id;
            const runRecord = this.buildRunRecord(view);
            await requestResult(
                events.put(
                    jsonClean({
                        event_id: event.event_id,
                        canonical,
                        digest,
                        prev_digest: storedRun?.last_digest ?? null,
                        recorded_at_ms: now,
                    } satisfies EventRecord),
                    [event.task_id, event.run_id, event.generation, event.sequence],
                ),
            );
            await requestResult(
                runs.put(jsonClean(runRecord), [event.task_id, event.run_id]),
            );
            const task = (await requestResult(tasks.get(event.task_id))) as
                | TaskRecord
                | undefined;
            if (task === undefined) {
                await requestResult(
                    tasks.put(
                        jsonClean({
                            task_id: event.task_id,
                            project_id: event.project_id,
                            title: event.task_id,
                            created_at_ms: event.recorded_at_ms,
                            archived: false,
                            pinned_local: false,
                            latest_run_id: latestRunId,
                        } satisfies TaskRecord),
                        event.task_id,
                    ),
                );
            } else {
                requireStoredTask(task);
                await requestResult(
                    tasks.put(
                        jsonClean({
                            ...task,
                            latest_run_id: latestRunId,
                        } satisfies TaskRecord),
                        event.task_id,
                    ),
                );
            }
            if (input.snapshot === true) {
                const snapshotRecord = this.buildSnapshot(view);
                await requestResult(
                    snapshots.put(jsonClean(snapshotRecord), [event.task_id, event.run_id]),
                );
            }
            if (this.faults.abort_next_append) {
                this.faults.abort_next_append = false;
                transaction.abort();
            }
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    private buildRunRecord(view: RunView): RunRecord {
        return {
            task_id: view.taskId,
            run_id: view.runId,
            generation: view.generation,
            state: view.state,
            retry_of_run_id: view.retryOfRunId,
            started_at_ms: view.startedAtMs ?? view.events[0]?.recordedAtMs ?? 0,
            ...(view.endedAtMs === null ? {} : { ended_at_ms: view.endedAtMs }),
            last_sequence: view.lastSequence,
            last_event_id: view.lastEventId,
            last_digest: view.lastDigest,
        };
    }

    private buildSnapshot(view: RunView): SnapshotRecord {
        if (view.lastEventId === null || view.lastDigest === null) {
            throw new AgentStoreError(
                'snapshot_missing',
                'run has no verified event to snapshot',
            );
        }
        return {
            schema: SNAPSHOT_SCHEMA,
            task_id: view.taskId,
            run_id: view.runId,
            generation: view.generation,
            last_event_id: view.lastEventId,
            last_sequence: view.lastSequence,
            head_digest: view.lastDigest,
            created_at_ms: this.clock(),
        };
    }

    private async persistTaskWithRollback(record: TaskRecord): Promise<void> {
        const checkpoint = this.store.checkpoint();
        try {
            this.store.upsertTask(taskToAgentRecord(record));
            await this.putRecord('tasks', jsonClean(record), record.task_id);
        } catch (error) {
            this.store.restore(checkpoint);
            throw storageError(error);
        }
    }

    private async updateTaskFlag(
        taskId: string,
        mutate: (record: TaskRecord) => void,
    ): Promise<TaskRecord> {
        this.requireOpen();
        if (!TASK_ID_PATTERN.test(taskId)) invalidArgument('task_id is invalid');
        const stored = (await this.readOne<TaskRecord>('tasks', taskId)) as
            | TaskRecord
            | undefined;
        if (stored === undefined) {
            throw new AgentStoreError('task_not_found', 'task record is missing');
        }
        requireStoredTask(stored);
        const view = this.store.snapshot().tasks.find((task) => task.taskId === taskId);
        if (view === undefined) {
            throw new AgentStoreError('task_not_found', 'task is not present in AgentStore');
        }
        const record: TaskRecord = {
            ...jsonClean(stored),
            latest_run_id: view.latestRunId,
        };
        mutate(record);
        const checkpoint = this.store.checkpoint();
        try {
            this.store.upsertTask(taskToAgentRecord(record));
            await this.putRecord('tasks', jsonClean(record), taskId);
        } catch (error) {
            this.store.restore(checkpoint);
            throw storageError(error);
        }
        return record;
    }

    private async putRecord(
        storeName: AgentDbStoreName,
        value: unknown,
        key: IDBValidKey,
    ): Promise<void> {
        const transaction = this.requireDb().transaction(storeName, 'readwrite');
        try {
            await requestResult(
                transaction.objectStore(storeName).put(jsonClean(value), key),
            );
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    private async readOne<T>(
        storeName: AgentDbStoreName,
        key: IDBValidKey,
        db?: IDBDatabase,
    ): Promise<T | undefined> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const record = (await requestResult(
            transaction.objectStore(storeName).get(key),
        )) as T | undefined;
        await transactionDone(transaction);
        return record;
    }

    private async readAll<T>(
        storeName: AgentDbStoreName,
        db?: IDBDatabase,
    ): Promise<T[]> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const records = (await requestResult(
            transaction.objectStore(storeName).getAll(),
        )) as T[];
        await transactionDone(transaction);
        return records;
    }

    private async readAllKeys(
        storeName: AgentDbStoreName,
        db?: IDBDatabase,
    ): Promise<IDBValidKey[]> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const keys = await requestResult(
            transaction.objectStore(storeName).getAllKeys(),
        );
        await transactionDone(transaction);
        return keys;
    }

    private async countStore(storeName: AgentDbStoreName): Promise<number> {
        const database = this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const count = await requestResult(
            transaction.objectStore(storeName).count(),
        );
        await transactionDone(transaction);
        return count;
    }

    private async readRunRecord(taskId: string, runId: string): Promise<RunRecord | null> {
        const record = (await this.readOne<RunRecord>('runs', [taskId, runId])) as
            | RunRecord
            | undefined;
        if (record === undefined) return null;
        requireStoredRun(record);
        return record;
    }

    private async readEventPairs(
        taskId: string,
        runId: string,
        db?: IDBDatabase,
    ): Promise<EventPair[]> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction('events', 'readonly');
        const store = transaction.objectStore('events');
        const range = IDBKeyRange.bound(
            [taskId, runId, 0, 0],
            [taskId, runId, Number.MAX_SAFE_INTEGER, Number.MAX_SAFE_INTEGER],
        );
        const records = (await requestResult(store.getAll(range))) as EventRecord[];
        const keys = await requestResult(store.getAllKeys(range));
        await transactionDone(transaction);
        if (records.length !== keys.length) {
            corrupt('event records and keys have different lengths');
        }
        return records.map((record, index) => ({
            record,
            key: keys[index]!,
        }));
    }

    private async readAllEventPairs(): Promise<EventPair[]> {
        const database = this.requireDb();
        const transaction = database.transaction('events', 'readonly');
        const store = transaction.objectStore('events');
        const records = (await requestResult(store.getAll())) as EventRecord[];
        const keys = await requestResult(store.getAllKeys());
        await transactionDone(transaction);
        if (records.length !== keys.length) {
            corrupt('event records and keys have different lengths');
        }
        return records.map((record, index) => ({
            record,
            key: keys[index]!,
        }));
    }

    private pairRunKey(pair: EventPair): string {
        const key = pair.key;
        if (!Array.isArray(key) || key.length !== 4) {
            corrupt('event store key is not [task_id, run_id, generation, sequence]');
        }
        return runKey(String(key[0]), String(key[1]));
    }

    private comparePairs(left: EventPair, right: EventPair): number {
        const leftKey = left.key as [string, string, number, number];
        const rightKey = right.key as [string, string, number, number];
        return (
            leftKey[0].localeCompare(rightKey[0]) ||
            leftKey[1].localeCompare(rightKey[1]) ||
            leftKey[2] - rightKey[2] ||
            leftKey[3] - rightKey[3]
        );
    }
}
