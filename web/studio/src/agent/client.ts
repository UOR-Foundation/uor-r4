import type {
    AgentCursor,
    AgentErrorCode,
    AgentEvent,
    RunState,
} from './generated/agentContract.js';
import {
    createAgentStore,
    parseAgentEventWire,
    type AgentAppendResult,
    type AgentChangeSnapshotView,
    type AgentIncomingEvent,
    type AgentStore,
    type AgentTaskRecord,
} from '../state/agent.js';
import { AgentStoreError } from './storeTypes.js';
import type { AgentTaskStore } from './store.js';

export type { AgentIncomingEvent } from '../state/agent.js';

const DIGEST_PATTERN = /^sha256:[0-9a-f]{64}$/;

export class AgentClientError extends Error {
    readonly code: AgentErrorCode;
    readonly retryable: boolean;
    readonly storage: 'indexeddb' | null;

    constructor(
        code: AgentErrorCode,
        message: string,
        retryable = false,
        storage: 'indexeddb' | null = null,
    ) {
        super(`${code}: ${message}`);
        this.name = 'AgentClientError';
        this.code = code;
        this.retryable = retryable;
        this.storage = storage;
    }
}

export interface ListTasksInput {
    includeArchived?: boolean;
    query?: string;
}

export interface TaskSummary {
    taskId: string;
    projectId: string;
    title: string;
    repo: string | null;
    branch: string | null;
    createdAtMs: number;
    archived: boolean;
    pinned: boolean;
    latestRunId: string | null;
    hold?: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    };
    state:
        | 'idle'
        | 'queued'
        | 'running'
        | 'waiting_for_approval'
        | 'waiting_for_background'
        | 'cancelling'
        | 'archived';
}

export interface CreateTaskInput {
    projectId?: string;
    title: string;
    repo?: string;
    branch?: string;
}

export interface StartRunInput {
    taskId: string;
    prompt: string;
    mode?: 'plan' | 'agent';
}

export interface RunSummary {
    taskId: string;
    runId: string;
    generation: number;
    state: RunState;
    retryOfRunId: string | null;
}

export type MessageDelivery = 'queue' | 'steer' | 'force';

export interface QueueMessageInput {
    taskId: string;
    text: string;
}

export interface SteerMessageInput {
    taskId: string;
    text: string;
}

export interface ForceSendInput {
    taskId: string;
    text: string;
}

export interface MessageReceipt {
    taskId: string;
    runId: string;
    messageId: string;
    delivery: MessageDelivery;
}

export interface PinTaskInput {
    taskId: string;
    pinned: boolean;
}

export interface ArchiveTaskInput {
    taskId: string;
    archived: boolean;
}

export interface StopInput {
    taskId: string;
    runId: string;
}

export interface StopReceipt {
    taskId: string;
    runId: string;
    acknowledged: boolean;
}

export interface RetryRunInput {
    taskId: string;
    runId: string;
}

export interface ResolveApprovalInput {
    approvalId: string;
    decision: 'approved' | 'denied';
}

export interface ApprovalReceipt {
    approvalId: string;
    decision: 'approved' | 'denied';
}

export interface ResumeTaskInput {
    taskId: string;
    runId: string;
    cursor?: AgentCursor;
}

export interface TaskSnapshot {
    taskId: string;
    runId: string;
    generation: number;
    lastSequence: number;
    lastEventId: string;
    headDigest: string;
}

export interface ResumeTaskResult {
    snapshot: TaskSnapshot;
    event: AgentEvent;
}

export interface SubscribeInput {
    taskId?: string;
    runId?: string;
}

export type TreeSource = 'local' | 'host';

export interface TreeInput {
    source?: TreeSource;
    path?: string;
}

export interface TreeEntry {
    path: string;
    kind: 'file' | 'directory';
    sizeBytes?: number;
}

export interface ChangesSnapshotInput {
    taskId: string;
    runId?: string;
    scope: 'last_turn' | 'branch';
}

export interface ChangesSnapshot {
    scope: 'last_turn' | 'branch';
    files: Array<Record<string, unknown>>;
    headDigest: string;
    capturedAtMs: number;
}

export interface ReadArtifactInput {
    taskId?: string;
    artifactId?: string;
    path?: string;
}

export interface ArtifactContent {
    path: string;
    mediaType: string;
    sizeBytes: number;
    digest: string;
    bytes: Uint8Array;
}

export interface HostCapabilities {
    tree: boolean;
    changes: boolean;
    readArtifact: boolean;
    terminal: boolean;
    git: boolean;
    mcp: boolean;
    approvals: boolean;
}

export interface LocalAgentFile {
    path: string;
    mediaType: string;
    bytes: Uint8Array;
    digest: string;
    artifactId?: string;
}

export interface AgentTransport {
    listTasks(input: ListTasksInput): Promise<TaskSummary[]>;
    createTask(input: CreateTaskInput): Promise<TaskSummary>;
    startRun(input: StartRunInput): Promise<RunSummary>;
    queueMessage(input: QueueMessageInput): Promise<MessageReceipt>;
    steer(input: SteerMessageInput): Promise<MessageReceipt>;
    forceSend(input: ForceSendInput): Promise<MessageReceipt>;
    pinTask(input: PinTaskInput): Promise<TaskSummary>;
    archiveTask(input: ArchiveTaskInput): Promise<TaskSummary>;
    stop(input: StopInput): Promise<StopReceipt>;
    retry(input: RetryRunInput): Promise<RunSummary>;
    resolveApproval(input: ResolveApprovalInput): Promise<ApprovalReceipt>;
    resumeTask(input: ResumeTaskInput): Promise<ResumeTaskResult>;
    // digest is the canonical SHA-256 of the event. A transport that cannot
    // supply it must deliver canonical text through ingestWireEvent instead,
    // so the store keeps a verifiable digest chain.
    subscribe(
        listener: (event: AgentIncomingEvent, digest?: string) => void,
        input?: SubscribeInput,
    ): () => void;
    tree(input: TreeInput): Promise<TreeEntry[]>;
    changesSnapshot(
        input: ChangesSnapshotInput,
    ): Promise<ChangesSnapshot | null>;
    readArtifact(input: ReadArtifactInput): Promise<ArtifactContent>;
    hostCapabilities(): Promise<HostCapabilities>;
}

export interface AgentEventStream extends AsyncIterable<AgentIncomingEvent> {
    close(): void;
}

class BufferedAgentEventStream implements AgentEventStream {
    private readonly buffer: AgentIncomingEvent[] = [];
    private readonly waiters: Array<
        (result: IteratorResult<AgentIncomingEvent>) => void
    > = [];
    private closed = false;

    constructor(
        private readonly filter: SubscribeInput,
        private readonly onClose: () => void,
    ) {}

    push(event: AgentIncomingEvent): void {
        if (this.closed) return;
        if (
            this.filter.taskId !== undefined &&
            this.filter.taskId !== event.task_id
        ) {
            return;
        }
        if (
            this.filter.runId !== undefined &&
            this.filter.runId !== event.run_id
        ) {
            return;
        }
        const waiter = this.waiters.shift();
        if (waiter !== undefined) {
            waiter({ value: event, done: false });
            return;
        }
        this.buffer.push(event);
    }

    close(): void {
        if (this.closed) return;
        this.closed = true;
        for (const waiter of this.waiters.splice(0)) {
            waiter({ value: undefined as never, done: true });
        }
        this.onClose();
    }

    next(): Promise<IteratorResult<AgentIncomingEvent>> {
        const buffered = this.buffer.shift();
        if (buffered !== undefined) {
            return Promise.resolve({ value: buffered, done: false });
        }
        if (this.closed) {
            return Promise.resolve({ value: undefined as never, done: true });
        }
        return new Promise((resolve) => {
            this.waiters.push(resolve);
        });
    }

    return(): Promise<IteratorResult<AgentIncomingEvent>> {
        this.close();
        return Promise.resolve({ value: undefined as never, done: true });
    }

    [Symbol.asyncIterator](): AsyncIterator<AgentIncomingEvent> {
        return this;
    }
}

export interface AgentClientOptions {
    transport?: AgentTransport | null;
    store?: AgentStore;
    taskStore?: AgentTaskStore | null;
}

export class AgentClient {
    readonly store: AgentStore;

    private readonly transport: AgentTransport | null;
    private readonly taskStore: AgentTaskStore | null;
    private readonly streams = new Set<BufferedAgentEventStream>();
    private readonly localFiles = new Map<string, LocalAgentFile>();
    private readonly localArtifactIds = new Map<string, string>();
    private readonly unsubscribeTransport: (() => void) | null;
    private durableQueue: Promise<void> = Promise.resolve();
    private persistenceError: AgentClientError | null = null;

    constructor(options: AgentClientOptions = {}) {
        this.transport = options.transport ?? null;
        this.store = options.store ?? createAgentStore();
        this.taskStore = options.taskStore ?? null;
        this.unsubscribeTransport =
            this.transport === null
                ? null
                : this.transport.subscribe((event, digest) => {
                      if (this.taskStore === null) {
                          const result = this.store.apply(event, digest);
                          if (result.status === 'applied') {
                              this.publish(event);
                          }
                          return;
                      }
                      this.durableQueue = this.durableQueue.then(async () => {
                          try {
                              const result = await this.taskStore!.appendEvent({
                                  event,
                                  digest,
                                  writer_id: this.taskStore!.defaultWriterId,
                              });
                              if (result.status === 'applied') {
                                  this.publish(event);
                              } else if (result.status === 'rejected') {
                                  this.persistenceError = new AgentClientError(
                                      result.code ?? 'internal',
                                      result.message ?? 'durable append was rejected',
                                      true,
                                      result.storage,
                                  );
                              }
                          } catch (error) {
                              this.persistenceError = this.toClientError(error);
                          }
                      });
                  });
    }

    get lastPersistenceError(): AgentClientError | null {
        return this.persistenceError;
    }

    // Replays the durable log into the same AgentStore the UI reads. Callers
    // must await this before any persistence-backed operation.
    async open(): Promise<void> {
        if (this.taskStore === null) return;
        try {
            await this.taskStore.open();
        } catch (error) {
            throw this.toClientError(error);
        }
    }

    async listTasks(input: ListTasksInput = {}): Promise<TaskSummary[]> {
        const durable = this.requireReady();
        const tasks = await this.requireTransport().listTasks(input);
        for (const task of tasks) {
            const record = this.toTaskRecord(task);
            if (durable !== null) await this.persistTask(record);
            else this.store.upsertTask(record);
        }
        const snapshot = this.store.snapshot();
        return tasks.map((task) => {
            const view = snapshot.tasks.find(
                (candidate) => candidate.taskId === task.taskId,
            );
            if (view === undefined) {
                return task;
            }
            return {
                ...task,
                state: view.state,
                latestRunId: view.latestRunId,
                archived: view.archived,
                pinned: view.pinnedLocal,
            };
        });
    }

    async createTask(input: CreateTaskInput): Promise<TaskSummary> {
        const durable = this.requireReady();
        const task = await this.requireTransport().createTask(input);
        const record = this.toTaskRecord(task);
        if (durable !== null) await this.persistTask(record);
        else this.store.upsertTask(record);
        return task;
    }

    async startRun(input: StartRunInput): Promise<RunSummary> {
        this.requireReady();
        return this.requireTransport().startRun(input);
    }

    async queueMessage(input: QueueMessageInput): Promise<MessageReceipt> {
        this.requireReady();
        return this.requireTransport().queueMessage(input);
    }

    async steer(input: SteerMessageInput): Promise<MessageReceipt> {
        this.requireReady();
        return this.requireTransport().steer(input);
    }

    async forceSend(input: ForceSendInput): Promise<MessageReceipt> {
        this.requireReady();
        return this.requireTransport().forceSend(input);
    }

    async pinTask(input: PinTaskInput): Promise<TaskSummary> {
        const durable = this.requireReady();
        const task = await this.requireTransport().pinTask(input);
        const record = this.toTaskRecord(task);
        if (durable !== null) await this.persistTask(record);
        else this.store.upsertTask(record);
        return task;
    }

    async archiveTask(input: ArchiveTaskInput): Promise<TaskSummary> {
        const durable = this.requireReady();
        const task = await this.requireTransport().archiveTask(input);
        const record = this.toTaskRecord(task);
        if (durable !== null) await this.persistTask(record);
        else this.store.upsertTask(record);
        return task;
    }

    async stop(input: StopInput): Promise<StopReceipt> {
        this.requireReady();
        return this.requireTransport().stop(input);
    }

    async retry(input: RetryRunInput): Promise<RunSummary> {
        this.requireReady();
        return this.requireTransport().retry(input);
    }

    async resolveApproval(
        input: ResolveApprovalInput,
    ): Promise<ApprovalReceipt> {
        this.requireReady();
        if (input.decision !== 'approved' && input.decision !== 'denied') {
            throw new AgentClientError(
                'invalid_arguments',
                'approval decision must be approved | denied',
            );
        }
        // The page sends exactly these two fields. Granted/path/command/scope
        // or any page-held key never crosses this boundary.
        const payload: ResolveApprovalInput = {
            approvalId: input.approvalId,
            decision: input.decision,
        };
        return this.requireTransport().resolveApproval(payload);
    }

    async resumeTask(input: ResumeTaskInput): Promise<TaskSnapshot> {
        this.requireReady();
        const transport = this.requireTransport();
        const result = await transport.resumeTask(input);
        const verification = this.store.verifySnapshot(result.snapshot);
        if (!verification.ok) {
            throw new AgentClientError(
                verification.code ?? 'snapshot_corrupt',
                verification.message ?? 'snapshot verification failed',
            );
        }
        const appended = await this.appendThroughStore(result.event, undefined, {
            snapshotAnchor: result.snapshot,
        });
        if (appended.status === 'rejected') {
            throw new AgentClientError(
                appended.code ?? 'internal',
                appended.message ?? 'resume event rejected',
            );
        }
        return result.snapshot;
    }

    subscribe(input: SubscribeInput = {}): AgentEventStream {
        this.requireReady();
        this.requireTransport();
        let stream: BufferedAgentEventStream;
        stream = new BufferedAgentEventStream(input, () => {
            this.streams.delete(stream);
        });
        this.streams.add(stream);
        return stream;
    }

    async ingestWireEvent(text: string): Promise<AgentAppendResult> {
        this.requireReady();
        const event = parseAgentEventWire(text);
        return this.appendThroughStore(event);
    }

    registerLocalFile(file: LocalAgentFile): void {
        if (
            file.path.length === 0 ||
            file.path.startsWith('/') ||
            file.path.split('/').includes('..')
        ) {
            throw new AgentClientError(
                'invalid_arguments',
                'local file path must be a relative path without parent segments',
            );
        }
        if (!DIGEST_PATTERN.test(file.digest)) {
            throw new AgentClientError(
                'invalid_arguments',
                'local file digest must be sha256:<64 lowercase hex>',
            );
        }
        this.localFiles.set(file.path, {
            path: file.path,
            mediaType: file.mediaType,
            bytes: new Uint8Array(file.bytes),
            digest: file.digest,
            artifactId: file.artifactId,
        });
        if (file.artifactId !== undefined) {
            this.localArtifactIds.set(file.artifactId, file.path);
        }
    }

    async tree(input: TreeInput = {}): Promise<TreeEntry[]> {
        const source = input.source ?? 'local';
        if (source === 'local') {
            return this.localTree(input.path);
        }
        const transport = this.transport;
        if (transport === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the local host companion is not connected',
            );
        }
        const capabilities = await transport.hostCapabilities();
        if (!capabilities.tree) {
            throw new AgentClientError(
                'host_unavailable',
                'the local host companion does not provide a file tree',
            );
        }
        return transport.tree(input);
    }

    async changesSnapshot(
        input: ChangesSnapshotInput,
    ): Promise<ChangesSnapshot | null> {
        const local: AgentChangeSnapshotView | null =
            this.store.latestChangesSnapshot(
                input.taskId,
                input.runId,
                input.scope,
            );
        if (local !== null) {
            return {
                scope: local.scope,
                files: local.files,
                headDigest: local.headDigest,
                capturedAtMs: local.capturedAtMs,
            };
        }
        if (input.scope === 'last_turn') {
            return null;
        }
        const transport = this.transport;
        if (transport === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the local host companion is not connected',
            );
        }
        const capabilities = await transport.hostCapabilities();
        if (!capabilities.changes) {
            throw new AgentClientError(
                'host_unavailable',
                'the local host companion does not provide branch diffs',
            );
        }
        return transport.changesSnapshot(input);
    }

    async readArtifact(input: ReadArtifactInput): Promise<ArtifactContent> {
        const local = this.localArtifact(input);
        if (local !== null) {
            return local;
        }
        const transport = this.transport;
        if (transport !== null) {
            const capabilities = await transport.hostCapabilities();
            if (capabilities.readArtifact) {
                return transport.readArtifact(input);
            }
        }
        throw new AgentClientError(
            'host_unavailable',
            'the requested artifact is not local and the local host companion is not connected',
        );
    }

    async hostCapabilities(): Promise<HostCapabilities> {
        return this.requireTransport().hostCapabilities();
    }

    dispose(): void {
        for (const stream of [...this.streams]) {
            stream.close();
        }
        this.unsubscribeTransport?.();
        this.taskStore?.close();
    }

    private requireReady(): AgentTaskStore | null {
        if (this.taskStore === null) return null;
        if (!this.taskStore.isOpen) {
            throw new AgentClientError(
                'runtime_unavailable',
                'durable task store is not open; await AgentClient.open() first',
            );
        }
        return this.taskStore;
    }

    private async persistTask(record: AgentTaskRecord): Promise<void> {
        if (this.taskStore === null) return;
        try {
            await this.taskStore.writeTaskRecord({
                task_id: record.taskId,
                project_id: record.projectId,
                title: record.title,
                ...(record.repo === null || record.repo === undefined
                    ? {}
                    : { repo: record.repo }),
                ...(record.branch === null || record.branch === undefined
                    ? {}
                    : { branch: record.branch }),
                created_at_ms: record.createdAtMs,
                archived: record.archived ?? false,
                pinned_local: record.pinnedLocal ?? false,
                latest_run_id: record.latestRunId ?? null,
                ...(record.hold === undefined ? {} : { hold: record.hold }),
            });
        } catch (error) {
            throw this.toClientError(error);
        }
    }

    private async appendThroughStore(
        event: AgentIncomingEvent,
        digest?: string,
        options: { snapshotAnchor?: TaskSnapshot } = {},
    ): Promise<AgentAppendResult> {
        if (this.taskStore !== null) {
            const anchor = options.snapshotAnchor;
            const result = await this.taskStore.appendEvent({
                event,
                digest,
                writer_id: this.taskStore.defaultWriterId,
                ...(anchor === undefined
                    ? {}
                    : {
                          snapshot_anchor: {
                              task_id: anchor.taskId,
                              run_id: anchor.runId,
                              generation: anchor.generation,
                              last_sequence: anchor.lastSequence,
                              last_event_id: anchor.lastEventId,
                              head_digest: anchor.headDigest,
                          },
                      }),
            });
            if (result.status === 'applied') {
                this.publish(event);
            }
            return {
                status: result.status,
                code: result.code,
                message: result.message,
                requiresResync: result.status === 'rejected',
                eventId: result.event_id,
                digest: result.digest,
            };
        }
        const result = await this.store.appendEvent(event);
        if (result.status === 'applied') {
            this.publish(event);
        }
        return result;
    }

    private toClientError(error: unknown): AgentClientError {
        if (error instanceof AgentClientError) return error;
        if (error instanceof AgentStoreError) {
            const prefix = `${error.code}: `;
            return new AgentClientError(
                error.code,
                error.message.startsWith(prefix)
                    ? error.message.slice(prefix.length)
                    : error.message,
                false,
                error.storage,
            );
        }
        return new AgentClientError(
            'internal',
            error instanceof Error ? error.message : String(error),
        );
    }

    private requireTransport(): AgentTransport {
        if (this.transport === null) {
            throw new AgentClientError(
                'runtime_unavailable',
                'the browser task runtime is not initialized',
            );
        }
        return this.transport;
    }

    private publish(event: AgentIncomingEvent): void {
        for (const stream of this.streams) {
            stream.push(event);
        }
    }

    private toTaskRecord(task: TaskSummary): AgentTaskRecord {
        return {
            taskId: task.taskId,
            projectId: task.projectId,
            title: task.title,
            repo: task.repo,
            branch: task.branch,
            createdAtMs: task.createdAtMs,
            archived: task.archived,
            pinnedLocal: task.pinned,
            latestRunId: task.latestRunId,
            ...(task.hold === undefined ? {} : { hold: task.hold }),
        };
    }

    private localTree(path: string | undefined): TreeEntry[] {
        const prefix = (path ?? '').replace(/\/+$/, '');
        const entries = new Map<string, TreeEntry>();
        for (const file of this.localFiles.values()) {
            if (
                prefix.length > 0 &&
                file.path !== prefix &&
                !file.path.startsWith(`${prefix}/`)
            ) {
                continue;
            }
            const segments = file.path.split('/').filter((segment) => segment.length > 0);
            let current = '';
            for (let index = 0; index < segments.length - 1; index += 1) {
                current =
                    current.length === 0
                        ? segments[index]!
                        : `${current}/${segments[index]}`;
                entries.set(current, { path: current, kind: 'directory' });
            }
            entries.set(file.path, {
                path: file.path,
                kind: 'file',
                sizeBytes: file.bytes.byteLength,
            });
        }
        return [...entries.values()].sort((left, right) =>
            left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
        );
    }

    private localArtifact(input: ReadArtifactInput): ArtifactContent | null {
        let path = input.path;
        if (path === undefined && input.artifactId !== undefined) {
            path = this.localArtifactIds.get(input.artifactId);
        }
        if (path === undefined) {
            if (input.path === undefined && input.artifactId === undefined) {
                throw new AgentClientError(
                    'invalid_arguments',
                    'readArtifact requires path or artifactId',
                );
            }
            return null;
        }
        const file = this.localFiles.get(path);
        if (file === undefined) {
            return null;
        }
        return {
            path: file.path,
            mediaType: file.mediaType,
            sizeBytes: file.bytes.byteLength,
            digest: file.digest,
            bytes: new Uint8Array(file.bytes),
        };
    }
}
