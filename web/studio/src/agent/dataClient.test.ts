import { describe, expect, test } from 'vitest';

import { AgentClient } from './client.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Bytes,
    sha256Canonical,
    type HostDataRequest,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from './hostClient.js';

const TASK_ID = 'tsk_0123456789abcdef';
const RUN_ID = 'run_0123456789abcdef';
const ZERO_TASK_ID = 'tsk_0000000000000000';
const ZERO_RUN_ID = 'run_0000000000000000';
const ALL_CAPS: HostTransportCapabilities = {
    kind: 'bridge',
    hostAvailable: true,
    streaming: 'none',
    tree: true,
    changes: true,
    readArtifact: true,
    terminal: false,
    git: false,
    mcp: false,
    approvals: false,
};

function base64(bytes: Uint8Array): string {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return btoa(binary);
}

async function resultFor(
    intent: HostToolIntent,
    output: unknown,
): Promise<HostToolResult> {
    return {
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
        output_preview: '',
        occurred_at_ms: 1_800_000_000_000,
    };
}

class RecordingHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = false;
    readonly intents: HostToolIntent[] = [];
    readonly contexts: Array<HostDataRequest | undefined> = [];

    constructor(
        private readonly result: (
            intent: HostToolIntent,
        ) => Promise<HostToolResult>,
    ) {}

    async capabilities(): Promise<HostTransportCapabilities> {
        return { ...ALL_CAPS };
    }

    async invoke(
        intent: HostToolIntent,
        context?: HostDataRequest,
    ): Promise<HostToolResult> {
        this.intents.push(intent);
        this.contexts.push(context);
        return this.result(intent);
    }

    subscribe() {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: {
        approvalId: string;
        decision: 'approved' | 'denied';
    }) {
        return { approvalId: input.approvalId, decision: input.decision };
    }
}

function clientWith(transport: RecordingHostTransport): AgentClient {
    return new AgentClient({
        hostClient: new HostClient({ transport }),
    });
}

function expectDataIdentity(intent: HostToolIntent): void {
    expect(intent.request_scope).toBe('data');
    expect(intent.task_id).toBe(ZERO_TASK_ID);
    expect(intent.run_id).toBe(ZERO_RUN_ID);
    expect(intent.generation).toBe(1);
}

function expectNoUrlArguments(intent: HostToolIntent): void {
    for (const key of [
        'url',
        'artifact_url',
        'download_url',
        'token',
        'expires_at',
        'link',
    ]) {
        expect(intent.arguments).not.toHaveProperty(key);
    }
}

describe('AgentClient companion data routing', () => {
    test('tree_uses_explicit_data_scope_and_grant_identity', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                grant_id: 'gr_abcd1234',
                path: 'src',
                entries: [
                    { path: 'src', kind: 'directory' },
                    { path: 'src/a.ts', kind: 'file', size_bytes: 5 },
                ],
                truncated: false,
            }),
        );
        const client = clientWith(transport);

        await expect(
            client.tree({
                source: 'host',
                path: 'src',
                grantId: 'gr_abcd1234',
                taskId: TASK_ID,
            }),
        ).resolves.toEqual([
            { path: 'src', kind: 'directory' },
            { path: 'src/a.ts', kind: 'file', sizeBytes: 5 },
        ]);

        expect(transport.intents).toHaveLength(1);
        const intent = transport.intents[0]!;
        expect(intent.tool).toBe('file.tree');
        expectDataIdentity(intent);
        expect(intent.arguments).toEqual({
            grant_id: 'gr_abcd1234',
            path: 'src',
        });
        expectNoUrlArguments(intent);
        expect(transport.contexts[0]?.taskId).toBe(TASK_ID);
        expect(transport.contexts[0]?.runId).toBeUndefined();
    });

    test('tree_without_grant_keeps_the_existing_argument_shape', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, { entries: [] }),
        );
        const client = clientWith(transport);

        await expect(
            client.tree({ source: 'host', path: 'src' }),
        ).resolves.toEqual([]);
        expect(transport.intents[0]!.arguments).toEqual({ path: 'src' });
    });

    test('branch_changes_use_host_but_last_turn_never_does', async () => {
        const headDigest = `sha256:${'a'.repeat(64)}`;
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                snapshot: {
                    scope: 'branch',
                    files: [{ path: 'src/a.ts', status: 'added' }],
                    head_digest: headDigest,
                    captured_at_ms: 1_800_000_000_000,
                },
            }),
        );
        const client = clientWith(transport);

        const branch = await client.changesSnapshot({
            taskId: TASK_ID,
            scope: 'branch',
            grantId: 'gr_abcd1234',
        });
        expect(branch).toEqual({
            scope: 'branch',
            files: [{ path: 'src/a.ts', status: 'added' }],
            headDigest,
            capturedAtMs: 1_800_000_000_000,
        });

        expect(transport.intents).toHaveLength(1);
        const intent = transport.intents[0]!;
        expect(intent.tool).toBe('git.changes');
        expectDataIdentity(intent);
        expect(intent.arguments).toEqual({
            grant_id: 'gr_abcd1234',
            scope: 'branch',
        });
        expectNoUrlArguments(intent);

        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'last_turn' }),
        ).resolves.toBeNull();
        expect(transport.intents).toHaveLength(1);
    });

    test('read_artifact_checks_browser_artifact_before_host', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {}),
        );
        const client = clientWith(transport);
        const bytes = new TextEncoder().encode('browser artifact');
        const digest = await sha256Bytes(bytes);
        client.registerLocalFile({
            path: 'src/browser.txt',
            mediaType: 'text/plain',
            bytes,
            digest,
            artifactId: 'art_browser',
        });

        const byId = await client.readArtifact({ artifactId: 'art_browser' });
        expect(byId).toMatchObject({
            path: 'src/browser.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
        });
        expect(new TextDecoder().decode(byId.bytes)).toBe('browser artifact');

        const byPath = await client.readArtifact({ path: 'src/browser.txt' });
        expect(byPath).toMatchObject({ path: 'src/browser.txt', digest });
        expect(transport.intents).toHaveLength(0);
    });

    test('host_artifact_fallback_verifies_digest_and_size', async () => {
        const bytes = new TextEncoder().encode('host artifact');
        const digest = await sha256Bytes(bytes);
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: bytes.byteLength,
                digest,
                bytes_base64: base64(bytes),
            }),
        );
        const client = clientWith(transport);

        const content = await client.readArtifact({
            artifactId: 'art_host',
            taskId: TASK_ID,
        });
        expect(content).toMatchObject({
            path: 'host.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
        });
        expect(new TextDecoder().decode(content.bytes)).toBe('host artifact');
        expect(Object.keys(content).sort()).toEqual([
            'bytes',
            'digest',
            'mediaType',
            'path',
            'sizeBytes',
        ]);

        expect(transport.intents).toHaveLength(1);
        const intent = transport.intents[0]!;
        expect(intent.tool).toBe('artifact.read');
        expectDataIdentity(intent);
        expect(intent.arguments).toEqual({ artifact_id: 'art_host' });
        expectNoUrlArguments(intent);
        expect(transport.contexts[0]?.taskId).toBe(TASK_ID);
    });

    test('host_artifact_range_verifies_returned_bytes', async () => {
        const bytes = new TextEncoder().encode('host ');
        const digest = await sha256Bytes(bytes);
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: bytes.byteLength,
                artifact_size_bytes: 13,
                digest,
                whole_digest: `sha256:${'b'.repeat(64)}`,
                range: { start: 0, end: bytes.byteLength },
                bytes_base64: base64(bytes),
            }),
        );
        const client = clientWith(transport);

        const content = await client.readArtifact({ artifactId: 'art_host' });
        expect(new TextDecoder().decode(content.bytes)).toBe('host ');
        expect(content.sizeBytes).toBe(5);
        expect(content.digest).toBe(digest);
    });

    test('host_artifact_rejects_corrupt_base64_digest_and_size', async () => {
        const bytes = new TextEncoder().encode('host artifact');
        const digest = await sha256Bytes(bytes);

        const corruptBase64 = clientWith(
            new RecordingHostTransport(async (intent) =>
                resultFor(intent, {
                    path: 'host.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength,
                    digest,
                    bytes_base64: '***',
                }),
            ),
        );
        await expect(
            corruptBase64.readArtifact({ artifactId: 'art_host' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });

        const corruptDigest = clientWith(
            new RecordingHostTransport(async (intent) =>
                resultFor(intent, {
                    path: 'host.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength,
                    digest: `sha256:${'0'.repeat(64)}`,
                    bytes_base64: base64(bytes),
                }),
            ),
        );
        await expect(
            corruptDigest.readArtifact({ artifactId: 'art_host' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });

        const corruptSize = clientWith(
            new RecordingHostTransport(async (intent) =>
                resultFor(intent, {
                    path: 'host.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength + 1,
                    digest,
                    bytes_base64: base64(bytes),
                }),
            ),
        );
        await expect(
            corruptSize.readArtifact({ artifactId: 'art_host' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('host_artifact_result_must_not_carry_urls_or_expiring_links', async () => {
        const bytes = new TextEncoder().encode('host artifact');
        const digest = await sha256Bytes(bytes);
        for (const field of [
            { url: 'https://example.invalid/artifact' },
            { download_url: 'https://example.invalid/artifact' },
            { expires_at: 1_900_000_000_000 },
            { token: 'page-secret' },
        ]) {
            const client = clientWith(
                new RecordingHostTransport(async (intent) =>
                    resultFor(intent, {
                        path: 'host.txt',
                        media_type: 'text/plain',
                        size_bytes: bytes.byteLength,
                        digest,
                        bytes_base64: base64(bytes),
                        ...field,
                    }),
                ),
            );
            await expect(
                client.readArtifact({ artifactId: 'art_host' }),
            ).rejects.toMatchObject({ code: 'invalid_event' });
        }
    });

    test('host_data_calls_keep_static_host_unavailable_behavior', async () => {
        const client = new AgentClient();
        await expect(client.tree()).resolves.toEqual([]);
        await expect(
            client.tree({ source: 'host' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'last_turn' }),
        ).resolves.toBeNull();
        await expect(
            client.readArtifact({ artifactId: 'art_missing' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
    });

    test('host_data_arguments_never_carry_unscoped_or_secret_fields', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, { entries: [] }),
        );
        const client = clientWith(transport);
        await client.tree({
            source: 'host',
            path: 'src',
            grantId: 'gr_abcd1234',
        });
        const intent = transport.intents[0]!;
        expect(Object.keys(intent.arguments).sort()).toEqual([
            'grant_id',
            'path',
        ]);
        expect(JSON.stringify(intent)).not.toContain('granted');
        expect(JSON.stringify(intent)).not.toContain('approval_receipt');
    });
});
