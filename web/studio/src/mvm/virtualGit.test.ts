import { describe, expect, it } from 'vitest';

import { VirtualGit, type VirtualGitFiles } from './virtualGit.js';

class MemoryFiles implements VirtualGitFiles {
    readonly values = new Map<string, string>();
    private readonly revisions = new Map<string, number>();

    async list(prefix = '/workspace') {
        return [...this.values.keys()].filter((path) => path.startsWith(prefix)).sort().map((path) => ({
            path,
            kind: 'file' as const,
            sizeBytes: new TextEncoder().encode(this.values.get(path)!).byteLength,
        }));
    }

    async read(path: string) {
        const text = this.values.get(path);
        if (text === undefined) throw new Error(`guest file not found: ${path}`);
        return { text, revision: this.revisions.get(path) ?? 1 };
    }

    async write(path: string, text: string) {
        const revision = (this.revisions.get(path) ?? 0) + 1;
        this.values.set(path, text);
        this.revisions.set(path, revision);
        return { path, revision };
    }

    async remove(path: string) {
        this.values.delete(path);
        this.revisions.delete(path);
    }
}

async function repository() {
    const files = new MemoryFiles();
    const git = new VirtualGit(files, { author: 'Test <test@invalid>', timestamp: '2026-01-02T03:04:05.000Z' });
    await git.init();
    await files.write('/workspace/a.txt', 'one\n');
    await git.add(['a.txt']);
    const initial = await git.commit('initial');
    return { files, git, initial };
}

describe('VirtualGit', () => {
    it('git_init_branch_add_commit_log_are_deterministic', async () => {
        const left = await repository();
        const right = await repository();
        expect(left.initial.hash).toBe(right.initial.hash);
        expect(left.initial.parents).toEqual([]);
        expect(left.initial.author).toBe('Test <test@invalid>');
        expect(left.initial.timestamp).toBe('2026-01-02T03:04:05.000Z');
        const blob = left.initial.tree['/workspace/a.txt']!;
        expect(left.git.serialize().blobs[blob]).toBe('one\n');

        await left.git.branch('feature');
        await left.git.checkout('feature');
        await left.files.write('/workspace/a.txt', 'two\n');
        await left.git.add(['a.txt']);
        const second = await left.git.commit('second');
        expect(second.hash).not.toBe(left.initial.hash);
        expect(second.parents).toEqual([left.initial.hash]);
        expect((await left.git.log()).map((commit) => commit.hash)).toEqual([second.hash, left.initial.hash]);
    });

    it('git_checkout_restores_tree', async () => {
        const { files, git, initial } = await repository();
        await git.branch('feature');
        await git.checkout('feature');
        await files.write('/workspace/a.txt', 'feature\n');
        await files.write('/workspace/feature.txt', 'new\n');
        await git.add(['a.txt', 'feature.txt']);
        const featured = await git.commit('feature change');

        await git.checkout('main');
        expect((await files.read('/workspace/a.txt')).text).toBe('one\n');
        await expect(files.read('/workspace/feature.txt')).rejects.toThrow(/not found/);
        expect(git.serialize().currentBranch).toBe('main');
        expect(git.serialize().branches.main).toBe(initial.hash);

        await git.checkout('feature');
        expect((await files.read('/workspace/a.txt')).text).toBe('feature\n');
        expect((await files.read('/workspace/feature.txt')).text).toBe('new\n');
        expect(git.serialize().branches.feature).toBe(featured.hash);
    });

    it('git_duplicate_branch_and_path_are_refused', async () => {
        const { git } = await repository();
        await git.branch('feature');
        await expect(git.branch('feature')).rejects.toThrow(/already exists|duplicate/i);
        await expect(git.add(['a.txt', './a.txt'])).rejects.toThrow(/duplicate/i);
        await expect(git.add(['../outside.txt'])).rejects.toThrow(/outside|workspace|path/i);
        await expect(git.checkout('missing')).rejects.toThrow(/unknown|missing/i);
    });

    it('git_save_and_status_agree', async () => {
        const { files, git } = await repository();
        expect((await git.status()).clean).toBe(true);
        await files.write('/workspace/a.txt', 'edited\n');
        let status = await git.status();
        expect(status.clean).toBe(false);
        expect(status.changes).toEqual([{ path: '/workspace/a.txt', staged: null, unstaged: 'modified' }]);

        await git.add(['a.txt']);
        status = await git.status();
        expect(status.changes).toEqual([{ path: '/workspace/a.txt', staged: 'modified', unstaged: null }]);
        await files.write('/workspace/a.txt', 'edited again\n');
        status = await git.status();
        expect(status.changes).toEqual([{ path: '/workspace/a.txt', staged: 'modified', unstaged: 'modified' }]);
        await git.add(['a.txt']);
        await git.commit('edit');
        expect((await git.status()).clean).toBe(true);
    });

    it('git_corrupt_history_and_cap_are_refused', async () => {
        const { files, git, initial } = await repository();
        const corrupt = git.serialize();
        corrupt.commits[initial.hash]!.parents = [`sha256:${'f'.repeat(64)}`];
        await expect(git.restore(corrupt)).rejects.toThrow(/parent|history/i);

        for (let index = 1; index < 256; index += 1) {
            await files.write('/workspace/a.txt', `value ${index}\n`);
            await git.add(['a.txt']);
            await git.commit(`commit ${index}`);
        }
        expect(Object.keys(git.serialize().commits)).toHaveLength(256);
        await files.write('/workspace/a.txt', 'overflow\n');
        await git.add(['a.txt']);
        await expect(git.commit('overflow')).rejects.toThrow(/256|limit|bound/i);
    });
});
