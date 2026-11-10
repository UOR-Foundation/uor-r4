import { AgentClientError } from './client.js';
import type {
    AgentTransport,
    ApprovalReceipt,
    HostCapabilities,
} from './client.js';
import type { AgentErrorCode } from './generated/agentContract.js';
import { canonicalJson } from './generated/agentCanonical.js';
import {
    HOST_RESULT_SCHEMA,
    sha256Canonical,
    type HostApprovalInput,
    type HostStreamingKind,
    type HostSubscribeOptions,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
    type HostTransportKind,
} from './hostClient.js';

const LOOPBACK_TOKEN_HEADER = 'X-UOR-Desktop-Token';
const LOOPBACK_INTENTS_PATH = '/v1/studio/host/intents';
const LOOPBACK_RESULTS_PATH = '/v1/studio/host/results';
const LOOPBACK_ORIGIN_PATTERN =
    /^http:\/\/(127\.0\.0\.1|localhost|\[::1\])(?::\d+)?$/;
const HOST_ERROR_CODES: ReadonlySet<AgentErrorCode> = new Set([
    'invalid_arguments',
    'invalid_event',
    'malformed_event',
    'truncated_frame',
    'out_of_order',
    'stale_generation',
    'cross_task_event',
    'cursor_expired',
    'approval_forbidden',
    'approval_expired',
    'approval_digest_mismatch',
    'host_unavailable',
    'host_denied',
    'transport_closed',
    'limit_exceeded',
    'internal',
]);

function isPlainObject(value: unknown): value is Record<string, unknown> {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        return false;
    }
    const prototype = Object.getPrototypeOf(value);
    return prototype === Object.prototype || prototype === null;
}

function toClientError(error: unknown): AgentClientError {
    if (error instanceof AgentClientError) return error;
    return new AgentClientError(
        'internal',
        error instanceof Error ? error.message : String(error),
    );
}

function falseCapabilities(
    kind: HostTransportKind,
    streaming: HostStreamingKind,
    hostAvailable: boolean,
): HostTransportCapabilities {
    return {
        kind,
        hostAvailable,
        streaming,
        tree: false,
        changes: false,
        readArtifact: false,
        terminal: false,
        git: false,
        mcp: false,
        approvals: false,
    };
}

function readCapabilities(
    value: unknown,
    kind: HostTransportKind,
    streaming: HostStreamingKind,
): HostTransportCapabilities {
    const record = isPlainObject(value) ? value : {};
    return {
        kind,
        hostAvailable: true,
        streaming,
        tree: record.tree === true,
        changes: record.changes === true,
        readArtifact: record.readArtifact === true,
        terminal: record.terminal === true,
        git: record.git === true,
        mcp: record.mcp === true,
        approvals: record.approvals === true,
    };
}

function isHostResult(value: unknown): value is HostToolResult {
    return (
        isPlainObject(value) &&
        value.schema === HOST_RESULT_SCHEMA
    );
}

function asHostResult(value: unknown): HostToolResult | null {
    if (isHostResult(value)) return value;
    if (isPlainObject(value) && isHostResult(value.result)) {
        return value.result;
    }
    return null;
}

function errorCode(value: unknown): AgentErrorCode {
    if (
        isPlainObject(value) &&
        typeof value.code === 'string' &&
        HOST_ERROR_CODES.has(value.code as AgentErrorCode)
    ) {
        return value.code as AgentErrorCode;
    }
    return 'host_unavailable';
}

abstract class UnavailableHostTransport implements HostTransport {
    abstract readonly kind: HostTransportKind;
    protected abstract readonly message: string;

    async capabilities(): Promise<HostTransportCapabilities> {
        return falseCapabilities(this.kind, 'none', false);
    }

    async invoke(_intent: HostToolIntent): Promise<HostToolResult> {
        throw new AgentClientError('host_unavailable', this.message);
    }

    subscribe(): HostSubscription {
        throw new AgentClientError('host_unavailable', this.message);
    }

    async approve(_input: HostApprovalInput): Promise<ApprovalReceipt> {
        throw new AgentClientError('host_unavailable', this.message);
    }
}

export class NullHostTransport extends UnavailableHostTransport {
    readonly kind = 'null' as const;
    protected readonly message =
        'the local host companion is not connected';
}

export class StaticHostTransport extends UnavailableHostTransport {
    readonly kind = 'static' as const;
    protected readonly message =
        'This action needs the local host companion. Approve it in the local companion.';
}

export interface TauriChannelHandle {
    onmessage: (message: unknown) => void;
}

export interface TauriHostBridge {
    readonly hasChannel: boolean;
    invoke(
        command: string,
        payload?: Record<string, unknown>,
    ): Promise<unknown>;
    channel(onMessage: (message: unknown) => void): TauriChannelHandle;
}

export interface TauriHostTransportOptions {
    bridge?: TauriHostBridge | null;
    cursor?: string | null;
}

function readPoll(value: unknown): {
    cursor: string | null;
    results: HostToolResult[];
} {
    const collect = (values: unknown[]): HostToolResult[] =>
        values
            .map((item) => asHostResult(item))
            .filter((item): item is HostToolResult => item !== null);
    if (Array.isArray(value)) {
        return { cursor: null, results: collect(value) };
    }
    if (isPlainObject(value)) {
        return {
            cursor: typeof value.cursor === 'string' ? value.cursor : null,
            results: Array.isArray(value.results)
                ? collect(value.results)
                : [],
        };
    }
    return { cursor: null, results: [] };
}

export class TauriHostTransport implements HostTransport {
    readonly kind = 'tauri' as const;
    private readonly bridge: TauriHostBridge | null;
    private readonly listeners = new Set<(result: HostToolResult) => void>();
    private cursor: string | null;

    constructor(options: TauriHostTransportOptions = {}) {
        this.bridge = options.bridge ?? null;
        this.cursor = options.cursor ?? null;
    }

    async capabilities(): Promise<HostTransportCapabilities> {
        const bridge = this.requireBridge();
        return readCapabilities(
            await bridge.invoke('agent_host_capabilities'),
            'tauri',
            bridge.hasChannel ? 'channel' : 'poll',
        );
    }

    async invoke(intent: HostToolIntent): Promise<HostToolResult> {
        const bridge = this.requireBridge();
        const invoked = await bridge.invoke('agent_host_invoke', { intent });
        if (bridge.hasChannel) {
            const result = asHostResult(invoked);
            if (result !== null) return result;
            throw new AgentClientError(
                'host_unavailable',
                'agent_host_invoke did not return a host result',
            );
        }
        const polled = readPoll(
            await bridge.invoke('agent_host_poll', {
                cursor: this.cursor,
            }),
        );
        if (polled.cursor !== null) this.cursor = polled.cursor;
        for (const result of polled.results) this.publish(result);
        const match = polled.results.find(
            (result) => result.intent_id === intent.intent_id,
        );
        if (match === undefined) {
            throw new AgentClientError(
                'host_unavailable',
                'agent_host_poll returned no result for the intent',
            );
        }
        return match;
    }

    subscribe(
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions = {},
    ): HostSubscription {
        const bridge = this.requireBridge();
        if (!bridge.hasChannel) {
            this.listeners.add(listener);
            return {
                close: () => {
                    this.listeners.delete(listener);
                },
                cursor: () => this.cursor,
            };
        }
        const channel = bridge.channel((message) => {
            const result = asHostResult(message);
            if (result !== null) listener(result);
        });
        void bridge
            .invoke('agent_subscribe', { channel })
            .catch((error: unknown) => {
                options.onError?.(toClientError(error));
            });
        return {
            close: () => {
                this.listeners.delete(listener);
            },
            cursor: () => this.cursor,
        };
    }

    async approve(input: HostApprovalInput): Promise<ApprovalReceipt> {
        const bridge = this.requireBridge();
        const response = await bridge.invoke('agent_host_resolve_approval', {
            approval_id: input.approvalId,
            decision: input.decision,
        });
        if (!isPlainObject(response)) {
            throw new AgentClientError(
                'invalid_event',
                'host approval response must be an object',
            );
        }
        if (
            response.approval_id !== input.approvalId ||
            response.decision !== input.decision
        ) {
            throw new AgentClientError(
                'approval_forbidden',
                'host approval response does not match the page decision',
            );
        }
        return {
            approvalId: input.approvalId,
            decision: input.decision,
        };
    }

    private requireBridge(): TauriHostBridge {
        if (this.bridge === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the Tauri host bridge is not connected',
            );
        }
        return this.bridge;
    }

    private publish(result: HostToolResult): void {
        for (const listener of this.listeners) listener(result);
    }
}

export interface LoopbackHostTransportOptions {
    baseUrl?: string;
    token: string;
    fetch?: typeof fetch | null;
    cursor?: string | null;
    capabilities?: Partial<HostCapabilities>;
    onResync?: (reason: 'cursor_expired') => void;
}

export class LoopbackHostTransport implements HostTransport {
    readonly kind = 'loopback' as const;
    private readonly baseUrl: string;
    private readonly token: string;
    private readonly fetchImpl: typeof fetch | null;
    private readonly capabilityValues: Partial<HostCapabilities>;
    private readonly onResync:
        | ((reason: 'cursor_expired') => void)
        | undefined;
    private cursor: string | null;

    constructor(options: LoopbackHostTransportOptions) {
        const baseUrl = options.baseUrl ?? '';
        if (baseUrl !== '' && !LOOPBACK_ORIGIN_PATTERN.test(baseUrl)) {
            throw new AgentClientError(
                'invalid_arguments',
                'loopback base URL must be a local http origin',
            );
        }
        if (typeof options.token !== 'string' || options.token.length === 0) {
            throw new AgentClientError(
                'invalid_arguments',
                'loopback transport requires a per-launch token',
            );
        }
        this.baseUrl = baseUrl.replace(/\/+$/, '');
        this.token = options.token;
        this.fetchImpl = options.fetch ?? null;
        this.capabilityValues = options.capabilities ?? {};
        this.onResync = options.onResync;
        this.cursor = options.cursor ?? null;
    }

    async capabilities(): Promise<HostTransportCapabilities> {
        const available = this.fetchImpl !== null;
        return {
            kind: 'loopback',
            hostAvailable: available,
            streaming: 'ndjson',
            tree: this.capabilityValues.tree === true,
            changes: this.capabilityValues.changes === true,
            readArtifact: this.capabilityValues.readArtifact === true,
            terminal: this.capabilityValues.terminal === true,
            git: this.capabilityValues.git === true,
            mcp: this.capabilityValues.mcp === true,
            approvals: this.capabilityValues.approvals === true,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostToolResult> {
        const response = await this.requireFetch()(this.url(LOOPBACK_INTENTS_PATH), {
            method: 'POST',
            headers: this.headers('application/json'),
            body: JSON.stringify(intent),
            credentials: 'omit',
            cache: 'no-store',
        });
        await this.ensureOk(response);
        const result = asHostResult(await this.safeJson(response));
        if (result === null) {
            throw new AgentClientError(
                'invalid_event',
                'loopback host response is not a HostToolResult',
            );
        }
        return result;
    }

    subscribe(
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions = {},
    ): HostSubscription {
        const fetchImpl = this.fetchImpl;
        if (fetchImpl === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the loopback host is not connected',
            );
        }
        const controller =
            typeof AbortController === 'function'
                ? new AbortController()
                : null;
        let closed = false;
        const cursor = options.cursor ?? this.cursor;
        void this.consume(fetchImpl, listener, cursor, controller).catch(
            (error: unknown) => {
                if (closed) return;
                const clientError = toClientError(error);
                if (clientError.code === 'cursor_expired') {
                    options.onResync?.('cursor_expired');
                    this.onResync?.('cursor_expired');
                    return;
                }
                options.onError?.(clientError);
            },
        );
        return {
            close: () => {
                closed = true;
                controller?.abort();
            },
            cursor: () => this.cursor,
        };
    }

    async approve(_input: HostApprovalInput): Promise<ApprovalReceipt> {
        throw new AgentClientError(
            'host_unavailable',
            'loopback host approval is not enabled in this stage',
        );
    }

    private async consume(
        fetchImpl: typeof fetch,
        listener: (result: HostToolResult) => void,
        cursor: string | null,
        controller: AbortController | null,
    ): Promise<void> {
        const query =
            cursor === null ? '' : `?cursor=${encodeURIComponent(cursor)}`;
        const response = await fetchImpl(
            this.url(`${LOOPBACK_RESULTS_PATH}${query}`),
            {
                method: 'GET',
                headers: this.headers('application/x-ndjson'),
                credentials: 'omit',
                cache: 'no-store',
                signal: controller?.signal,
            },
        );
        await this.ensureOk(response);
        const headerCursor = response.headers.get('x-uor-host-cursor');
        if (headerCursor !== null && headerCursor.length > 0) {
            this.cursor = headerCursor;
        }
        if (response.body === null) {
            throw new AgentClientError(
                'transport_closed',
                'loopback host result stream has no body',
            );
        }
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';
        for (;;) {
            const read = await reader.read();
            if (read.done) break;
            buffer += decoder.decode(read.value, { stream: true });
            const lines = buffer.split('\n');
            buffer = lines.pop() ?? '';
            for (const line of lines) {
                const trimmed = line.trim();
                if (trimmed.length === 0) continue;
                listener(this.parseLine(trimmed));
            }
        }
        buffer += decoder.decode();
        const trailing = buffer.trim();
        if (trailing.length > 0) listener(this.parseLine(trailing));
    }

    private parseLine(line: string): HostToolResult {
        let parsed: unknown;
        try {
            parsed = JSON.parse(line);
        } catch {
            throw new AgentClientError(
                'truncated_frame',
                'loopback host stream line is not complete JSON',
            );
        }
        if (isPlainObject(parsed) && parsed.code === 'cursor_expired') {
            throw new AgentClientError(
                'cursor_expired',
                'host cursor expired; a snapshot resync is required',
            );
        }
        const result = asHostResult(parsed);
        if (result === null) {
            throw new AgentClientError(
                'invalid_event',
                'loopback host stream carried a non-HostToolResult frame',
            );
        }
        if (typeof result.cursor === 'string') {
            this.cursor = result.cursor;
        }
        return result;
    }

    private async ensureOk(response: Response): Promise<void> {
        if (response.status === 410) {
            throw new AgentClientError(
                'cursor_expired',
                'host cursor expired; a snapshot resync is required',
            );
        }
        if (response.ok) return;
        const body = await this.safeJson(response);
        const code = errorCode(body);
        if (code === 'cursor_expired') {
            throw new AgentClientError(
                'cursor_expired',
                'host cursor expired; a snapshot resync is required',
            );
        }
        throw new AgentClientError(
            response.status === 403 ? 'host_denied' : code,
            'loopback host request was refused',
        );
    }

    private async safeJson(response: Response): Promise<unknown> {
        try {
            return (await response.json()) as unknown;
        } catch {
            return null;
        }
    }

    private requireFetch(): typeof fetch {
        if (this.fetchImpl === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the loopback host is not connected',
            );
        }
        return this.fetchImpl;
    }

    private headers(accept: string): Record<string, string> {
        return {
            accept,
            [LOOPBACK_TOKEN_HEADER]: this.token,
            ...(accept === 'application/json'
                ? { 'content-type': 'application/json' }
                : {}),
        };
    }

    private url(path: string): string {
        return `${this.baseUrl}${path}`;
    }
}

export class AgentTransportHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;

    constructor(private readonly transport: AgentTransport) {}

    async capabilities(): Promise<HostTransportCapabilities> {
        const capabilities = await this.transport.hostCapabilities();
        return {
            kind: 'bridge',
            hostAvailable:
                capabilities.tree ||
                capabilities.changes ||
                capabilities.readArtifact ||
                capabilities.terminal ||
                capabilities.git ||
                capabilities.mcp ||
                capabilities.approvals,
            streaming: 'none',
            ...capabilities,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostToolResult> {
        const args = intent.arguments;
        if (intent.tool === 'file.tree') {
            const entries = await this.transport.tree({
                source: 'host',
                ...(typeof args.path === 'string' ? { path: args.path } : {}),
            });
            return hostResultFor(intent, {
                entries: entries.map((entry) => ({
                    path: entry.path,
                    kind: entry.kind,
                    ...(entry.sizeBytes === undefined
                        ? {}
                        : { size_bytes: entry.sizeBytes }),
                })),
            });
        }
        if (intent.tool === 'git.changes') {
            const snapshot = await this.transport.changesSnapshot({
                taskId: intent.task_id,
                runId: intent.run_id,
                scope: 'branch',
            });
            return hostResultFor(intent, {
                snapshot:
                    snapshot === null
                        ? null
                        : {
                              scope: snapshot.scope,
                              files: snapshot.files,
                              head_digest: snapshot.headDigest,
                              captured_at_ms: snapshot.capturedAtMs,
                          },
            });
        }
        if (intent.tool === 'artifact.read') {
            const content = await this.transport.readArtifact({
                taskId: intent.task_id,
                ...(typeof args.path === 'string' ? { path: args.path } : {}),
                ...(typeof args.artifact_id === 'string'
                    ? { artifactId: args.artifact_id }
                    : {}),
            });
            return hostResultFor(intent, {
                path: content.path,
                media_type: content.mediaType,
                size_bytes: content.sizeBytes,
                digest: content.digest,
                bytes_base64: base64(content.bytes),
            });
        }
        throw new AgentClientError(
            'host_unavailable',
            `the host bridge does not execute ${intent.tool}`,
        );
    }

    subscribe(): HostSubscription {
        throw new AgentClientError(
            'host_unavailable',
            'an AgentTransport carries AgentEvents, not host results',
        );
    }

    async approve(input: HostApprovalInput): Promise<ApprovalReceipt> {
        return this.transport.resolveApproval({
            approvalId: input.approvalId,
            decision: input.decision,
        });
    }
}

async function hostResultFor(
    intent: HostToolIntent,
    output: Record<string, unknown>,
    status: 'ok' | 'error' = 'ok',
): Promise<HostToolResult> {
    if (intent.approval_id !== undefined) {
        throw new AgentClientError(
            'approval_forbidden',
            'the fixture host bridge cannot issue host approval receipts',
        );
    }
    let canonical: string;
    try {
        canonical = canonicalJson(output);
    } catch (error) {
        throw new AgentClientError(
            'invalid_arguments',
            `host output is not canonical: ${
                error instanceof Error ? error.message : String(error)
            }`,
        );
    }
    return {
        schema: HOST_RESULT_SCHEMA,
        intent_id: intent.intent_id,
        task_id: intent.task_id,
        run_id: intent.run_id,
        generation: intent.generation,
        tool_call_id: intent.tool_call_id,
        status,
        output: JSON.parse(canonical) as unknown,
        output_digest: await sha256Canonical(output),
        output_preview: canonical.slice(0, 512),
        occurred_at_ms: Date.now(),
    };
}

function base64(bytes: Uint8Array): string {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return globalThis.btoa(binary);
}
