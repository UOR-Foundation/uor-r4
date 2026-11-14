import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import {
    CompanionGitHubTransport,
    FetchGitHubTransport,
    GitHubClient,
    GitHubError,
    type GitHubTransport,
    type GitHubTransportRequest,
    type RepoTarget,
} from './githubClient.js';

const TARGET: RepoTarget = { owner: 'casey-allard', repo: 'studio-smoke', defaultBranch: 'main' };
const BASE = 'a'.repeat(40);
const COMMIT = 'b'.repeat(40);
const BLOB = 'c'.repeat(40);

function harness(results: unknown[] = []) {
    const calls: GitHubTransportRequest[] = [];
    const transport: GitHubTransport = {
        kind: 'fetch',
        async request(input) {
            calls.push(input);
            if (results.length === 0) throw new Error(`unexpected ${input.method} ${input.path}`);
            return results.shift();
        },
    };
    const client = new GitHubClient({ target: TARGET, transport });
    return { client, calls, transport };
}

function approval(action: 'branch' | 'commit' | 'pr') {
    return { action, taskId: 'tsk_0000000000000001', runId: 'run_0000000000000001' } as const;
}

describe('GitHub client', () => {
    it('github_public_repo_read_needs_no_token', async () => {
        const fetchMock = vi.fn(async () => new Response(JSON.stringify({ full_name: 'casey-allard/studio-smoke', default_branch: 'main' }), { status: 200 }));
        const transport = new FetchGitHubTransport({ fetch: fetchMock });
        const client = new GitHubClient({ target: TARGET, transport });
        await expect(client.getRepo()).resolves.toMatchObject({ default_branch: 'main' });
        const [, init] = fetchMock.mock.calls[0]!;
        const headers = init?.headers as Record<string, string>;
        expect(headers.Authorization).toBeUndefined();
        expect(headers.Accept).toBe('application/vnd.github+json');
        expect(fetchMock.mock.calls[0]![0]).toBe('https://api.github.com/repos/casey-allard/studio-smoke');
    });

    it('github_desktop_read_uses_companion_or_unavailable', async () => {
        const execute = vi.fn(async () => ({ full_name: 'casey-allard/studio-smoke', default_branch: 'main' }));
        const transport = new CompanionGitHubTransport({ execute });
        const client = new GitHubClient({ target: TARGET, transport });
        await client.getRepo();
        expect(execute).toHaveBeenCalledWith({ method: 'GET', path: '/repos/casey-allard/studio-smoke', body: undefined, approval: undefined });
        const unavailable = new CompanionGitHubTransport({ execute: async () => { throw new GitHubError('host_unavailable', 'companion unavailable'); } });
        await expect(new GitHubClient({ target: TARGET, transport: unavailable }).getRepo()).rejects.toThrow('companion unavailable');
    });

    it('github_repo_target_is_pinned', async () => {
        const { client, calls } = harness([{ object: { sha: BASE } }]);
        await expect(client.createRef({ branch: 'fix', baseBranch: 'main', baseRefSha: BASE, approval: approval('branch') })).rejects.toThrow(/unexpected/i);
        expect(calls).toHaveLength(2);
        expect(calls.every((call) => call.path.startsWith('/repos/casey-allard/studio-smoke'))).toBe(true);
        const other = new GitHubClient({ target: TARGET, transport: { kind: 'fetch', request: async () => ({}) } });
        await expect(other.getRepo({ ...TARGET, repo: 'other' })).rejects.toThrow(/pinned|cross.repository/i);
    });

    it('github_production_and_unapproved_repo_are_refused_in_smoke', () => {
        for (const target of [TARGET, { ...TARGET, owner: 'UOR-Foundation', repo: 'uor-r4' }]) {
            expect(() => new GitHubClient({ target, transport: { kind: 'fetch', request: async () => ({}) }, smokeTarget: { ...TARGET, owner: 'someone-else' } })).toThrow(/smoke|allowlist|production/i);
        }
    });

    it('github_cross_repo_pr_and_default_branch_write_are_refused', async () => {
        const { client, calls } = harness();
        await expect(client.createRef({ branch: 'main', baseBranch: 'main', baseRefSha: BASE, approval: approval('branch') })).rejects.toThrow(/default branch/i);
        await expect(client.createPull({ base: 'main', head: 'fix', headOwner: 'someone-else', title: 'fix', body: '', baseRefSha: BASE, approval: approval('pr') })).rejects.toThrow(/cross.repository/i);
        expect(calls).toHaveLength(0);
    });

    it('github_write_requires_explicit_action', async () => {
        const { client, calls } = harness();
        await expect(client.createRef({ branch: 'fix', baseBranch: 'main', baseRefSha: BASE })).rejects.toThrow(/approval|action/i);
        await expect(client.createRef({ branch: 'fix', baseBranch: 'main', baseRefSha: BASE, approval: approval('commit') })).rejects.toThrow(/approval|action/i);
        expect(calls).toHaveLength(0);
    });

    it('github_token_is_never_persisted_logged_or_registered_with_mvm', async () => {
        const token = 'github_pat_SYNTHETIC_NOT_A_CREDENTIAL';
        const fetchMock = vi.fn(async () => new Response(JSON.stringify({ object: { sha: BASE } }), { status: 200 }));
        const transport = new FetchGitHubTransport({ fetch: fetchMock, token });
        const client = new GitHubClient({ target: TARGET, transport });
        const mvm = { register_host_secret: vi.fn(), write: vi.fn() };
        const storageSet = vi.spyOn(Storage.prototype, 'setItem');
        const consoleError = vi.spyOn(console, 'error').mockImplementation(() => undefined);
        await client.getRef('main');
        expect(mvm.register_host_secret).not.toHaveBeenCalled();
        expect(mvm.write).not.toHaveBeenCalled();
        expect(storageSet).not.toHaveBeenCalled();
        expect(consoleError).not.toHaveBeenCalled();
        expect(JSON.stringify(localStorage)).not.toContain(token);
        client.clearToken();
        expect(transport.hasToken()).toBe(false);
        storageSet.mockRestore();
        consoleError.mockRestore();
    });

    it('github_ref_content_sha_and_pr_payloads_are_exact', async () => {
        const { client, calls } = harness([
            { object: { sha: BASE } },
            { ref: 'refs/heads/fix', object: { sha: COMMIT } },
            { object: { sha: COMMIT } },
            { path: 'fixture.txt', sha: BLOB, content: Buffer.from('before\n').toString('base64'), encoding: 'base64' },
            { content: { path: 'fixture.txt', sha: 'd'.repeat(40) }, commit: { sha: 'e'.repeat(40) } },
            { object: { sha: BASE } },
            { number: 17, html_url: 'https://github.com/casey-allard/studio-smoke/pull/17', head: { ref: 'fix', sha: 'e'.repeat(40) }, base: { ref: 'main' } },
        ]);
        await client.createRef({ branch: 'fix', baseBranch: 'main', baseRefSha: BASE, approval: approval('branch') });
        await client.createOrUpdateContent({ branch: 'fix', path: 'fixture.txt', content: 'after\n', message: 'Repair fixture', baseRefSha: COMMIT, baseBlobSha: BLOB, approval: approval('commit') });
        await client.createPull({ base: 'main', head: 'fix', title: 'Repair fixture', body: 'Small approved edit', baseRefSha: BASE, approval: approval('pr') });
        expect(calls[1]).toMatchObject({ method: 'POST', path: '/repos/casey-allard/studio-smoke/git/refs', body: { ref: 'refs/heads/fix', sha: BASE } });
        expect(calls[4]).toMatchObject({ method: 'PUT', path: '/repos/casey-allard/studio-smoke/contents/fixture.txt', body: { message: 'Repair fixture', content: Buffer.from('after\n').toString('base64'), sha: BLOB, branch: 'fix' } });
        expect(calls[6]).toMatchObject({ method: 'POST', path: '/repos/casey-allard/studio-smoke/pulls', body: { title: 'Repair fixture', head: 'casey-allard:fix', base: 'main', body: 'Small approved edit' } });
    });

    it('github_stale_base_is_refused', async () => {
        const { client, calls } = harness([{ object: { sha: 'f'.repeat(40) } }]);
        await expect(client.createRef({ branch: 'fix', baseBranch: 'main', baseRefSha: BASE, approval: approval('branch') })).rejects.toThrow(/stale|base/i);
        expect(calls).toHaveLength(1);
    });

    it('github_double_click_creates_one_pr', async () => {
        const { client, calls } = harness([{ object: { sha: BASE } }, { number: 17, html_url: 'https://example.test/pull/17' }]);
        const input = { base: 'main', head: 'fix', title: 'Repair fixture', body: '', baseRefSha: BASE, approval: approval('pr') };
        const [left, right] = await Promise.all([client.createPull(input), client.createPull(input)]);
        expect(left).toBe(right);
        expect(calls.filter((call) => call.path.endsWith('/pulls'))).toHaveLength(1);
    });

    it('github_compound_payload_and_error_are_bounded', async () => {
        const huge = 'x'.repeat(70_000);
        const transport = new FetchGitHubTransport({ fetch: async () => new Response(huge, { status: 200 }) });
        await expect(new GitHubClient({ target: TARGET, transport }).getRepo()).rejects.toThrow(/bounded|payload|large/i);
        const failure = new FetchGitHubTransport({ fetch: async () => new Response('no', { status: 500 }) });
        await expect(new GitHubClient({ target: TARGET, transport: failure }).getRepo()).rejects.toThrow(/500|failed/i);
    });

    it('github_companion_binds_selected_existing_task_run_without_synthetic_records', async () => {
        const execute = vi.fn(async () => ({}));
        const transport = new CompanionGitHubTransport({ execute });
        const client = new GitHubClient({ target: TARGET, transport });
        await expect(client.createRef({ branch: 'fix', baseBranch: 'main', baseRefSha: BASE, approval: { action: 'branch' } })).rejects.toThrow(/task|run/i);
        expect(execute).not.toHaveBeenCalled();
    });

    it('github_repository_seed_records_remote_blob_shas_and_exact_paths', async () => {
        const { client, calls } = harness([
            [{ path: 'src', type: 'dir', sha: BASE }, { path: 'README.md', type: 'file', sha: BASE }],
            [{ path: 'src/fix.c', type: 'file', sha: BLOB }],
            { path: 'README.md', sha: BASE, content: Buffer.from('read me\n').toString('base64'), encoding: 'base64' },
            { path: 'src/fix.c', sha: BLOB, content: Buffer.from('int main(void){return 42;}\n').toString('base64'), encoding: 'base64' },
        ]);
        const writes: Array<{ path: string; text: string }> = [];
        const blobs = await client.seedRepository({ write: async (path, text) => { writes.push({ path, text }); return {}; } });
        expect(blobs).toEqual({ 'README.md': BASE, 'src/fix.c': BLOB });
        expect(writes).toEqual([
            { path: '/workspace/README.md', text: 'read me\n' },
            { path: '/workspace/src/fix.c', text: 'int main(void){return 42;}\n' },
        ]);
        expect(calls.map((call) => call.path)).toEqual([
            '/repos/casey-allard/studio-smoke/contents',
            '/repos/casey-allard/studio-smoke/contents/src',
            '/repos/casey-allard/studio-smoke/contents/README.md',
            '/repos/casey-allard/studio-smoke/contents/src/fix.c',
        ]);
    });

    it('github_repository_seed_refuses_truncation_before_mvm_writes', async () => {
        const entries = Array.from({ length: 100 }, (_, index) => ({ path: `file-${index}.txt`, type: 'file', sha: BASE }));
        const { client } = harness([entries]);
        const write = vi.fn(async () => ({}));
        await expect(client.seedRepository({ write })).rejects.toThrow(/truncated|bounded/i);
        expect(write).not.toHaveBeenCalled();
    });

    it('github_tauri_token_entry_is_disabled_and_csp_unchanged', () => {
        expect(() => new FetchGitHubTransport({ platform: 'tauri', token: 'synthetic' })).toThrow(/token|disabled|direct/i);
        const config = JSON.parse(readFileSync(resolve(process.cwd(), '../../src-tauri/tauri.conf.json'), 'utf8')) as { app: { security: { csp: string } } };
        expect(config.app.security.csp).not.toContain('api.github.com');
        expect(config.app.security.csp).toBe("default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost; worker-src 'self'");
    });
});
