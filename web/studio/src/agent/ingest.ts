import {
    AGENT_EVENT_TYPES,
    MAX_EVENT_BYTES,
    type AgentErrorCode,
} from './generated/agentContract.js';
import {
    AgentProtocolError,
    parseAgentEventWire,
    type AgentAppendResult,
    type AgentIncomingEvent,
} from '../state/agent.js';

const KNOWN_EVENT_TYPES: ReadonlySet<string> = new Set(AGENT_EVENT_TYPES);
const RESYNC_CODES: ReadonlySet<AgentErrorCode> = new Set([
    'duplicate_event',
    'out_of_order',
    'stale_generation',
    'cross_task_event',
    'cursor_expired',
    'snapshot_corrupt',
    'snapshot_missing',
]);
const EMPTY = new Uint8Array(0);

export type IngestOutcomeKind =
    | 'applied'
    | 'duplicate'
    | 'rejected'
    | 'unknown';

// Four explicit outcomes for one ingested frame. `unknown` still means the
// frame was applied and preserved by the reducer; it only reports that the
// event type is not a known rendering type, so callers must not project it as
// assistant text, a tool result or a completion.
export class IngestOutcome {
    readonly kind: IngestOutcomeKind;
    readonly event: AgentIncomingEvent | null;
    private readonly errorCode: AgentErrorCode | null;

    private constructor(
        kind: IngestOutcomeKind,
        errorCode: AgentErrorCode | null,
        event: AgentIncomingEvent | null,
    ) {
        this.kind = kind;
        this.errorCode = errorCode;
        this.event = event;
    }

    static applied(event: AgentIncomingEvent | null = null): IngestOutcome {
        return new IngestOutcome('applied', null, event);
    }

    static duplicate(event: AgentIncomingEvent | null = null): IngestOutcome {
        return new IngestOutcome('duplicate', null, event);
    }

    static rejected(code: AgentErrorCode): IngestOutcome {
        return new IngestOutcome('rejected', code, null);
    }

    static unknown(event: AgentIncomingEvent | null = null): IngestOutcome {
        return new IngestOutcome('unknown', null, event);
    }

    code(): AgentErrorCode | null {
        return this.errorCode;
    }

    get requiresResync(): boolean {
        return this.errorCode !== null && RESYNC_CODES.has(this.errorCode);
    }
}

export interface DecodedFrameBatch {
    frames: string[];
    buffer: Uint8Array;
    error: AgentErrorCode | null;
}

function concatBytes(left: Uint8Array, right: Uint8Array): Uint8Array {
    if (left.length === 0) return right.slice();
    if (right.length === 0) return left.slice();
    const combined = new Uint8Array(left.length + right.length);
    combined.set(left, 0);
    combined.set(right, left.length);
    return combined;
}

function decodeLine(line: Uint8Array): string | null {
    try {
        const decoded = new TextDecoder('utf-8', { fatal: true }).decode(line);
        return decoded.endsWith('\r') ? decoded.slice(0, -1) : decoded;
    } catch {
        return null;
    }
}

// Splits NDJSON bytes on complete LF frames and keeps the trailing partial
// frame buffered. A chunk may end inside a multi-byte UTF-8 code point; only
// complete lines are decoded, so the split is invisible to the frame text.
export function decodeFrame(
    bytes: Uint8Array,
    buffer: Uint8Array = EMPTY,
): DecodedFrameBatch {
    const combined = concatBytes(buffer, bytes);
    const frames: string[] = [];
    let start = 0;
    for (let index = 0; index < combined.length; index += 1) {
        if (combined[index] !== 0x0a) continue;
        const line = combined.subarray(start, index);
        if (line.length > MAX_EVENT_BYTES) {
            return {
                frames,
                buffer: EMPTY,
                error: 'limit_exceeded',
            };
        }
        const decoded = decodeLine(line);
        if (decoded === null) {
            return {
                frames,
                buffer: EMPTY,
                error: 'malformed_event',
            };
        }
        frames.push(decoded);
        start = index + 1;
    }
    const rest = combined.subarray(start);
    if (rest.length > MAX_EVENT_BYTES) {
        return { frames, buffer: EMPTY, error: 'limit_exceeded' };
    }
    return { frames, buffer: rest.slice(), error: null };
}

// EOF never applies the buffered partial frame. A clean empty buffer has no
// outcome; any bytes without a terminating LF are truncated input.
export function decodeEof(buffer: Uint8Array): IngestOutcome | null {
    return buffer.length === 0
        ? null
        : IngestOutcome.rejected('truncated_frame');
}

export type IngestSink = (
    event: AgentIncomingEvent,
) => Promise<AgentAppendResult>;

export interface NdjsonIngestOptions {
    sink: IngestSink;
    taskId?: string;
    runId?: string;
    onResync?: (code: AgentErrorCode) => void;
}

function protocolCode(error: unknown): AgentErrorCode {
    if (error instanceof AgentProtocolError) return error.code;
    return 'malformed_event';
}

// One stream worker owns one byte buffer and one bound task/run identity.
// Events go through the caller's existing reducer/durable append sink; this
// class never applies or projects an event itself.
export class NdjsonIngest {
    private readonly sink: IngestSink;
    private readonly taskId: string | undefined;
    private readonly runId: string | undefined;
    private readonly onResync: ((code: AgentErrorCode) => void) | undefined;
    private buffer: Uint8Array = EMPTY;
    private haltCode: AgentErrorCode | null = null;

    constructor(options: NdjsonIngestOptions) {
        this.sink = options.sink;
        this.taskId = options.taskId;
        this.runId = options.runId;
        this.onResync = options.onResync;
    }

    get bufferedBytes(): Uint8Array {
        return this.buffer;
    }

    get halted(): AgentErrorCode | null {
        return this.haltCode;
    }

    async push(bytes: Uint8Array): Promise<IngestOutcome[]> {
        if (this.haltCode !== null) {
            return [IngestOutcome.rejected(this.haltCode)];
        }
        const decoded = decodeFrame(bytes, this.buffer);
        this.buffer = decoded.buffer;
        const outcomes: IngestOutcome[] = [];
        for (const frame of decoded.frames) {
            if (this.haltCode !== null) {
                outcomes.push(IngestOutcome.rejected(this.haltCode));
                continue;
            }
            outcomes.push(await this.ingestFrame(frame));
        }
        if (decoded.error !== null && this.haltCode === null) {
            this.halt(decoded.error);
            outcomes.push(IngestOutcome.rejected(decoded.error));
        }
        return outcomes;
    }

    async eof(): Promise<IngestOutcome | null> {
        if (this.haltCode !== null) return null;
        if (this.buffer.length === 0) return null;
        this.buffer = EMPTY;
        this.halt('truncated_frame');
        return IngestOutcome.rejected('truncated_frame');
    }

    // A 410/cursor_expired response resyncs from a verified snapshot in the
    // runtime. The worker only reports the interruption; it never fabricates
    // a completion event for the expired cursor.
    cursorExpired(): IngestOutcome {
        this.buffer = EMPTY;
        this.halt('cursor_expired');
        return IngestOutcome.rejected('cursor_expired');
    }

    private async ingestFrame(frame: string): Promise<IngestOutcome> {
        if (frame.trim().length === 0) {
            return this.reject('malformed_event');
        }
        let event: AgentIncomingEvent;
        try {
            event = parseAgentEventWire(frame);
        } catch (error) {
            return this.reject(protocolCode(error));
        }
        if (this.taskId !== undefined && event.task_id !== this.taskId) {
            return this.reject('cross_task_event');
        }
        if (this.runId !== undefined && event.run_id !== this.runId) {
            return this.reject('out_of_order');
        }
        let result: AgentAppendResult;
        try {
            result = await this.sink(event);
        } catch (error) {
            return this.reject(protocolCode(error));
        }
        if (result.status === 'ignored') {
            return IngestOutcome.duplicate(event);
        }
        if (result.status === 'rejected') {
            return this.reject(result.code ?? 'invalid_event');
        }
        return KNOWN_EVENT_TYPES.has(event.type)
            ? IngestOutcome.applied(event)
            : IngestOutcome.unknown(event);
    }

    private reject(code: AgentErrorCode): IngestOutcome {
        this.halt(code);
        return IngestOutcome.rejected(code);
    }

    private halt(code: AgentErrorCode): void {
        this.haltCode = code;
        this.buffer = EMPTY;
        if (this.onResync !== undefined && RESYNC_CODES.has(code)) {
            this.onResync(code);
        }
    }
}
