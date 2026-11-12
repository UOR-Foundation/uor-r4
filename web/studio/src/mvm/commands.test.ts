import { describe, expect, it } from 'vitest';

import { executeVirtualCommand } from './commands.js';
import { VirtualGit, type VirtualGitFiles } from './virtualGit.js';

function files(): VirtualGitFiles {
    const values = new Map<string, string>();
    return {
        async list(prefix = '/workspace') {
            return [...values.keys()].filter((path) => path.startsWith(prefix)).sort().map((path) => ({ path, kind: 'file' as const, sizeBytes: values.get(path)!.length }));
        },
        async read(path: string) {
            const text = values.get(path);
            if (text === undefined) throw new Error(`not found: ${path}`);
            return { text, revision: 1 };
        },
        async write(path: string, text: string) {
            values.set(path, text);
            return { path, revision: 1 };
        },
        async remove(path: string) { values.delete(path); },
    };
}

describe('virtual terminal commands', () => {
    it('git_commands_render_local_staging_and_deterministic_diffs', async () => {
        const git = new VirtualGit(files(), { author: 'Test <test@invalid>', timestamp: '2026-01-02T03:04:05.000Z' });
        expect((await executeVirtualCommand(git, 'git init')).result.exitCode).toBe(0);
        await git.files.write('/workspace/a.txt', 'one\n');
        expect((await executeVirtualCommand(git, 'git add a.txt')).result.exitCode).toBe(0);
        expect((await executeVirtualCommand(git, 'git commit -m "fix the parser"')).result.exitCode).toBe(0);
        const log = await executeVirtualCommand(git, 'git log --oneline');
        expect(log.result.stdout).toMatch(/[0-9a-f]{12} local staging fix the parser/);
        expect((await executeVirtualCommand(git, 'git status')).result.stdout).toContain('working tree clean');

        await git.files.write('/workspace/a.txt', 'two\n');
        const diff = await executeVirtualCommand(git, 'git diff');
        expect(diff.result.stdout).toContain('-one');
        expect(diff.result.stdout).toContain('+two');
        expect((await executeVirtualCommand(git, 'git branch feature')).result.stdout).toContain('feature');
        expect((await executeVirtualCommand(git, 'git switch feature')).result.exitCode).toBe(0);
        expect((await executeVirtualCommand(git, 'git checkout main')).result.exitCode).toBe(0);
        expect((await executeVirtualCommand(git, 'echo hello')).handled).toBe(false);
    });

    it('cargo_rustc_report_unavailable_not_success', async () => {
        const git = new VirtualGit(files());
        for (const command of ['cargo check', 'cargo test', 'rustc --version']) {
            const result = await executeVirtualCommand(git, command);
            expect(result.handled).toBe(true);
            expect(result.persist).toBe(false);
            expect(result.result.exitCode).not.toBe(0);
            expect(result.result.stderr).toContain('no genuine guest Rust toolchain');
            expect(result.result.stdout).toBe('');
        }
    });

    it('git_diff_preserves_unterminated_single_lines', async () => {
        const backend = files();
        const git = new VirtualGit(backend);
        await git.init();
        await backend.write('/workspace/a.txt', 'one');
        await git.add(['a.txt']);
        await git.commit('initial');
        await backend.write('/workspace/a.txt', 'two');
        const result = await executeVirtualCommand(git, 'git diff');
        expect(result.result.stdout).toContain('-one');
        expect(result.result.stdout).toContain('+two');
    });
});
