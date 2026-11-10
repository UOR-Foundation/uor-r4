import { canonicalJson } from './generated/agentCanonical.js';
import { AgentClientError } from './client.js';
import type { ApprovalReceipt, HostCapabilities } from './client.js';

export const HOST_INTENT_SCHEMA = 'uor-r4.studio.host-intent/1' as const;
export const HOST_RESULT_SCHEMA = 'uor-r4.studio.host-result/1' as const;

const DIGEST_PATTERN = /^sha256:[0-9a-f]{64}$/;
const RECEIPT_PATTERN = /^hmac-sha256:[0-9a-f]{64}$/;
const TOOL_PATTERN = /^[a-z][a-z0-9_.-]*$/;
const TASK_ID_PATTERN = /^tsk_[0-9a-f]{16}$/;
const RUN_ID_PATTERN = /^run_[0-9a-f]{16}$/;
const APPROVAL_ID_PATTERN = /^[A-Za-z0-9_-]+$/;
const MAX_PREVIEW_LENGTH = 512;
const DATA_TASK_ID = 'tsk_0000000000000000';
const DATA_RUN_ID = 'run_0000000000000000';
const DATA_GENERATION = 1;
const AUTHORITY_KEYS = [
    'granted',
    'scope',
    'command',
    'key',
    'receipt',
    'path',
] as const;

export type HostToolStatus = 'ok' | 'error' | 'cancelled';
export type HostTransportKind =
    | 'null'
    | 'static'
    | 'tauri'
    | 'loopback'
    | 'bridge';
export type HostStreamingKind = 'none' | 'channel' | 'poll' | 'ndjson';
export type HostDataTool = 'file.tree' | 'git.changes' | 'artifact.read';

export interface HostArtifactRef {
    artifact_id: string;
    digest: string;
    media_type: string;
    size_bytes: number;
}

export interface HostToolIntent {
    schema: typeof HOST_INTENT_SCHEMA;
    intent_id: string;
    task_id: string;
    run_id: string;
    generation: number;
    tool_call_id: string;
    tool: string;
    arguments: Record<string, unknown>;
    arguments_digest: string;
    approval_id?: string;
}

export interface HostToolResult {
    schema: typeof HOST_RESULT_SCHEMA;
    intent_id: string;
    task_id: string;
    run_id: string;
    generation: number;
    tool_call_id: string;
    status: HostToolStatus;
    output?: unknown;
    output_digest: string;
    output_preview: string;
    artifact?: HostArtifactRef;
    approval_receipt?: string;
    occurred_at_ms: number;
    code?: string;
    message?: string;
    truncated?: boolean;
    cursor?: string;
}

export interface HostToolRequest {
    taskId: string;
    runId: string;
    generation: number;
    toolCallId: string;
    tool: string;
    arguments: Record<string, unknown>;
    approvalId?: string;
}

export interface HostDataRequest {
    tool: HostDataTool;
    arguments: Record<string, unknown>;
    taskId?: string;
    runId?: string;
    generation?: number;
}

export interface HostApprovalInput {
    approvalId: string;
    decision: 'approved' | 'denied';
}

export interface HostSubscription {
    close(): void;
    cursor(): string | null;
}

export interface HostSubscribeOptions {
    cursor?: string | null;
    onResync?: (reason: 'cursor_expired') => void;
    onError?: (error: AgentClientError) => void;
}

export interface HostTransportCapabilities extends HostCapabilities {
    kind: HostTransportKind;
    hostAvailable: boolean;
    streaming: HostStreamingKind;
}

export interface HostTransport {
    readonly kind: HostTransportKind;
    capabilities(): Promise<HostTransportCapabilities>;
    invoke(intent: HostToolIntent): Promise<HostToolResult>;
    subscribe(
        listener: (result: HostToolResult) => void,
        options?: HostSubscribeOptions,
    ): HostSubscription;
    approve(input: HostApprovalInput): Promise<ApprovalReceipt>;
}

export interface HostClientOptions {
    transport: HostTransport;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        return false;
    }
    const prototype = Object.getPrototypeOf(value);
    return prototype === Object.prototype || prototype === null;
}

function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
}

function invalidArguments(message: string): never {
    throw new AgentClientError('invalid_arguments', message);
}

function invalidEvent(message: string): never {
    throw new AgentClientError('invalid_event', message);
}

export function canonicalToolArguments(
    value: unknown,
    label = 'tool arguments',
): Record<string, unknown> {
    if (!isPlainObject(value)) {
        return invalidArguments(`${label} must be a canonical JSON object`);
    }
    let canonical: string;
    try {
        canonical = canonicalJson(value);
    } catch (error) {
        return invalidArguments(`${label} are not canonical: ${errorMessage(error)}`);
    }
    return JSON.parse(canonical) as Record<string, unknown>;
}

export async function sha256Canonical(value: unknown): Promise<string> {
    const canonical = canonicalJson(value);
    const subtle = globalThis.crypto?.subtle;
    if (subtle === undefined) {
        throw new AgentClientError(
            'runtime_unavailable',
            'Web Crypto SHA-256 is not available in this environment',
        );
    }
    const digest = new Uint8Array(
        await subtle.digest('SHA-256', new TextEncoder().encode(canonical)),
    );
    let hex = '';
    for (const byte of digest) hex += byte.toString(16).padStart(2, '0');
    return `sha256:${hex}`;
}

async function argumentsDigest(value: Record<string, unknown>): Promise<string> {
    try {
        return await sha256Canonical(value);
    } catch (error) {
        if (error instanceof AgentClientError) throw error;
        return invalidArguments(
            `tool arguments are not canonical: ${errorMessage(error)}`,
        );
    }
}

function assertNoAuthorityFields(
    value: Record<string, unknown>,
    label: string,
): void {
    for (const key of AUTHORITY_KEYS) {
        if (key in value) {
            invalidArguments(
                `${label} may not carry host authority field ${key}`,
            );
        }
    }
}

export async function validateHostToolIntent(
    input: HostToolIntent,
): Promise<HostToolIntent> {
    if (!isPlainObject(input)) {
        return invalidArguments('host tool intent must be an object');
    }
    assertNoAuthorityFields(input, 'host tool intent');
    if (input.schema !== HOST_INTENT_SCHEMA) {
        return invalidArguments('host tool intent schema is not host-intent/1');
    }
    if (typeof input.intent_id !== 'string' || input.intent_id.length === 0) {
        return invalidArguments('host tool intent requires an intent_id');
    }
    if (
        typeof input.task_id !== 'string' ||
        !TASK_ID_PATTERN.test(input.task_id)
    ) {
        return invalidArguments('host tool intent requires a canonical task_id');
    }
    if (
        typeof input.run_id !== 'string' ||
        !RUN_ID_PATTERN.test(input.run_id)
    ) {
        return invalidArguments('host tool intent requires a canonical run_id');
    }
    if (
        !Number.isSafeInteger(input.generation) ||
        input.generation < 1
    ) {
        return invalidArguments(
            'host tool intent generation must be a positive safe integer',
        );
    }
    if (
        typeof input.tool_call_id !== 'string' ||
        input.tool_call_id.length === 0
    ) {
        return invalidArguments('host tool intent requires a tool_call_id');
    }
    if (typeof input.tool !== 'string' || !TOOL_PATTERN.test(input.tool)) {
        return invalidArguments('host tool intent requires a canonical tool name');
    }
    const argumentsValue = canonicalToolArguments(input.arguments);
    if (
        typeof input.arguments_digest !== 'string' ||
        !DIGEST_PATTERN.test(input.arguments_digest)
    ) {
        return invalidArguments(
            'host tool intent arguments_digest must be sha256:<64 lowercase hex>',
        );
    }
    if ((await argumentsDigest(argumentsValue)) !== input.arguments_digest) {
        return invalidArguments(
            'host tool intent arguments_digest does not match the arguments',
        );
    }
    if (
        input.approval_id !== undefined &&
        (typeof input.approval_id !== 'string' ||
            !APPROVAL_ID_PATTERN.test(input.approval_id))
    ) {
        return invalidArguments(
            'host tool intent approval_id must be a non-empty identity',
        );
    }
    try {
        canonicalJson({ ...input, arguments: argumentsValue });
    } catch (error) {
        return invalidArguments(
            `host tool intent is not canonical: ${errorMessage(error)}`,
        );
    }
    return { ...input, arguments: argumentsValue };
}

export async function validateHostToolResult(
    intent: HostToolIntent,
    value: unknown,
): Promise<HostToolResult> {
    if (!isPlainObject(value)) {
        return invalidEvent('host tool result must be an object');
    }
    if (
        'type' in value ||
        'event_id' in value ||
        'sequence' in value ||
        'recorded_at_ms' in value
    ) {
        return invalidEvent(
            'host tool result is not an AgentEvent and may not carry event fields',
        );
    }
    if (value.schema !== HOST_RESULT_SCHEMA) {
        return invalidEvent('host tool result schema is not host-result/1');
    }
    if (value.intent_id !== intent.intent_id) {
        throw new AgentClientError(
            'out_of_order',
            'host tool result does not match the intent_id',
        );
    }
    if (value.task_id !== intent.task_id) {
        throw new AgentClientError(
            'cross_task_event',
            'host tool result belongs to a different task',
        );
    }
    if (value.run_id !== intent.run_id) {
        throw new AgentClientError(
            'out_of_order',
            'host tool result belongs to a different run',
        );
    }
    if (value.generation !== intent.generation) {
        throw new AgentClientError(
            'stale_generation',
            'host tool result belongs to a stale generation',
        );
    }
    if (value.tool_call_id !== intent.tool_call_id) {
        throw new AgentClientError(
            'out_of_order',
            'host tool result does not match the tool_call_id',
        );
    }
    if (
        value.status !== 'ok' &&
        value.status !== 'error' &&
        value.status !== 'cancelled'
    ) {
        return invalidEvent('host tool result status is invalid');
    }
    if (
        typeof value.occurred_at_ms !== 'number' ||
        !Number.isSafeInteger(value.occurred_at_ms) ||
        value.occurred_at_ms < 0
    ) {
        return invalidEvent(
            'host tool result occurred_at_ms must be a non-negative safe integer',
        );
    }
    try {
        canonicalJson(value);
    } catch (error) {
        return invalidArguments(
            `host tool result is not canonical: ${errorMessage(error)}`,
        );
    }
    const output = value.output === undefined ? {} : value.output;
    let digest: string;
    try {
        digest = await sha256Canonical(output);
    } catch (error) {
        return invalidArguments(
            `host tool result output is not canonical: ${errorMessage(error)}`,
        );
    }
    if (value.output_digest !== digest) {
        return invalidArguments(
            'host tool result output_digest does not match the output',
        );
    }
    if (
        typeof value.output_preview !== 'string' ||
        value.output_preview.length > MAX_PREVIEW_LENGTH
    ) {
        return invalidEvent(
            'host tool result output_preview must be a bounded string',
        );
    }
    if (value.code !== undefined && typeof value.code !== 'string') {
        return invalidEvent('host tool result code must be a string');
    }
    if (value.message !== undefined && typeof value.message !== 'string') {
        return invalidEvent('host tool result message must be a string');
    }
    if (value.truncated !== undefined && typeof value.truncated !== 'boolean') {
        return invalidEvent('host tool result truncated must be a boolean');
    }
    if (value.cursor !== undefined && typeof value.cursor !== 'string') {
        return invalidEvent('host tool result cursor must be a string');
    }
    if (value.artifact !== undefined) {
        if (!isPlainObject(value.artifact)) {
            return invalidEvent('host tool result artifact must be an object');
        }
        const artifact = value.artifact;
        if (
            typeof artifact.artifact_id !== 'string' ||
            artifact.artifact_id.length === 0
        ) {
            return invalidEvent(
                'host tool result artifact requires an artifact_id',
            );
        }
        if (
            typeof artifact.digest !== 'string' ||
            !DIGEST_PATTERN.test(artifact.digest)
        ) {
            return invalidEvent(
                'host tool result artifact digest must be sha256:<64 lowercase hex>',
            );
        }
        if (
            typeof artifact.media_type !== 'string' ||
            artifact.media_type.length === 0
        ) {
            return invalidEvent(
                'host tool result artifact requires a media_type',
            );
        }
        if (
            typeof artifact.size_bytes !== 'number' ||
            !Number.isSafeInteger(artifact.size_bytes) ||
            artifact.size_bytes < 0
        ) {
            return invalidEvent(
                'host tool result artifact size_bytes must be a non-negative safe integer',
            );
        }
    }
    if (intent.approval_id !== undefined) {
        if (
            typeof value.approval_receipt !== 'string' ||
            !RECEIPT_PATTERN.test(value.approval_receipt)
        ) {
            throw new AgentClientError(
                'approval_forbidden',
                'host tool result lacks a host approval receipt',
            );
        }
    } else if (value.approval_receipt !== undefined) {
        throw new AgentClientError(
            'approval_forbidden',
            'host tool result carries an approval receipt that was not requested',
        );
    }
    return value as unknown as HostToolResult;
}

function isHostToolIntent(
    value: HostToolIntent | HostToolRequest | HostDataRequest,
): value is HostToolIntent {
    return 'schema' in value;
}

function isHostToolRequest(
    value: HostToolIntent | HostToolRequest | HostDataRequest,
): value is HostToolRequest {
    return 'toolCallId' in value;
}

export class HostClient {
    private readonly transport: HostTransport;
    private readonly pending = new Map<string, HostToolIntent>();
    private intentCounter = 0;

    constructor(options: HostClientOptions) {
        this.transport = options.transport;
    }

    async capabilities(): Promise<HostCapabilities> {
        try {
            const capabilities = await this.transport.capabilities();
            return {
                tree: capabilities.tree === true,
                changes: capabilities.changes === true,
                readArtifact: capabilities.readArtifact === true,
                terminal: capabilities.terminal === true,
                git: capabilities.git === true,
                mcp: capabilities.mcp === true,
                approvals: capabilities.approvals === true,
            };
        } catch (error) {
            if (
                error instanceof AgentClientError &&
                error.code === 'host_unavailable'
            ) {
                return {
                    tree: false,
                    changes: false,
                    readArtifact: false,
                    terminal: false,
                    git: false,
                    mcp: false,
                    approvals: false,
                };
            }
            throw error;
        }
    }

    async createIntent(input: HostToolRequest): Promise<HostToolIntent> {
        if (!isPlainObject(input)) {
            return invalidArguments('host tool request must be an object');
        }
        if (
            typeof input.taskId !== 'string' ||
            typeof input.runId !== 'string' ||
            typeof input.toolCallId !== 'string' ||
            typeof input.tool !== 'string' ||
            !Number.isSafeInteger(input.generation)
        ) {
            return invalidArguments(
                'host tool request requires task, run, generation, tool call and tool',
            );
        }
        const argumentsValue = canonicalToolArguments(input.arguments);
        const digest = await argumentsDigest(argumentsValue);
        const counter = this.intentCounter++;
        const intentId = `hti_${input.runId.replace(/^run_/, '')}_${String(
            counter,
        ).padStart(12, '0')}`;
        return validateHostToolIntent({
            schema: HOST_INTENT_SCHEMA,
            intent_id: intentId,
            task_id: input.taskId,
            run_id: input.runId,
            generation: input.generation,
            tool_call_id: input.toolCallId,
            tool: input.tool,
            arguments: argumentsValue,
            arguments_digest: digest,
            ...(input.approvalId === undefined
                ? {}
                : { approval_id: input.approvalId }),
        });
    }

    async execute(input: HostToolIntent): Promise<HostToolResult>;
    async execute(input: HostToolRequest): Promise<HostToolResult>;
    async execute(input: HostDataRequest): Promise<HostToolResult>;
    async execute(
        input: HostToolIntent | HostToolRequest | HostDataRequest,
    ): Promise<HostToolResult> {
        const intent = isHostToolIntent(input)
            ? await validateHostToolIntent(input)
            : isHostToolRequest(input)
              ? await this.createIntent(input)
              : await this.createDataIntent(input);
        if (this.pending.has(intent.intent_id)) {
            throw new AgentClientError(
                'out_of_order',
                'a host intent with this intent_id is already in flight',
            );
        }
        this.pending.set(intent.intent_id, intent);
        try {
            return await validateHostToolResult(
                intent,
                await this.transport.invoke(intent),
            );
        } finally {
            this.pending.delete(intent.intent_id);
        }
    }

    subscribe(
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions = {},
    ): HostSubscription {
        return this.transport.subscribe((result) => {
            const intent = this.pending.get(result.intent_id);
            if (intent === undefined) return;
            void validateHostToolResult(intent, result)
                .then((validated) => listener(validated))
                .catch((error: unknown) => {
                    options.onError?.(toClientError(error));
                });
        }, options);
    }

    async approve(input: HostApprovalInput): Promise<ApprovalReceipt> {
        if (!isPlainObject(input)) {
            return invalidArguments('page approval input must be an object');
        }
        const keys = Object.keys(input).sort();
        if (
            keys.length !== 2 ||
            keys[0] !== 'approvalId' ||
            keys[1] !== 'decision'
        ) {
            return invalidArguments(
                'page approval accepts only approvalId and decision',
            );
        }
        if (
            typeof input.approvalId !== 'string' ||
            !APPROVAL_ID_PATTERN.test(input.approvalId)
        ) {
            return invalidArguments('approvalId must be a non-empty identity');
        }
        if (input.decision !== 'approved' && input.decision !== 'denied') {
            return invalidArguments(
                'approval decision must be approved | denied',
            );
        }
        return this.transport.approve({
            approvalId: input.approvalId,
            decision: input.decision,
        });
    }

    private async createDataIntent(
        input: HostDataRequest,
    ): Promise<HostToolIntent> {
        if (!isPlainObject(input)) {
            return invalidArguments('host data request must be an object');
        }
        const argumentsValue = canonicalToolArguments(input.arguments);
        const capability =
            input.tool === 'file.tree'
                ? 'tree'
                : input.tool === 'git.changes'
                  ? 'changes'
                  : input.tool === 'artifact.read'
                    ? 'readArtifact'
                    : null;
        if (capability === null) {
            return invalidArguments('host data tool is not supported');
        }
        const capabilities = await this.transport.capabilities();
        if (capabilities[capability] !== true) {
            throw new AgentClientError(
                'host_unavailable',
                `the local host companion does not provide ${input.tool}`,
            );
        }
        const digest = await argumentsDigest(argumentsValue);
        const counter = this.intentCounter++;
        const suffix = String(counter).padStart(12, '0');
        return validateHostToolIntent({
            schema: HOST_INTENT_SCHEMA,
            intent_id: `hti_data_${suffix}`,
            task_id: input.taskId ?? DATA_TASK_ID,
            run_id: input.runId ?? DATA_RUN_ID,
            generation: input.generation ?? DATA_GENERATION,
            tool_call_id: `tc_data_${suffix}`,
            tool: input.tool,
            arguments: argumentsValue,
            arguments_digest: digest,
        });
    }
}

function toClientError(error: unknown): AgentClientError {
    if (error instanceof AgentClientError) return error;
    return new AgentClientError(
        'internal',
        error instanceof Error ? error.message : String(error),
    );
}
