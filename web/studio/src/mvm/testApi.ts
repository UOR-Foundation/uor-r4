import { MvmClient } from './client.js';

declare global {
    interface Window {
        __studioMvmTest?: {
            status(): Promise<{ architecture: string; guestCid: number; cowStateSha256: string }>;
            exec(cmd: string, cwd?: string): Promise<{ exitCode: number; stdout: string; stderr: string; compiledElfMachine?: number }>;
            write(path: string, text: string): Promise<unknown>;
            read(path: string): Promise<{ text: string }>;
            snapshot(): Promise<{ snapshotId: string; digest: string }>;
            restore(snapshot: { snapshotId: string; digest: string }): Promise<unknown>;
            dispose(): Promise<void>;
        };
    }
}

export function installMvmTestApi(): void {
    if (window.__studioMvmTest !== undefined) return;
    const client = new MvmClient();
    window.__studioMvmTest = {
        status: () => client.status() as Promise<{ architecture: string; guestCid: number; cowStateSha256: string }>,
        exec: (cmd, cwd) => client.exec(cmd, cwd),
        write: (path, text) => client.write(path, text),
        read: (path) => client.read(path),
        snapshot: () => client.snapshot(),
        restore: (snapshot) => client.restore(snapshot),
        dispose: () => client.dispose(),
    };
}
