import { describe, expect, test } from 'vitest';

import { AgentClient } from './client.js';
import { FixtureTransport } from './fixtureTransport.js';
import { createRunQueuedEvent } from './fixtures.js';
import { canonicalJson, digestEvent } from './generated/agentCanonical.js';
import type { AgentEvent } from './generated/agentContract.js';
import { createAgentStore, type AgentStore } from '../state/agent.js';
import type { AgentTaskStore } from './store.js';
import type {
    AppendEventStoreInput,
    AppendStoreResult,
    TaskRecord,
} from './storeTypes.js';

class MemoryTaskStore {
    isOpen = false;
    closed = false;
    readonly writer = 'writer-client-test';
    readonly appended: AppendEventStoreInput[] = [];
    readonly tasks: TaskRecord[] = [];

    constructor(private readonly target: AgentStore) {}

    get defaultWriterId(): string {
        return this.writer;
    }

    async open(): Promise<{ tasks: TaskRecord[]; runs: unknown[]; incomplete: unknown[] }> {
        this.isOpen = true;
        return { tasks: [], runs: [], incomplete: [] };
    }

    async appendEvent(input: AppendEventStoreInput): Promise<AppendStoreResult> {
        this.appended.push(input);
        const digest = await digestEvent(input.event as AgentEvent);
        const applied = this.target.apply(input.event, digest);
        return {
            status: applied.status === 'ignored' ? 'ignored' : applied.status,
            code: applied.code,
            message: applied.message,
            event_id: input.event.event_id,
            digest,
            storage: null,
        };
    }

    async writeTaskRecord(record: TaskRecord): Promise<TaskRecord> {
        this.tasks.push(record);
        return record;
    }

    close(): void {
        this.closed = true;
        this.isOpen = false;
    }
}

function taskStoreFor(store: AgentStore): {
    stub: MemoryTaskStore;
    durable: AgentTaskStore;
} {
    const stub = new MemoryTaskStore(store);
    return { stub, durable: stub as unknown as AgentTaskStore };
}

describe('AgentClient durable task-store wiring', () => {
    test('requires open before persistence-backed operations', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });

        await expect(client.listTasks()).rejects.toMatchObject({
            code: 'runtime_unavailable',
        });
        await client.open();
        await expect(client.listTasks()).resolves.toEqual([]);
    });

    test('persists task summaries through the durable adapter', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();

        const task = await client.createTask({ title: 'durable task' });
        expect(stub.tasks).toContainEqual(
            expect.objectContaining({
                task_id: task.taskId,
                title: 'durable task',
                latest_run_id: null,
            }),
        );
        expect(client.store.snapshot().tasks).toHaveLength(1);
    });

    test('ingest uses the durable append path and preserves writer identity', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();

        const event = createRunQueuedEvent({
            projectId: 'prj_4b1d9e0c7a2f6e51',
            taskId: 'tsk_9a3c5e7b1d2f4a60',
            runId: 'run_7f3a2c1b9d4e6a80',
            generation: 1,
            recordedAtMs: 1_794_283_200_000,
        });
        const result = await client.ingestWireEvent(canonicalJson(event));
        expect(result.status).toBe('applied');
        expect(stub.appended).toHaveLength(1);
        expect(stub.appended[0]!.writer_id).toBe(stub.writer);
        expect(client.store.snapshot().runs).toHaveLength(1);
    });

    test('transport events are appended durably before publishing', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();
        const task = await client.createTask({ title: 'stream task' });
        const stream = client.subscribe();
        try {
            await client.startRun({ taskId: task.taskId, prompt: 'hello' });
            const iterator = stream[Symbol.asyncIterator]();
            const first = await iterator.next();
            const second = await iterator.next();
            expect(first.done).toBe(false);
            expect(second.done).toBe(false);
            expect(first.value?.type).toBe('run.queued');
            expect(second.value?.type).toBe('run.started');
            expect(stub.appended.map((entry) => entry.event.type)).toEqual([
                'run.queued',
                'run.started',
            ]);
            for (const entry of stub.appended) {
                expect(entry.writer_id).toBe(stub.writer);
            }
        } finally {
            stream.close();
            client.dispose();
        }
        expect(stub.closed).toBe(true);
    });
});
