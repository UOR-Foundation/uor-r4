import { useEffect, useRef, useState } from 'preact/hooks';
import type { MvmClient, MvmExecResult } from '../mvm/client.js';

interface MvmTerminalProps {
    client: MvmClient;
}

const MAX_OUTPUT_BYTES = 1024 * 1024;

function resolveCwd(cwd: string, argument: string): string {
    const input = argument.trim();
    const joined = input.startsWith('/') ? input : `${cwd}/${input}`;
    const parts: string[] = [];
    for (const part of joined.split('/')) {
        if (!part || part === '.') continue;
        if (part === '..') parts.pop();
        else parts.push(part);
    }
    return `/${parts.join('/')}` || '/';
}

function statusLine(status: Record<string, unknown>): string {
    return `${String(status.architecture ?? 'unknown')} · CID ${String(status.guestCid ?? '?')} · ${String(status.dirtyCowSectorCount ?? 0)} CoW sectors`;
}

export function MvmTerminal({ client }: MvmTerminalProps) {
    const mounted = useRef(true);
    const [status, setStatus] = useState('Initializing MVM…');
    const [cwd, setCwd] = useState('/workspace');
    const [command, setCommand] = useState('');
    const [output, setOutput] = useState('');
    const [running, setRunning] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const composing = useRef(false);
    const [unavailable, setUnavailable] = useState(false);

    function append(text: string): void {
        setOutput((current) => `${current}${text}`.slice(-MAX_OUTPUT_BYTES));
    }

    async function refreshStatus(): Promise<void> {
        try {
            const result = await client.status();
            if (!mounted.current) return;
            setStatus(statusLine(result));
            setUnavailable(false);
            setError(null);
        } catch (failure) {
            if (!mounted.current) return;
            setStatus(`MVM unavailable: ${failure instanceof Error ? failure.message : String(failure)}`);
            setUnavailable(true);
        }
    }

    useEffect(() => {
        mounted.current = true;
        void refreshStatus();
        return () => {
            mounted.current = false;
        };
    }, [client]);

    async function run(raw: string): Promise<void> {
        const cmd = raw.trim();
        if (!cmd || running || unavailable) return;
        setRunning(true);
        setError(null);
        try {
            if (cmd === 'clear') {
                setOutput('');
                setCommand('');
                return;
            }
            if (cmd === 'help') {
                append('Commands: help pwd cd ls cat write patch mkdir rm mv cp git cc mvm vsock status history clear reset\n');
                setCommand('');
                return;
            }
            if (cmd === 'history') {
                const history = client.history();
                append(history.map((entry) => `${entry.cwd} $ ${entry.cmd}\n`).join('') || 'No commands yet.\n');
                setCommand('');
                return;
            }
            if (cmd === 'reset') {
                await client.reset();
                setOutput('');
                setCwd('/workspace');
                setCommand('');
                await refreshStatus();
                return;
            }
            if (cmd === 'mvm status' || cmd === 'status') {
                const result = await client.status();
                append(`${JSON.stringify(result, null, 2)}\n`);
                setStatus(statusLine(result));
                setCommand('');
                return;
            }
            if (cmd === 'cd' || cmd.startsWith('cd ')) {
                const next = resolveCwd(cwd, cmd.slice(2).trim());
                const result = await client.exec('pwd', next);
                setCwd(next);
                append(result.stdout);
                setCommand('');
                return;
            }
            append(`${cwd} $ ${cmd}\n`);
            const result: MvmExecResult = await client.exec(cmd, cwd);
            if (result.stdout) append(result.stdout);
            if (result.stderr) append(result.stderr);
            if (result.exitCode !== 0) setError(`Command exited with code ${result.exitCode}`);
            setCommand('');
        } catch (failure) {
            setError(`MVM command failed: ${failure instanceof Error ? failure.message : String(failure)}`);
        } finally {
            if (mounted.current) setRunning(false);
        }
    }

    return <section aria-label="MVM terminal" class="mvm-terminal" data-testid="mvm-terminal">
        <header class="lex-pane-heading"><div><h2>MVM terminal</h2><p role="status" aria-live="polite">{status}</p></div><span>RV64IMAC</span></header>
        <pre role="log" aria-live="polite" class="mvm-terminal-output">{output}</pre>
        {error && <p role="alert" class="workbench-alert">{error}</p>}
        <form class="mvm-terminal-form" onSubmit={(event) => { event.preventDefault(); if (!composing.current) void run(command); }}>
            <label for="mvm-command">Command</label>
            <input
                id="mvm-command"
                aria-label="MVM command"
                value={command}
                disabled={running || unavailable}
                placeholder="mvm status"
                onInput={(event) => setCommand(event.currentTarget.value)}
                onCompositionStart={() => { composing.current = true; }}
                onCompositionEnd={() => { composing.current = false; }}
                onKeyDown={(event) => {
                    if (event.key === 'ArrowUp') {
                        event.preventDefault();
                        const history = client.history();
                        if (history.length > 0) setCommand(history[history.length - 1]!.cmd);
                    } else if (event.key === 'ArrowDown') {
                        event.preventDefault();
                        setCommand('');
                    } else if (event.key === 'Enter' && !composing.current && !event.isComposing && event.keyCode !== 229) {
                        event.preventDefault();
                        void run(command);
                    }
                }}
            />
            <button type="submit" disabled={running || unavailable}>{running ? 'Running…' : 'Run'}</button>
            <button type="button" aria-label="Clear terminal" onClick={() => { setOutput(''); setError(null); }}>Clear</button>
            <button type="button" aria-label="Reset sandbox" disabled={running} onClick={() => { void run('reset'); }}>Reset</button>
        </form>
        <p class="mvm-terminal-cwd">cwd: {cwd} · Enter to run · ↑/↓ history · history is memory-only</p>
    </section>;
}
