export interface RepoTarget {
    owner: string;
    repo: string;
    defaultBranch: string;
}

export type GitHubApprovalAction = 'branch' | 'commit' | 'pr';

export interface GitHubApproval {
    action: GitHubApprovalAction;
    taskId?: string;
    runId?: string;
}

export interface GitHubTransportRequest {
    method: 'GET' | 'POST' | 'PUT';
    path: string;
    body?: Record<string, unknown>;
    repo: { owner: string; repo: string };
    approval?: GitHubApproval;
}

export interface GitHubTransport {
    readonly kind: 'fetch' | 'companion';
    request(input: GitHubTransportRequest): Promise<unknown>;
}

export class GitHubError extends Error {
    constructor(readonly code: string, message: string) {
        super(message.slice(0, 2048));
        this.name = 'GitHubError';
    }
}

const SHA = /^[0-9a-f]{40}$/;
const MAX_JSON_BYTES = 65_536;
const MAX_FILE_BYTES = 1024 * 1024;
const MAX_WORKSPACE_BYTES = 16 * 1024 * 1024;
const MAX_FILES = 512;

function sanitize(value: unknown): string {
    return String(value instanceof Error ? value.message : value)
        .replace(/github_pat_[A-Za-z0-9_]+|ghp_[A-Za-z0-9]+/gi, '[redacted]')
        .slice(0, 2048);
}

function assertTarget(target: RepoTarget): void {
    if (!/^[A-Za-z0-9-]{1,39}$/.test(target.owner)
        || !/^[A-Za-z0-9._-]{1,100}$/.test(target.repo)
        || !validBranch(target.defaultBranch)) {
        throw new GitHubError('invalid_target', 'repository target is invalid');
    }
}

function validBranch(branch: string): boolean {
    return /^[A-Za-z0-9][A-Za-z0-9._/-]{0,63}$/.test(branch)
        && !branch.includes('..')
        && !branch.includes('//')
        && !branch.endsWith('/')
        && !branch.endsWith('.lock');
}

function sameTarget(left: RepoTarget, right: RepoTarget): boolean {
    return left.owner === right.owner && left.repo === right.repo && left.defaultBranch === right.defaultBranch;
}

function production(target: RepoTarget): boolean {
    return target.owner === 'UOR-Foundation' && target.repo === 'uor-r4';
}

function encodedPath(path: string): string {
    return path.split('/').map((part) => encodeURIComponent(part)).join('/');
}

function repoPath(target: RepoTarget, suffix: string): string {
    return `/repos/${encodeURIComponent(target.owner)}/${encodeURIComponent(target.repo)}${suffix}`;
}

function requireApproval(approval: GitHubApproval | undefined, action: GitHubApprovalAction): GitHubApproval {
    if (approval?.action !== action) {
        throw new GitHubError('approval_required', `explicit ${action} approval is required`);
    }
    if (typeof approval.taskId !== 'string' || approval.taskId.length === 0
        || typeof approval.runId !== 'string' || approval.runId.length === 0) {
        throw new GitHubError('task_required', 'a selected existing task and run are required for this write');
    }
    return approval;
}

function requireSha(value: string, label: string): string {
    if (!SHA.test(value)) throw new GitHubError('invalid_arguments', `${label} must be a GitHub 40-character SHA`);
    return value;
}

function requireBranch(branch: string, defaultBranch: string, allowDefault = false): string {
    if (!validBranch(branch)) throw new GitHubError('invalid_arguments', 'branch name is invalid');
    if (!allowDefault && branch === defaultBranch) throw new GitHubError('default_branch_refused', 'writes to the default branch are refused');
    return branch;
}

function utf8Bytes(text: string): number {
    return new TextEncoder().encode(text).byteLength;
}

function encodeBase64(text: string): string {
    const bytes = new TextEncoder().encode(text);
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return globalThis.btoa(binary);
}

function decodeBase64(value: string): string {
    const binary = globalThis.atob(value.replace(/\s/g, ''));
    const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
    return new TextDecoder('utf-8', { fatal: true }).decode(bytes);
}

async function responseJson(response: Response): Promise<unknown> {
    const text = await response.text();
    if (utf8Bytes(text) > MAX_JSON_BYTES) throw new GitHubError('payload_too_large', 'GitHub response exceeds the bounded payload limit');
    if (!response.ok) {
        let detail = text.slice(0, 512);
        try {
            const parsed = JSON.parse(text) as { message?: string };
            if (typeof parsed.message === 'string') detail = parsed.message;
        } catch {
            // The status is still a useful bounded refusal when the body is not JSON.
        }
        throw new GitHubError('request_failed', `GitHub request failed (${response.status}): ${detail}`);
    }
    try {
        return text ? JSON.parse(text) : null;
    } catch {
        throw new GitHubError('invalid_response', 'GitHub response was not valid JSON');
    }
}

export interface FetchGitHubTransportOptions {
    fetch?: typeof fetch;
    token?: string;
    platform?: 'static' | 'tauri';
}

export class FetchGitHubTransport implements GitHubTransport {
    readonly kind = 'fetch' as const;
    private readonly fetchImpl?: typeof fetch;
    private token?: string;

    constructor(options: FetchGitHubTransportOptions = {}) {
        if (options.platform === 'tauri') {
            throw new GitHubError('token_entry_disabled', options.token === undefined
                ? 'direct GitHub fetch is disabled on desktop; use the companion'
                : 'GitHub token entry is disabled on desktop; use the companion');
        }
        this.fetchImpl = options.fetch ?? globalThis.fetch;
        this.token = options.token;
    }

    hasToken(): boolean {
        return this.token !== undefined && this.token.length > 0;
    }

    setToken(token?: string): void {
        this.token = token;
    }

    clearToken(): void {
        this.token = undefined;
    }

    async request(input: GitHubTransportRequest): Promise<unknown> {
        if (typeof this.fetchImpl !== 'function') throw new GitHubError('host_unavailable', 'fetch is unavailable');
        const headers: Record<string, string> = {
            Accept: 'application/vnd.github+json',
            'X-GitHub-Api-Version': '2022-11-28',
        };
        if (input.body !== undefined) headers['Content-Type'] = 'application/json';
        if (input.method !== 'GET') {
            if (!this.hasToken()) throw new GitHubError('credentials_unavailable', 'a memory-only GitHub token is required for this write');
            headers.Authorization = `Bearer ${this.token}`;
        }
        let response: Response;
        try {
            response = await this.fetchImpl(`https://api.github.com${input.path}`, {
                method: input.method,
                headers,
                body: input.body === undefined ? undefined : JSON.stringify(input.body),
                credentials: 'omit',
                cache: 'no-store',
            });
        } catch (error) {
            throw new GitHubError('request_failed', sanitize(error));
        }
        return responseJson(response);
    }
}

export type CompanionExecutor = (input: {
    method: 'GET' | 'POST' | 'PUT';
    path: string;
    body?: Record<string, unknown>;
    approval?: GitHubApproval;
}) => Promise<unknown>;

export class CompanionGitHubTransport implements GitHubTransport {
    readonly kind = 'companion' as const;

    constructor(private readonly options: { execute: CompanionExecutor }) {}

    async request(input: GitHubTransportRequest): Promise<unknown> {
        if (input.method !== 'GET' && (typeof input.approval?.taskId !== 'string' || typeof input.approval?.runId !== 'string'
            || input.approval.taskId.length === 0 || input.approval.runId.length === 0)) {
            throw new GitHubError('task_required', 'a selected existing task and run are required for companion writes');
        }
        try {
            return await this.options.execute({
                method: input.method,
                path: input.path,
                body: input.body,
                approval: input.approval,
            });
        } catch (error) {
            if (error instanceof GitHubError) throw error;
            throw new GitHubError('request_failed', sanitize(error));
        }
    }
}

export interface GitHubRef {
    ref: string;
    object: { sha: string };
}

export interface GitHubContent {
    path: string;
    sha: string;
    text: string;
}

export interface GitHubDirectoryEntry {
    path: string;
    type: 'file' | 'dir';
    sha: string;
    size?: number;
}

export interface CreateRefInput {
    branch: string;
    baseBranch: string;
    baseRefSha: string;
    approval?: GitHubApproval;
}

export interface CreateContentInput {
    branch: string;
    path: string;
    content: string;
    message: string;
    baseRefSha: string;
    baseBlobSha: string;
    approval?: GitHubApproval;
}

export interface CreatePullInput {
    base: string;
    head: string;
    headOwner?: string;
    title: string;
    body: string;
    baseRefSha: string;
    approval?: GitHubApproval;
}

export interface MvmSeedWriter {
    write(path: string, text: string): Promise<unknown>;
}

export class GitHubClient {
    private readonly target: RepoTarget;
    private readonly transport: GitHubTransport;
    private readonly inFlight = new Map<string, Promise<unknown>>();

    constructor(options: { target: RepoTarget; transport: GitHubTransport; smokeTarget?: RepoTarget }) {
        assertTarget(options.target);
        if (options.smokeTarget !== undefined) {
            assertTarget(options.smokeTarget);
            if (!sameTarget(options.target, options.smokeTarget) || production(options.target)) {
                throw new GitHubError('repository_forbidden', 'smoke requires its exact non-production repository allowlist');
            }
        }
        this.target = options.target;
        this.transport = options.transport;
    }

    clearToken(): void {
        if (this.transport instanceof FetchGitHubTransport) this.transport.clearToken();
    }

    private pinned(target?: RepoTarget): RepoTarget {
        if (target !== undefined && !sameTarget(target, this.target)) {
            throw new GitHubError('cross_repository', 'repository does not match the pinned RepoTarget');
        }
        return this.target;
    }

    private async call(method: 'GET' | 'POST' | 'PUT', path: string, body?: Record<string, unknown>, approval?: GitHubApproval): Promise<unknown> {
        try {
            return await this.transport.request({ method, path, body, repo: { owner: this.target.owner, repo: this.target.repo }, approval });
        } catch (error) {
            if (error instanceof GitHubError) throw error;
            throw new GitHubError('request_failed', sanitize(error));
        }
    }

    async getRepo(target?: RepoTarget): Promise<Record<string, unknown>> {
        const pinned = this.pinned(target);
        const value = await this.call('GET', repoPath(pinned, '')) as { full_name?: string };
        if (typeof value.full_name === 'string' && value.full_name !== `${pinned.owner}/${pinned.repo}`) {
            throw new GitHubError('cross_repository', 'GitHub response repository does not match RepoTarget');
        }
        return value as Record<string, unknown>;
    }

    async getRef(branch: string, target?: RepoTarget): Promise<GitHubRef> {
        const pinned = this.pinned(target);
        if (!validBranch(branch)) throw new GitHubError('invalid_arguments', 'branch name is invalid');
        const value = await this.call('GET', repoPath(pinned, `/git/ref/heads/${encodedPath(branch)}`)) as GitHubRef;
        requireSha(value.object?.sha, 'ref SHA');
        return value;
    }

    async listDirectory(path = '', target?: RepoTarget): Promise<GitHubDirectoryEntry[]> {
        const pinned = this.pinned(target);
        if (path.includes('..') || path.startsWith('/')) throw new GitHubError('invalid_path', 'repository directory is invalid');
        const suffix = path ? `/${encodedPath(path)}` : '';
        const value = await this.call('GET', repoPath(pinned, `/contents${suffix}`)) as unknown;
        if (!Array.isArray(value)) throw new GitHubError('invalid_response', 'repository directory response is not an array');
        if (value.length >= 100) throw new GitHubError('truncated_refused', 'repository directory requires pagination or exceeds its bounded read');
        return value.map((entry): GitHubDirectoryEntry => {
            if (entry === null || typeof entry !== 'object' || (entry.type !== 'file' && entry.type !== 'dir')
                || typeof entry.path !== 'string' || entry.path.includes('..') || entry.path.startsWith('/')
                || typeof entry.sha !== 'string' || !SHA.test(entry.sha)) {
                throw new GitHubError('invalid_response', 'repository directory entry is invalid');
            }
            return { path: entry.path, type: entry.type, sha: entry.sha, ...(typeof entry.size === 'number' ? { size: entry.size } : {}) };
        }).sort((left, right) => left.path < right.path ? -1 : left.path > right.path ? 1 : 0);
    }

    async getContent(path: string, branch?: string, target?: RepoTarget): Promise<GitHubContent> {
        const pinned = this.pinned(target);
        if (!path || path.includes('..') || path.startsWith('/')) throw new GitHubError('invalid_path', 'repository content path is invalid');
        const query = branch === undefined ? '' : `?ref=${encodeURIComponent(branch)}`;
        const value = await this.call('GET', repoPath(pinned, `/contents/${encodedPath(path)}${query}`)) as {
            path: string;
            sha: string;
            content: string;
            encoding?: string;
            size?: number;
        };
        if (value.path !== path || value.encoding !== 'base64') throw new GitHubError('invalid_response', 'repository content identity or encoding is invalid');
        requireSha(value.sha, 'blob SHA');
        if (value.size !== undefined && value.size > MAX_FILE_BYTES) throw new GitHubError('limit_exceeded', 'repository file exceeds its byte bound');
        let text: string;
        try {
            text = decodeBase64(value.content);
        } catch {
            throw new GitHubError('unsupported_content', 'repository file is not UTF-8 text');
        }
        if (utf8Bytes(text) > MAX_FILE_BYTES) throw new GitHubError('limit_exceeded', 'repository file exceeds its byte bound');
        return { path, sha: value.sha, text };
    }

    async seedRepository(writer: MvmSeedWriter): Promise<Record<string, string>> {
        const directories = [''];
        const files: Array<{ path: string; sha: string; text: string }> = [];
        const blobs: Record<string, string> = {};
        let totalBytes = 0;
        for (let index = 0; index < directories.length; index += 1) {
            const entries = await this.listDirectory(directories[index]!);
            for (const entry of entries) {
                if (entry.type === 'dir') directories.push(entry.path);
                else files.push(entry);
            }
            if (directories.length + files.length > MAX_FILES) throw new GitHubError('limit_exceeded', 'repository seed exceeds the file bound');
        }
        const seeded: Array<GitHubContent> = [];
        for (const file of files.sort((left, right) => left.path < right.path ? -1 : left.path > right.path ? 1 : 0)) {
            const content = await this.getContent(file.path);
            totalBytes += utf8Bytes(content.text);
            if (totalBytes > MAX_WORKSPACE_BYTES) throw new GitHubError('limit_exceeded', 'repository seed exceeds the workspace byte bound');
            blobs[file.path] = content.sha;
            seeded.push(content);
        }
        for (const file of seeded) await writer.write(`/workspace/${file.path}`, file.text);
        return blobs;
    }

    private flight<T>(key: string, operation: () => Promise<T>): Promise<T> {
        const existing = this.inFlight.get(key);
        if (existing !== undefined) return existing as Promise<T>;
        const result = operation().finally(() => {
            if (this.inFlight.get(key) === result) this.inFlight.delete(key);
        });
        this.inFlight.set(key, result);
        return result;
    }

    async createRef(input: CreateRefInput): Promise<unknown> {
        requireApproval(input.approval, 'branch');
        const branch = requireBranch(input.branch, this.target.defaultBranch);
        const baseRefSha = requireSha(input.baseRefSha, 'base ref SHA');
        return this.flight(`branch:${branch}:${baseRefSha}`, async () => {
            const base = await this.getRef(input.baseBranch);
            if (base.object.sha !== baseRefSha) throw new GitHubError('stale_base', `base ref moved: expected ${baseRefSha}, found ${base.object.sha}`);
            return this.call('POST', repoPath(this.target, '/git/refs'), { ref: `refs/heads/${branch}`, sha: baseRefSha }, input.approval);
        });
    }

    async createOrUpdateContent(input: CreateContentInput): Promise<unknown> {
        requireApproval(input.approval, 'commit');
        const branch = requireBranch(input.branch, this.target.defaultBranch);
        const baseRefSha = requireSha(input.baseRefSha, 'base ref SHA');
        const baseBlobSha = requireSha(input.baseBlobSha, 'base blob SHA');
        if (!input.path || input.path.includes('..') || input.path.startsWith('/')) throw new GitHubError('invalid_path', 'repository content path is invalid');
        if (!input.message.trim() || utf8Bytes(input.message) > 4096) throw new GitHubError('invalid_arguments', 'commit message is empty or too large');
        if (utf8Bytes(input.content) > MAX_FILE_BYTES) throw new GitHubError('limit_exceeded', 'repository file exceeds its byte bound');
        return this.flight(`commit:${branch}:${input.path}:${baseRefSha}:${baseBlobSha}`, async () => {
            const base = await this.getRef(branch);
            if (base.object.sha !== baseRefSha) throw new GitHubError('stale_base', `branch moved: expected ${baseRefSha}, found ${base.object.sha}`);
            const content = await this.getContent(input.path, branch);
            if (content.sha !== baseBlobSha) throw new GitHubError('stale_base', `blob moved: expected ${baseBlobSha}, found ${content.sha}`);
            return this.call('PUT', repoPath(this.target, `/contents/${encodedPath(input.path)}`), {
                message: input.message,
                content: encodeBase64(input.content),
                sha: baseBlobSha,
                branch,
            }, input.approval);
        });
    }

    async createPull(input: CreatePullInput): Promise<Record<string, unknown>> {
        requireApproval(input.approval, 'pr');
        const base = requireBranch(input.base, this.target.defaultBranch, true);
        const head = requireBranch(input.head, this.target.defaultBranch);
        const headOwner = input.headOwner ?? this.target.owner;
        if (headOwner !== this.target.owner) throw new GitHubError('cross_repository', 'cross-repository pull requests are refused');
        if (!input.title.trim() || utf8Bytes(input.title) > 512 || utf8Bytes(input.body) > 16_384) {
            throw new GitHubError('invalid_arguments', 'pull request title or body is invalid or too large');
        }
        const baseRefSha = requireSha(input.baseRefSha, 'base ref SHA');
        return this.flight(`pr:${base}:${head}:${baseRefSha}`, async () => {
            const current = await this.getRef(base);
            if (current.object.sha !== baseRefSha) throw new GitHubError('stale_base', `pull base moved: expected ${baseRefSha}, found ${current.object.sha}`);
            return this.call('POST', repoPath(this.target, '/pulls'), {
                title: input.title,
                head: `${headOwner}:${head}`,
                base,
                body: input.body,
            }, input.approval) as Promise<Record<string, unknown>>;
        });
    }
}
