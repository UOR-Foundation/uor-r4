import type { MvmExecResult } from './client.js';
import { VirtualGit, VirtualGitError } from './virtualGit.js';

export interface VirtualCommandOutcome {
    handled: boolean;
    persist: boolean;
    result: MvmExecResult;
}

function words(command: string): string[] {
    const result: string[] = [];
    let current = '';
    let quote = '';
    let escaped = false;
    for (const character of command.trim()) {
        if (escaped) {
            current += character;
            escaped = false;
            continue;
        }
        if (character === '\\' && quote !== "'") {
            escaped = true;
            continue;
        }
        if (quote) {
            if (character === quote) quote = '';
            else current += character;
            continue;
        }
        if (character === "'" || character === '"') {
            quote = character;
        } else if (/\s/.test(character)) {
            if (current) {
                result.push(current);
                current = '';
            }
        } else {
            current += character;
        }
    }
    if (escaped || quote) throw new VirtualGitError('invalid_command', 'unterminated command quoting');
    if (current) result.push(current);
    return result;
}

function output(exitCode: number, stdout = '', stderr = ''): MvmExecResult {
    return { exitCode, stdout, stderr };
}

function statusText(status: Awaited<ReturnType<VirtualGit['status']>>): string {
    const lines = [`On branch ${status.branch}`];
    const codes = { added: 'A', modified: 'M', deleted: 'D', untracked: '?' };
    for (const change of status.changes) {
        const staged = change.staged === null ? ' ' : codes[change.staged];
        const unstaged = change.unstaged === null ? ' ' : codes[change.unstaged];
        lines.push(`${staged}${unstaged} ${change.path}`);
    }
    if (status.clean) lines.push('working tree clean');
    return `${lines.join('\n')}\n`;
}

async function runGit(git: VirtualGit, args: string[], cwd: string): Promise<MvmExecResult> {
    const [action, ...rest] = args;
    switch (action) {
        case 'init':
            if (rest.length) return output(1, '', 'usage: git init\n');
            await git.init();
            return output(0, 'Initialized empty virtual Git repository in /workspace (local staging)\n');
        case 'status':
            if (rest.length) return output(1, '', 'usage: git status\n');
            return output(0, statusText(await git.status()));
        case 'branch': {
            if (rest.length > 1) return output(1, '', 'usage: git branch [name]\n');
            const branches = await git.branch(rest[0]);
            return output(0, branches.map((branch) => `${branch.current ? '*' : ' '} ${branch.name}\n`).join(''));
        }
        case 'add':
            if (!rest.length) return output(1, '', 'usage: git add <path> [...]\n');
            await git.add(rest, cwd);
            return output(0);
        case 'diff': {
            if (rest.length > 1 || (rest[0] !== undefined && rest[0] !== '--cached')) return output(1, '', 'usage: git diff [--cached]\n');
            return output(0, await git.diff(rest[0] === '--cached'));
        }
        case 'log': {
            if (rest.length > 1 || (rest[0] !== undefined && rest[0] !== '--oneline')) return output(1, '', 'usage: git log [--oneline]\n');
            const commits = await git.log();
            if (rest[0] === '--oneline') {
                return output(0, commits.map((commit) => `${commit.hash.slice(7, 19)} local staging ${commit.message}\n`).join(''));
            }
            return output(0, commits.map((commit) => [
                `commit ${commit.hash}`,
                `tree ${virtualTreeIdentity(commit.tree)}`,
                `parents ${commit.parents.join(' ') || '(none)'}`,
                `author ${commit.author}`,
                `timestamp ${commit.timestamp}`,
                'local staging',
                `    ${commit.message}`,
                '',
            ].join('\n')).join('\n'));
        }
        case 'commit': {
            if (rest[0] !== '-m' || rest.length < 2) return output(1, '', 'usage: git commit -m <message>\n');
            const commit = await git.commit(rest.slice(1).join(' '));
            return output(0, `[${git.serialize().currentBranch} ${commit.hash.slice(7, 19)}] local staging ${commit.message}\n`);
        }
        case 'switch':
        case 'checkout': {
            const create = rest[0] === '-b';
            const name = create ? rest[1] : rest[0] === '--' ? git.serialize().currentBranch : rest[0];
            if (!name || rest.length > (create ? 2 : rest[0] === '--' ? 2 : 1)) return output(1, '', `usage: git ${action} [-b] <branch>\n`);
            const branch = await git.checkout(name, create);
            return output(0, `Switched to ${branch.current ? 'branch' : 'unknown'} ${name}\n`);
        }
        default:
            return output(1, '', `unsupported virtual Git command: ${action ?? '(missing)'}\n`);
    }
}

function virtualTreeIdentity(tree: Record<string, string>): string {
    return JSON.stringify(Object.entries(tree).sort(([left], [right]) => left.localeCompare(right)));
}

export async function executeVirtualCommand(git: VirtualGit, command: string, cwd = '/workspace'): Promise<VirtualCommandOutcome> {
    let args: string[];
    try {
        args = words(command);
    } catch (error) {
        if (error instanceof VirtualGitError) return { handled: true, persist: false, result: output(1, '', `${error.message}\n`) };
        throw error;
    }
    const [program, ...rest] = args;
    if (program === 'cargo' || program === 'rustc') {
        return {
            handled: true,
            persist: false,
            result: output(2, '', `${program} is unavailable: no genuine guest Rust toolchain is installed.\n`),
        };
    }
    if (program !== 'git') return { handled: false, persist: false, result: output(0) };
    try {
        const result = await runGit(git, rest, cwd);
        return { handled: true, persist: git.dirty, result };
    } catch (error) {
        if (error instanceof VirtualGitError) {
            return { handled: true, persist: git.dirty, result: output(1, '', `${error.message}\n`) };
        }
        throw error;
    }
}
