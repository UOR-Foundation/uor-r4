import { describe, expect, test, vi } from 'vitest';

import { canonicalJson } from './generated/agentCanonical.js';
import { AgentClient, AgentClientError, type HostCapabilities } from './client.js';
import {
    HOST_INTENT_SCHEMA,
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Canonical,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from './hostClient.js';
import {
    LoopbackHostTransport,
    NullHostTransport,
    StaticHostTransport,
    TauriHostTransport,
    type TauriHostBridge,
    type TauriChannelHandle,
} from './transports.js';
import { AgentRuntime } from './runtime.js';
import { ReplayModelPort } from './modelPort.js';
import { FixtureTransport } from './fixtureTransport.js';
import { createAgentStore } from '../state/agent.js';

const ALL_FALSE: HostCapabilities = {
    tree: false,
    changes: false,
    readArtifact: false,
    terminal: false,
    git: false,
    mcp: false,
    approvals: false,
};

const ALL_CAPS: HostCapabilities = {
    tree: true,
    changes: true,
    readArtifact: true,
    terminal: false,
    git: false,
    mcp: false,
    approvals: true,
};

const TASK_ID = 'tsk_0123456789abcdef';
const RUN_ID = 'run_0123456789abcdef';

class FakeHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly intents: HostToolIntent[] = [];
    readonly approvals: Array<{
        approvalId: string;
        decision: 'approved' | 'denied';
    }> = [];
    capabilityValues: HostCapabilities = ALL_FALSE;
    resultFactory: (intent: HostToolIntent) => Promise<HostToolResult> = (
        intent,
    ) => validResult(intent);

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge',
            hostAvailable: true,
            streaming: 'none',
            ...this.capabilityValues,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostToolResult> {
        this.intents.push(intent);
        return this.resultFactory(intent);
    }

    subscribe(): HostSubscription {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: {
        approvalId: string;
        decision: 'approved' | 'denied';
    }) {
        this.approvals.push(input);
        return { approvalId: input.approvalId, decision: input.decision };
    }
}

async function validIntent(
    overrides: Partial<HostToolIntent> = {},
): Promise<HostToolIntent> {
    const args = overrides.arguments ?? { path: 'src/a.ts' };
    return {
        schema: HOST_INTENT_SCHEMA,
        intent_id: 'hti_0123456789abcdef_000000000001',
        task_id: TASK_ID,
        run_id: RUN_ID,
        generation: 2,
        tool_call_id: 'tc_02',
        tool: 'file.read',
        ...overrides,
        arguments: args,
        arguments_digest:
            overrides.arguments_digest ?? (await sha256Canonical(args)),
    };
}

async function validResult(
    intent: HostToolIntent,
    overrides: Partial<HostToolResult> = {},
): Promise<HostToolResult> {
    const output = overrides.output ?? { content: 'hello' };
    let preview = '';
    let digest = `sha256:${'0'.repeat(64)}`;
    try {
        preview = canonicalJson(output).slice(0, 512);
        digest = await sha256Canonical(output);
    } catch {
        // Hostile fixtures keep their bytes; the validator must reject them.
    }
    return {
        schema: HOST_RESULT_SCHEMA,
        intent_id: intent.intent_id,
        task_id: intent.task_id,
        run_id: intent.run_id,
        generation: intent.generation,
        tool_call_id: intent.tool_call_id,
        status: 'ok',
        output,
        output_digest: digest,
        output_preview: preview,
        occurred_at_ms: 1_800_000_000_000,
        ...(intent.approval_id === undefined
            ? {}
            : { approval_receipt: `hmac-sha256:${'a'.repeat(64)}` }),
        ...overrides,
    };
}

async function until(
    predicate: () => boolean,
    label: string,
    timeoutMs = 2_000,
): Promise<void> {
    const deadline = Date.now() + timeoutMs;
    while (!predicate()) {
        if (Date.now() > deadline) {
            throw new Error(`timed out waiting for ${label}`);
        }
        await new Promise((resolve) => setTimeout(resolve, 5));
    }
}

function flush(): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, 0));
}

describe('HostClient transport matrix', () => {
    test('static_host_is_unavailable', async () => {
        const fetchSpy = vi.fn();
        const originalFetch = globalThis.fetch;
        Object.defineProperty(globalThis, 'fetch', {
            value: fetchSpy,
            configurable: true,
            writable: true,
        });
        try {
            const client = new HostClient({
                transport: new StaticHostTransport(),
            });
            await expect(client.capabilities()).resolves.toEqual(ALL_FALSE);
            await expect(client.execute(await validIntent())).rejects.toMatchObject(
                {
                    name: 'AgentClientError',
                    code: 'host_unavailable',
                },
            );
            await expect(
                client.execute(await validIntent()),
            ).rejects.not.toMatchObject({ code: 'runtime_unavailable' });
            await expect(
                client.approve({
                    approvalId: 'apr_5c2e8a1f9b7d3e40',
                    decision: 'approved',
                }),
            ).rejects.toMatchObject({ code: 'host_unavailable' });
            expect(() => client.subscribe(() => undefined)).toThrowError(
                AgentClientError,
            );
            expect(fetchSpy).not.toHaveBeenCalled();

            const nullClient = new HostClient({
                transport: new NullHostTransport(),
            });
            await expect(
                nullClient.execute(await validIntent()),
            ).rejects.toMatchObject({ code: 'host_unavailable' });
            await expect(nullClient.capabilities()).resolves.toEqual(ALL_FALSE);
        } finally {
            Object.defineProperty(globalThis, 'fetch', {
                value: originalFetch,
                configurable: true,
                writable: true,
            });
        }
    });

    test('tauri_transport_uses_channel_when_available', async () => {
        const commands: string[] = [];
        let channelHandler: ((message: unknown) => void) | null = null;
        let channelCreated = 0;
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command, payload) {
                commands.push(command);
                if (command === 'agent_host_capabilities') return ALL_CAPS;
                if (command === 'agent_host_invoke') {
                    const intent = (payload as { intent: HostToolIntent }).intent;
                    return validResult(intent);
                }
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                channelCreated += 1;
                channelHandler = onMessage;
                return { onmessage: onMessage };
            },
        };
        const transport = new TauriHostTransport({ bridge });
        const intent = await validIntent();
        const result = await transport.invoke(intent);

        expect(result.intent_id).toBe(intent.intent_id);
        expect(commands).toContain('agent_host_invoke');
        expect(commands).not.toContain('agent_host_poll');

        const received: HostToolResult[] = [];
        const subscription = transport.subscribe((value) => received.push(value));
        await flush();

        expect(commands).toContain('agent_subscribe');
        expect(channelCreated).toBe(1);
        expect(channelHandler).not.toBeNull();
        const emit = channelHandler as unknown as (message: unknown) => void;
        emit(await validResult(intent));
        expect(received).toHaveLength(1);
        subscription.close();
    });

    test('tauri_offline_shim_uses_poll', async () => {
        const commands: string[] = [];
        let channelCreated = 0;
        let pollCursor: unknown = 'not-polled';
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command, payload) {
                commands.push(command);
                if (command === 'agent_host_capabilities') return ALL_CAPS;
                if (command === 'agent_host_invoke') {
                    const intent = (payload as { intent: HostToolIntent }).intent;
                    return { queued: true, intent_id: intent.intent_id };
                }
                if (command === 'agent_host_poll') {
                    pollCursor = (payload as { cursor?: string | null }).cursor;
                    const intent = await validIntent();
                    return {
                        cursor: 'c1',
                        results: [await validResult(intent)],
                    };
                }
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(): TauriChannelHandle {
                channelCreated += 1;
                throw new Error('the offline shim has no Channel');
            },
        };
        const transport = new TauriHostTransport({ bridge });
        const intent = await validIntent();
        const result = await transport.invoke(intent);

        expect(result.intent_id).toBe(intent.intent_id);
        expect(commands).toContain('agent_host_poll');
        expect(commands).not.toContain('agent_subscribe');
        expect(pollCursor).toBeNull();
        expect(channelCreated).toBe(0);

        const received: HostToolResult[] = [];
        const subscription = transport.subscribe((value) => received.push(value));
        await flush();
        expect(commands).not.toContain('agent_subscribe');
        expect(channelCreated).toBe(0);
        subscription.close();
    });

    test('loopback_transport_uses_fetch_stream_not_event_source', async () => {
        const calls: Array<{ url: string; init: RequestInit }> = [];
        const intent = await validIntent();
        const result = await validResult(intent);
        const fetchImpl = (async (
            input: RequestInfo | URL,
            init: RequestInit = {},
        ): Promise<Response> => {
            const url = String(input);
            calls.push({ url, init });
            if (url.includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify(result), {
                    status: 200,
                    headers: { 'content-type': 'application/json' },
                });
            }
            const body = `${JSON.stringify(result)}\n${JSON.stringify({
                ...result,
                intent_id: 'hti_0123456789abcdef_000000000002',
            })}\n`;
            const stream = new ReadableStream<Uint8Array>({
                start(controller) {
                    controller.enqueue(new TextEncoder().encode(body));
                    controller.close();
                },
            });
            return new Response(stream, {
                status: 200,
                headers: { 'content-type': 'application/x-ndjson' },
            });
        }) as typeof fetch;

        const eventSources: number[] = [];
        const originalEventSource = (globalThis as { EventSource?: unknown })
            .EventSource;
        (globalThis as { EventSource?: unknown }).EventSource = class {
            constructor() {
                eventSources.push(1);
            }
        };
        try {
            const transport = new LoopbackHostTransport({
                token: 'launch-token',
                fetch: fetchImpl,
                cursor: '7',
            });
            await transport.invoke(intent);

            expect(calls[0]?.url).toContain('/v1/studio/host/intents');
            expect(calls[0]?.init.method).toBe('POST');
            const postHeaders = calls[0]?.init.headers as Record<string, string>;
            expect(postHeaders['X-UOR-Desktop-Token']).toBe('launch-token');
            expect(postHeaders['content-type']).toBe('application/json');
            expect(JSON.parse(String(calls[0]?.init.body))).toMatchObject({
                schema: HOST_INTENT_SCHEMA,
                intent_id: intent.intent_id,
            });

            const received: HostToolResult[] = [];
            const subscription = transport.subscribe((value) =>
                received.push(value),
            );
            await until(() => received.length === 2, 'loopback NDJSON results');

            expect(calls[1]?.url).toContain('/v1/studio/host/results?cursor=7');
            expect(calls[1]?.init.method).toBe('GET');
            const getHeaders = calls[1]?.init.headers as Record<string, string>;
            expect(getHeaders['X-UOR-Desktop-Token']).toBe('launch-token');
            expect(getHeaders.accept).toBe('application/x-ndjson');
            const headerNames = Object.keys(getHeaders).map((name) =>
                name.toLowerCase(),
            );
            expect(headerNames).not.toContain('last-event-id');
            expect(eventSources).toHaveLength(0);
            subscription.close();
        } finally {
            (globalThis as { EventSource?: unknown }).EventSource =
                originalEventSource;
        }
    });

    test('loopback_cursor_expired_requests_snapshot_resync', async () => {
        const fetchImpl = (async () =>
            new Response(JSON.stringify({ code: 'cursor_expired' }), {
                status: 410,
                headers: { 'content-type': 'application/json' },
            })) as typeof fetch;
        const transport = new LoopbackHostTransport({
            token: 'launch-token',
            fetch: fetchImpl,
            cursor: '7',
        });
        const received: HostToolResult[] = [];
        const resyncs: string[] = [];
        const errors: AgentClientError[] = [];
        transport.subscribe((value) => received.push(value), {
            onResync: (reason) => resyncs.push(reason),
            onError: (error) => errors.push(error),
        });
        await until(() => resyncs.length === 1, 'cursor_expired resync');
        expect(resyncs).toEqual(['cursor_expired']);
        expect(received).toHaveLength(0);
        expect(errors).toHaveLength(0);
    });
});

describe('HostClient result and approval authority', () => {
    test('host_result_is_validated_against_intent', async () => {
        const intent = await validIntent();
        const cases: Array<[Partial<HostToolResult>, string]> = [
            [{ intent_id: 'hti_other' }, 'out_of_order'],
            [{ task_id: 'tsk_ffffffffffffffff' }, 'cross_task_event'],
            [{ run_id: 'run_ffffffffffffffff' }, 'out_of_order'],
            [{ generation: intent.generation + 1 }, 'stale_generation'],
            [{ tool_call_id: 'tc_other' }, 'out_of_order'],
            [{ output_digest: `sha256:${'0'.repeat(64)}` }, 'invalid_arguments'],
        ];
        for (const [override, code] of cases) {
            const transport = new FakeHostTransport();
            transport.resultFactory = async (received) => ({
                ...(await validResult(received)),
                ...override,
            });
            const client = new HostClient({ transport });
            await expect(client.execute(intent)).rejects.toMatchObject({ code });
        }

        const eventTransport = new FakeHostTransport();
        eventTransport.resultFactory = async (received) => ({
            ...(await validResult(received)),
            type: 'tool.completed',
            event_id: 'evt_run_0123456789abcdef_2_000000000003',
        });
        await expect(
            new HostClient({ transport: eventTransport }).execute(intent),
        ).rejects.toMatchObject({ code: 'invalid_event' });

        const acceptedTransport = new FakeHostTransport();
        const accepted = await new HostClient({
            transport: acceptedTransport,
        }).execute(intent);
        expect(accepted.intent_id).toBe(intent.intent_id);
        expect(accepted.output).toEqual({ content: 'hello' });
    });

    test('approval_receipt_is_required_for_an_approval_intent', async () => {
        const intent = await validIntent({
            approval_id: 'apr_5c2e8a1f9b7d3e40',
        });
        const missing = new FakeHostTransport();
        missing.resultFactory = async (received) => {
            const value = await validResult(received);
            delete value.approval_receipt;
            return value;
        };
        await expect(
            new HostClient({ transport: missing }).execute(intent),
        ).rejects.toMatchObject({ code: 'approval_forbidden' });

        const malformed = new FakeHostTransport();
        malformed.resultFactory = async (received) => ({
            ...(await validResult(received)),
            approval_receipt: 'page-supplied',
        });
        await expect(
            new HostClient({ transport: malformed }).execute(intent),
        ).rejects.toMatchObject({ code: 'approval_forbidden' });

        const accepted = new FakeHostTransport();
        await expect(
            new HostClient({ transport: accepted }).execute(intent),
        ).resolves.toMatchObject({
            approval_receipt: `hmac-sha256:${'a'.repeat(64)}`,
        });
    });

    test('page_approval_input_is_only_id_and_decision', async () => {
        const transport = new FakeHostTransport();
        const client = new HostClient({ transport });
        const approvalId = 'apr_5c2e8a1f9b7d3e40';
        const authorityFields = [
            { granted: true },
            { path: 'src/a.ts' },
            { scope: 'write' },
            { command: 'git commit' },
            { key: 'secret' },
            { receipt: 'host-receipt' },
        ];
        for (const extra of authorityFields) {
            await expect(
                client.approve({
                    approvalId,
                    decision: 'approved',
                    ...extra,
                } as never),
            ).rejects.toMatchObject({ code: 'invalid_arguments' });
        }
        await expect(
            client.approve({
                approvalId,
                decision: 'maybe',
            } as never),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
        expect(transport.approvals).toHaveLength(0);

        await expect(
            client.approve({ approvalId, decision: 'denied' }),
        ).resolves.toEqual({ approvalId, decision: 'denied' });
        expect(transport.approvals).toEqual([
            { approvalId, decision: 'denied' },
        ]);
    });
});

describe('HostClient canonical tool values', () => {
    test('rejects_non_canonical_tool_arguments', async () => {
        const nonCanonical: Array<Record<string, unknown>> = [
            { path: 'src/a.ts', weight: 1.5 },
            { path: 'src/a.ts', offset: -1 },
            { path: 'src/a.ts', 'café': 'value' },
        ];
        for (const argumentsValue of nonCanonical) {
            const transport = new FakeHostTransport();
            const client = new HostClient({ transport });
            await expect(
                client.execute({
                    taskId: TASK_ID,
                    runId: RUN_ID,
                    generation: 2,
                    toolCallId: 'tc_02',
                    tool: 'file.read',
                    arguments: argumentsValue,
                }),
            ).rejects.toMatchObject({ code: 'invalid_arguments' });
            expect(transport.intents).toHaveLength(0);
        }

        const transport = new FakeHostTransport();
        const client = new HostClient({ transport });
        await expect(
            client.execute(await validIntent({ arguments_digest: `sha256:${'0'.repeat(64)}` })),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
        expect(transport.intents).toHaveLength(0);
    });

    test('rejects_non_canonical_host_results', async () => {
        const nonCanonical: Array<Record<string, unknown>> = [
            { value: 1.5 },
            { value: -1 },
            { 'café': 1 },
        ];
        for (const output of nonCanonical) {
            const transport = new FakeHostTransport();
            transport.resultFactory = async (intent) =>
                validResult(intent, { output });
            const client = new HostClient({ transport });
            await expect(
                client.execute(await validIntent()),
            ).rejects.toMatchObject({ code: 'invalid_arguments' });
        }
    });
});

describe('Runtime host boundary', () => {
    test('runtime_accepts_a_valid_host_result_before_tool_completed', async () => {
        const transport = new FakeHostTransport();
        transport.capabilityValues = ALL_CAPS;
        const client = new HostClient({ transport });
        const model = new ReplayModelPort({
            script: [
                [
                    {
                        type: 'tool-call',
                        toolCallId: 'tc_02',
                        tool: 'file.read',
                        arguments: { path: 'src/a.ts' },
                    },
                    { type: 'finished', reason: 'tool' },
                ],
                [
                    { type: 'text-delta', text: 'read complete' },
                    { type: 'finished', reason: 'stop' },
                ],
            ],
        });
        const runtime = new AgentRuntime({
            store: createAgentStore(),
            modelPort: model,
            hostClient: client,
            now: () => 1_800_000_000_000,
        });
        await runtime.open();
        try {
            const task = await runtime.createTask({ title: 'host result' });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: 'read a file',
            });
            await until(
                () =>
                    runtime.store.snapshot().runs.find(
                        (candidate) =>
                            candidate.taskId === task.taskId &&
                            candidate.runId === run.runId,
                    )?.state === 'finished',
                'host result run finished',
            );
            const view = runtime.store.snapshot().runs.find(
                (candidate) =>
                    candidate.taskId === task.taskId &&
                    candidate.runId === run.runId,
            );
            const types = view?.events.map((event) => event.type) ?? [];
            expect(types).toContain('tool.completed');
            expect(types).not.toContain('artifact.created');
            expect(transport.intents).toHaveLength(1);
            expect(transport.intents[0]).toMatchObject({
                schema: HOST_INTENT_SCHEMA,
                task_id: task.taskId,
                run_id: run.runId,
                generation: run.generation,
                tool_call_id: 'tc_02',
                tool: 'file.read',
            });
        } finally {
            await runtime.dispose();
        }
    });

    test('runtime_rejects_a_cross_task_host_result_before_events', async () => {
        const transport = new FakeHostTransport();
        transport.capabilityValues = ALL_CAPS;
        transport.resultFactory = async (intent) => ({
            ...(await validResult(intent)),
            task_id: 'tsk_ffffffffffffffff',
        });
        const client = new HostClient({ transport });
        const model = new ReplayModelPort({
            script: [
                [
                    {
                        type: 'tool-call',
                        toolCallId: 'tc_02',
                        tool: 'file.read',
                        arguments: { path: 'src/a.ts' },
                    },
                    { type: 'finished', reason: 'tool' },
                ],
                [
                    { type: 'text-delta', text: 'after rejected result' },
                    { type: 'finished', reason: 'stop' },
                ],
            ],
        });
        const runtime = new AgentRuntime({
            store: createAgentStore(),
            modelPort: model,
            hostClient: client,
            now: () => 1_800_000_000_000,
        });
        await runtime.open();
        try {
            const task = await runtime.createTask({ title: 'cross task' });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: 'read a file',
            });
            await until(
                () =>
                    runtime.store.snapshot().runs.find(
                        (candidate) =>
                            candidate.taskId === task.taskId &&
                            candidate.runId === run.runId,
                    )?.state === 'finished',
                'cross-task run finished',
            );
            const view = runtime.store.snapshot().runs.find(
                (candidate) =>
                    candidate.taskId === task.taskId &&
                    candidate.runId === run.runId,
            );
            const types = view?.events.map((event) => event.type) ?? [];
            expect(types).not.toContain('tool.completed');
            expect(types).not.toContain('artifact.created');
            const failed = view?.events.find(
                (event) => event.type === 'tool.failed',
            );
            expect(failed?.data.code).toBe('cross_task_event');
        } finally {
            await runtime.dispose();
        }
    });
});

describe('AgentClient host delegation', () => {
    test('agent_client_routes_host_data_through_host_client', async () => {
        const transport = new FakeHostTransport();
        transport.capabilityValues = ALL_CAPS;
        transport.resultFactory = async (intent) => {
            if (intent.tool === 'file.tree') {
                return validResult(intent, {
                    output: {
                        entries: [
                            { path: 'src', kind: 'directory' },
                            {
                                path: 'src/a.ts',
                                kind: 'file',
                                size_bytes: 3,
                            },
                        ],
                    },
                });
            }
            if (intent.tool === 'git.changes') {
                return validResult(intent, {
                    output: {
                        snapshot: {
                            scope: 'branch',
                            files: [{ path: 'src/a.ts' }],
                            head_digest: `sha256:${'b'.repeat(64)}`,
                            captured_at_ms: 1_800_000_000_000,
                        },
                    },
                });
            }
            throw new Error(`unexpected data tool ${intent.tool}`);
        };
        const client = new AgentClient({
            transport: new FakeAgentTransport(),
            hostClient: new HostClient({ transport }),
        });
        await expect(
            client.tree({ source: 'host', path: 'src' }),
        ).resolves.toEqual([
            { path: 'src', kind: 'directory' },
            { path: 'src/a.ts', kind: 'file', sizeBytes: 3 },
        ]);
        await expect(
            client.changesSnapshot({
                taskId: TASK_ID,
                runId: RUN_ID,
                scope: 'branch',
            }),
        ).resolves.toMatchObject({
            scope: 'branch',
            headDigest: `sha256:${'b'.repeat(64)}`,
        });
        expect(transport.intents.map((intent) => intent.tool)).toEqual([
            'file.tree',
            'git.changes',
        ]);
    });
});

class FakeAgentTransport extends FixtureTransport {
    override async hostCapabilities(): Promise<HostCapabilities> {
        return ALL_FALSE;
    }
}
