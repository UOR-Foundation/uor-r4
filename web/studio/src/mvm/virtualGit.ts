import { MVM_LIMITS } from './protocol.js';

export interface VirtualCommit {
    hash: string;
    parents: string[];
    message: string;
    author: string;
    timestamp: string;
    tree: Record<string, string>;
}

export interface VirtualGitSnapshot {
    version: 1;
    initialized: boolean;
    currentBranch: string;
    branches: Record<string, string | null>;
    commits: Record<string, VirtualCommit>;
    staged: Record<string, string | null>;
    blobs: Record<string, string>;
}

export interface VirtualGitFiles {
    list(prefix?: string): Promise<Array<{ path: string; kind: 'file'; sizeBytes: number }>>;
    read(path: string): Promise<{ text: string; revision?: number }>;
    write(path: string, text: string): Promise<unknown>;
    remove(path: string): Promise<unknown>;
}

export interface VirtualBranch {
    name: string;
    current: boolean;
    hash: string | null;
}

export interface VirtualGitChange {
    path: string;
    staged: 'added' | 'modified' | 'deleted' | null;
    unstaged: 'added' | 'modified' | 'deleted' | 'untracked' | null;
}

export interface VirtualGitStatus {
    branch: string;
    head: string | null;
    clean: boolean;
    changes: VirtualGitChange[];
}

export interface VirtualGitOptions {
    author?: string;
    timestamp?: string;
}

export class VirtualGitError extends Error {
    constructor(readonly code: string, message: string) {
        super(message);
        this.name = 'VirtualGitError';
    }
}

const DEFAULT_AUTHOR = 'UOR Studio <local@invalid>';
const DEFAULT_TIMESTAMP = '1970-01-01T00:00:00.000Z';
const HASH = /^sha256:[0-9a-f]{64}$/;

function emptySnapshot(): VirtualGitSnapshot {
    return { version: 1, initialized: false, currentBranch: 'main', branches: {}, commits: {}, staged: {}, blobs: {} };
}

function isObject(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function bytes(text: string): number {
    return new TextEncoder().encode(text).byteLength;
}

function diffLines(text: string): string[] {
    if (!text) return [];
    const lines = text.split('\n');
    if (lines[lines.length - 1] === '') lines.pop();
    return lines;
}

function sorted<T>(entries: Iterable<[string, T]>): Record<string, T> {
    return Object.fromEntries([...entries].sort(([left], [right]) => left.localeCompare(right)));
}

function validBranchName(name: string): boolean {
    return /^[A-Za-z0-9][A-Za-z0-9._/-]{0,63}$/.test(name)
        && !name.includes('..')
        && !name.endsWith('/')
        && !name.endsWith('.lock');
}

export function workspacePath(cwd: string, input: string): string {
    const value = input.trim();
    if (!value || /[\0\r\n'"]/.test(value)) throw new VirtualGitError('invalid_path', 'Git path is empty or contains unsupported characters');
    const joined = value.startsWith('/') ? value : `${cwd}/${value}`;
    const parts: string[] = [];
    for (const part of joined.split('/')) {
        if (!part || part === '.') continue;
        if (part === '..') {
            if (parts.length === 1) throw new VirtualGitError('invalid_path', 'Git path escapes the workspace');
            parts.pop();
        } else {
            parts.push(part);
        }
    }
    const path = `/${parts.join('/')}`;
    if (path !== '/workspace' && !path.startsWith('/workspace/')) {
        throw new VirtualGitError('invalid_path', 'Git path is outside /workspace');
    }
    return path;
}

function validWorkspaceFile(path: unknown): path is string {
    if (typeof path !== 'string' || !path.startsWith('/workspace/')) return false;
    try {
        return workspacePath('/workspace', path) === path && path !== '/workspace';
    } catch {
        return false;
    }
}

export async function sha256Text(text: string): Promise<string> {
    const input = new TextEncoder().encode(text);
    const digest = await crypto.subtle.digest('SHA-256', input.buffer as ArrayBuffer);
    return `sha256:${[...new Uint8Array(digest)].map((value) => value.toString(16).padStart(2, '0')).join('')}`;
}

export async function virtualCommitHash(commit: Omit<VirtualCommit, 'hash'>): Promise<string> {
    const canonical = JSON.stringify({
        parents: commit.parents,
        message: commit.message,
        author: commit.author,
        timestamp: commit.timestamp,
        tree: sorted(Object.entries(commit.tree)),
    });
    return sha256Text(canonical);
}

export function validateVirtualGitSnapshot(input: unknown): VirtualGitSnapshot {
    if (!isObject(input) || input.version !== 1 || typeof input.initialized !== 'boolean') {
        throw new VirtualGitError('invalid_git_state', 'virtual Git snapshot version or shape is invalid');
    }
    if (typeof input.currentBranch !== 'string' || !validBranchName(input.currentBranch)) {
        throw new VirtualGitError('invalid_git_state', 'virtual Git current branch is invalid');
    }
    if (!isObject(input.branches) || !isObject(input.commits) || !isObject(input.staged) || !isObject(input.blobs)) {
        throw new VirtualGitError('invalid_git_state', 'virtual Git maps are invalid');
    }
    if (!input.initialized) {
        const empty = Object.keys(input.branches).length === 0 && Object.keys(input.commits).length === 0
            && Object.keys(input.staged).length === 0 && Object.keys(input.blobs).length === 0;
        if (!empty || input.currentBranch !== 'main') {
            throw new VirtualGitError('invalid_git_state', 'uninitialized virtual Git state is not empty');
        }
        return emptySnapshot();
    }
    if (!Object.prototype.hasOwnProperty.call(input.branches, input.currentBranch)) {
        throw new VirtualGitError('invalid_git_state', 'virtual Git current branch has no ref');
    }
    const branches: Record<string, string | null> = {};
    for (const [name, hash] of Object.entries(input.branches)) {
        if (!validBranchName(name) || (hash !== null && (typeof hash !== 'string' || !HASH.test(hash)))) {
            throw new VirtualGitError('invalid_git_state', `virtual Git branch is invalid: ${name}`);
        }
        branches[name] = hash;
    }
    const blobs: Record<string, string> = {};
    let totalBytes = 0;
    for (const [hash, text] of Object.entries(input.blobs)) {
        if (!HASH.test(hash) || typeof text !== 'string') {
            throw new VirtualGitError('invalid_git_state', 'virtual Git blob identity or content is invalid');
        }
        const size = bytes(text);
        if (size > MVM_LIMITS.maxFileBytes) throw new VirtualGitError('invalid_git_state', 'virtual Git blob exceeds its byte bound');
        totalBytes += size;
        blobs[hash] = text;
    }
    if (totalBytes > MVM_LIMITS.maxWorkspaceBytes) {
        throw new VirtualGitError('invalid_git_state', 'virtual Git blobs exceed the 16 MiB bound');
    }
    const commitEntries = Object.entries(input.commits);
    if (commitEntries.length > MVM_LIMITS.maxGitCommits) {
        throw new VirtualGitError('invalid_git_state', 'virtual Git history exceeds 256 commits');
    }
    const commits: Record<string, VirtualCommit> = {};
    for (const [identity, value] of commitEntries) {
        if (!HASH.test(identity) || !isObject(value) || value.hash !== identity) {
            throw new VirtualGitError('invalid_git_state', 'virtual Git commit identity is invalid');
        }
        const { parents, message, author, timestamp, tree } = value;
        if (!Array.isArray(parents) || parents.length > 16 || parents.some((parent) => typeof parent !== 'string' || !HASH.test(parent))) {
            throw new VirtualGitError('invalid_git_state', `virtual Git commit parents are invalid: ${identity}`);
        }
        for (const parent of parents) {
            if (!Object.prototype.hasOwnProperty.call(input.commits, parent)) {
                throw new VirtualGitError('invalid_git_state', `virtual Git parent is missing: ${parent}`);
            }
        }
        if (typeof message !== 'string' || message.length === 0 || bytes(message) > 4096
            || typeof author !== 'string' || author.length === 0 || bytes(author) > 256
            || typeof timestamp !== 'string' || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(timestamp)
            || !isObject(tree) || Object.keys(tree).length > MVM_LIMITS.maxFiles) {
            throw new VirtualGitError('invalid_git_state', `virtual Git commit metadata or tree is invalid: ${identity}`);
        }
        const normalizedTree: Record<string, string> = {};
        for (const [path, blob] of Object.entries(tree)) {
            if (!validWorkspaceFile(path) || typeof blob !== 'string' || !HASH.test(blob) || !Object.prototype.hasOwnProperty.call(blobs, blob)) {
                throw new VirtualGitError('invalid_git_state', `virtual Git tree entry is invalid: ${path}`);
            }
            normalizedTree[path] = blob;
        }
        commits[identity] = {
            hash: identity,
            parents: [...parents] as string[],
            message,
            author,
            timestamp,
            tree: sorted(Object.entries(normalizedTree)),
        };
    }
    for (const [name, hash] of Object.entries(branches)) {
        if (hash !== null && !Object.prototype.hasOwnProperty.call(commits, hash)) {
            throw new VirtualGitError('invalid_git_state', `virtual Git branch ${name} references missing commit ${hash}`);
        }
    }
    const staged: Record<string, string | null> = {};
    for (const [path, blob] of Object.entries(input.staged)) {
        if (!validWorkspaceFile(path) || (blob !== null && (typeof blob !== 'string' || !HASH.test(blob) || !Object.prototype.hasOwnProperty.call(blobs, blob)))) {
            throw new VirtualGitError('invalid_git_state', `virtual Git staged entry is invalid: ${path}`);
        }
        staged[path] = blob;
    }
    return {
        version: 1,
        initialized: true,
        currentBranch: input.currentBranch,
        branches: sorted(Object.entries(branches)),
        commits: sorted(Object.entries(commits)),
        staged: sorted(Object.entries(staged)),
        blobs: sorted(Object.entries(blobs)),
    };
}

export async function assertVirtualGitHashes(snapshot: VirtualGitSnapshot): Promise<void> {
    for (const [identity, text] of Object.entries(snapshot.blobs)) {
        if (await sha256Text(text) !== identity) throw new VirtualGitError('invalid_git_state', `virtual Git blob hash mismatch: ${identity}`);
    }
    for (const commit of Object.values(snapshot.commits)) {
        const { hash: _hash, ...body } = commit;
        if (await virtualCommitHash(body) !== commit.hash) {
            throw new VirtualGitError('invalid_git_state', `virtual Git commit hash mismatch: ${commit.hash}`);
        }
    }
}

export class VirtualGit {
    private state = emptySnapshot();
    private dirtyState = false;
    private readonly author: string;
    private readonly timestamp: string;

    constructor(readonly files: VirtualGitFiles, options: VirtualGitOptions = {}) {
        this.author = options.author ?? DEFAULT_AUTHOR;
        this.timestamp = options.timestamp ?? DEFAULT_TIMESTAMP;
    }

    get dirty(): boolean {
        return this.dirtyState;
    }

    markPersisted(): void {
        this.dirtyState = false;
    }

    serialize(): VirtualGitSnapshot {
        return validateVirtualGitSnapshot(this.state);
    }

    async restore(snapshot?: VirtualGitSnapshot): Promise<void> {
        if (snapshot === undefined) {
            this.state = emptySnapshot();
        } else {
            const normalized = validateVirtualGitSnapshot(snapshot);
            await assertVirtualGitHashes(normalized);
            this.state = normalized;
        }
        this.dirtyState = false;
    }

    async init(): Promise<void> {
        if (this.state.initialized) return;
        this.state = { ...emptySnapshot(), initialized: true, branches: { main: null } };
        this.dirtyState = true;
    }

    private requireInitialized(): void {
        if (!this.state.initialized) throw new VirtualGitError('not_initialized', 'not a virtual Git repository; run git init');
    }

    private head(): string | null {
        this.requireInitialized();
        return this.state.branches[this.state.currentBranch] ?? null;
    }

    private headTree(): Record<string, string> {
        const head = this.head();
        return head === null ? {} : this.state.commits[head]!.tree;
    }

    private stagedTree(): Record<string, string> {
        const tree = { ...this.headTree() };
        for (const [path, blob] of Object.entries(this.state.staged)) {
            if (blob === null) delete tree[path];
            else tree[path] = blob;
        }
        return tree;
    }

    private readTree(tree: Record<string, string>): Map<string, string> {
        return new Map(Object.entries(tree).map(([path, blob]) => [path, this.state.blobs[blob]!]));
    }

    private async workspace(): Promise<Map<string, string | null>> {
        const listed = await this.files.list('/workspace');
        const result = new Map<string, string | null>();
        for (const entry of listed) {
            if (!validWorkspaceFile(entry.path)) throw new VirtualGitError('invalid_path', `guest file path is invalid: ${entry.path}`);
            try {
                result.set(entry.path, (await this.files.read(entry.path)).text);
            } catch (error) {
                if (error instanceof Error && /utf-?8/i.test(error.message)) result.set(entry.path, null);
                else throw error;
            }
        }
        return result;
    }

    async status(): Promise<VirtualGitStatus> {
        const head = this.head();
        const headTree = this.headTree();
        const stagedTree = this.stagedTree();
        const workspace = await this.workspace();
        const paths = new Set([...Object.keys(headTree), ...Object.keys(stagedTree), ...workspace.keys()]);
        const changes: VirtualGitChange[] = [];
        for (const path of [...paths].sort()) {
            const headBlob = headTree[path];
            const stagedBlob = stagedTree[path];
            let staged: VirtualGitChange['staged'] = null;
            if (headBlob === undefined && stagedBlob !== undefined) staged = 'added';
            else if (headBlob !== undefined && stagedBlob === undefined) staged = 'deleted';
            else if (headBlob !== stagedBlob) staged = 'modified';

            const text = workspace.get(path);
            const workBlob = text === undefined || text === null ? undefined : await sha256Text(text);
            let unstaged: VirtualGitChange['unstaged'] = null;
            if (text === null && stagedBlob === undefined) unstaged = 'untracked';
            else if (headBlob === undefined && stagedBlob === undefined && text !== undefined) unstaged = 'untracked';
            else if (text === undefined && stagedBlob !== undefined) unstaged = 'deleted';
            else if (stagedBlob !== workBlob) unstaged = 'modified';
            if (staged !== null || unstaged !== null) changes.push({ path, staged, unstaged });
        }
        return { branch: this.state.currentBranch, head, clean: changes.length === 0, changes };
    }

    async branch(name?: string): Promise<VirtualBranch[]> {
        this.requireInitialized();
        if (name !== undefined) {
            if (!validBranchName(name)) throw new VirtualGitError('invalid_branch', `invalid branch name: ${name}`);
            if (Object.prototype.hasOwnProperty.call(this.state.branches, name)) {
                throw new VirtualGitError('duplicate_branch', `branch already exists: ${name}`);
            }
            this.state.branches[name] = this.head();
            this.state = this.serialize();
            this.dirtyState = true;
        }
        return Object.entries(this.state.branches).sort(([left], [right]) => left.localeCompare(right)).map(([branch, hash]) => ({
            name: branch,
            current: branch === this.state.currentBranch,
            hash,
        }));
    }

    async add(inputs: string[], cwd = '/workspace'): Promise<void> {
        this.requireInitialized();
        if (inputs.length === 0) throw new VirtualGitError('path_required', 'git add requires at least one path');
        const paths = inputs.map((input) => workspacePath(cwd, input));
        if (new Set(paths).size !== paths.length) throw new VirtualGitError('duplicate_path', 'duplicate Git path after canonicalization');
        const workspace = await this.workspace();
        const candidates = new Set<string>();
        for (const path of paths) {
            const exact = workspace.has(path);
            const prefix = `${path}/`;
            const selected = [...workspace.keys()].filter((candidate) => candidate === path || candidate.startsWith(prefix));
            const previous = new Set([...Object.keys(this.headTree()), ...Object.keys(this.stagedTree())]);
            const removed = [...previous].filter((candidate) => candidate === path || candidate.startsWith(prefix));
            if (!exact && selected.length === 0 && removed.length === 0) {
                throw new VirtualGitError('path_not_found', `path did not match any workspace files: ${path}`);
            }
            for (const candidate of [...selected, ...removed]) candidates.add(candidate);
        }
        for (const path of [...candidates].sort()) {
            const text = workspace.get(path);
            if (text === undefined) {
                this.state.staged[path] = null;
            } else {
                if (text === null) throw new VirtualGitError('binary_file_unsupported', `binary file cannot be staged as UTF-8 text: ${path}`);
                const hash = await sha256Text(text);
                this.state.blobs[hash] = text;
                this.state.staged[path] = hash;
            }
        }
        this.state = this.serialize();
        this.dirtyState = true;
    }

    async diff(cached = false): Promise<string> {
        this.requireInitialized();
        const before = this.readTree(cached ? this.headTree() : this.stagedTree());
        const after = cached ? this.readTree(this.stagedTree()) : await this.workspace();
        const paths = new Set([...before.keys(), ...after.keys()]);
        let output = '';
        for (const path of [...paths].sort()) {
            const left = before.get(path);
            const right = after.get(path);
            if (left === right) continue;
            output += `diff --git a${path} b${path}\n`;
            output += `--- ${left === undefined ? '/dev/null' : `a${path}`}\n`;
            output += `+++ ${right === undefined ? '/dev/null' : `b${path}`}\n`;
            if (left === null || right === null) {
                output += 'Binary files differ\n';
                continue;
            }
            for (const line of diffLines(left ?? '')) output += `-${line}\n`;
            for (const line of diffLines(right ?? '')) output += `+${line}\n`;
        }
        return output;
    }

    async commit(message: string): Promise<VirtualCommit> {
        this.requireInitialized();
        const text = message.trim();
        if (!text) throw new VirtualGitError('message_required', 'git commit requires a non-empty message');
        if (bytes(text) > 4096) throw new VirtualGitError('limit_exceeded', 'commit message exceeds its byte bound');
        if (Object.keys(this.state.staged).length === 0) throw new VirtualGitError('nothing_to_commit', 'nothing staged to commit');
        if (Object.keys(this.state.commits).length >= MVM_LIMITS.maxGitCommits) {
            throw new VirtualGitError('limit_exceeded', 'virtual Git history is limited to 256 commits');
        }
        const head = this.head();
        const body = {
            parents: head === null ? [] : [head],
            message: text,
            author: this.author,
            timestamp: this.timestamp,
            tree: this.stagedTree(),
        };
        const hash = await virtualCommitHash(body);
        const commit: VirtualCommit = { hash, ...body };
        this.state.commits[hash] = commit;
        this.state.branches[this.state.currentBranch] = hash;
        this.state.staged = {};
        this.state = this.serialize();
        this.dirtyState = true;
        return commit;
    }

    async log(): Promise<VirtualCommit[]> {
        const head = this.head();
        if (head === null) return [];
        const result: VirtualCommit[] = [];
        const queue = [head];
        const seen = new Set<string>();
        while (queue.length > 0 && result.length < MVM_LIMITS.maxGitCommits) {
            const hash = queue.shift()!;
            if (seen.has(hash)) continue;
            seen.add(hash);
            const commit = this.state.commits[hash];
            if (commit === undefined) throw new VirtualGitError('invalid_git_state', `virtual Git history references missing commit ${hash}`);
            result.push(commit);
            queue.push(...commit.parents);
        }
        return result;
    }

    async checkout(name: string, create = false): Promise<VirtualBranch> {
        this.requireInitialized();
        if (!validBranchName(name)) throw new VirtualGitError('invalid_branch', `invalid branch name: ${name}`);
        const exists = Object.prototype.hasOwnProperty.call(this.state.branches, name);
        if (!exists && !create) throw new VirtualGitError('unknown_branch', `unknown branch: ${name}`);
        if (exists && create) throw new VirtualGitError('duplicate_branch', `branch already exists: ${name}`);
        const hash = exists ? this.state.branches[name]! : this.head();
        const target = hash === null ? {} : this.state.commits[hash]!.tree;
        const current = this.headTree();
        for (const path of Object.keys(current).sort()) {
            if (!Object.prototype.hasOwnProperty.call(target, path)) await this.files.remove(path);
        }
        for (const [path, blob] of Object.entries(target).sort(([left], [right]) => left.localeCompare(right))) {
            await this.files.write(path, this.state.blobs[blob]!);
        }
        this.state.branches[name] = hash;
        this.state.currentBranch = name;
        this.state.staged = {};
        this.state = this.serialize();
        this.dirtyState = true;
        return { name, current: true, hash };
    }
}
