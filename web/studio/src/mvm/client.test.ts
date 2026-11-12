import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import { MvmClient, MvmClientError, type MvmWorkerLike } from './client.js';
import { createMvmRequest, parseMvmResponse } from './protocol.js';

class FakeWorker implements MvmWorkerLike {
    readonly postMessage = vi.fn();
    readonly terminate = vi.fn();
    private listener: ((event: { data: string }) => void) | null = null;

    constructor(private readonly results: Record<string, unknown> = {}) {}

    addEventListener(_type: 'message', listener: (event: { data: string }) => void): void {
        this.listener = listener;
    }

    removeEventListener(): void {
        this.listener = null;
    }

    start(): void {
        this.postMessage.mockImplementation((request: { id: number; kind: string }) => {
            const result = this.results[request.kind] ?? { ok: true };
            queueMicrotask(() => this.listener?.({ data: JSON.stringify({ id: request.id, ok: true, result }) }));
        });
    }
}

function makeClient(results: Record<string, unknown> = {}) {
    const worker = new FakeWorker(results);
    worker.start();
    return { worker, client: new MvmClient({ workerFactory: () => worker }) };
}

describe('MvmClient protocol and lifecycle', () => {
    it('mvm_worker_protocol_is_bounded_and_typed', () => {
        const request = createMvmRequest({ id: 7, kind: 'exec', payload: { cmd: 'mvm status' } });
        expect(JSON.parse(request)).toEqual({ id: 7, kind: 'exec', payload: { cmd: 'mvm status' } });
        expect(parseMvmResponse(JSON.stringify({ id: 7, ok: true, result: { answer: 42 } }))).toEqual({ id: 7, ok: true, result: { answer: 42 } });
        expect(() => createMvmRequest({ id: 8, kind: 'invalid' as never, payload: {} })).toThrow(MvmClientError);
        expect(() => createMvmRequest({ id: 8, kind: 'write', payload: { path: 'a', text: 'x'.repeat(2 * 1024 * 1024 + 1) } })).toThrow(/bound|size|large/i);
        expect(() => parseMvmResponse('{"id":7,"ok":true,"result":null,"error":null,"extra":1}')).toThrow(/shape|key/i);
    });

    it('mvm_worker_is_lazy_until_terminal_open', async () => {
        const { client, worker } = makeClient({ status: { architecture: 'riscv64-rv64imac', guestCid: 3 } });
        expect(worker.postMessage).not.toHaveBeenCalled();
        expect(client.history()).toEqual([]);
        await client.status();
        expect(worker.postMessage).toHaveBeenCalledOnce();
        await client.status();
        expect(worker.postMessage).toHaveBeenCalledTimes(2);
    });

    it('mvm_status_and_exact_c_output_match_native', async () => {
        const { client } = makeClient({
            status: { architecture: 'riscv64-rv64imac', guestCid: 3 },
            exec: { exitCode: 0, stdout: 'FIB12=144 COLLATZ27=111 SUM=255\n', stderr: '', compiledElfMachine: 243 },
        });
        expect((await client.status()).architecture).toBe('riscv64-rv64imac');
        const result = await client.exec('cc bench.c -o bench && ./bench', '/workspace');
        expect(result.stdout).toBe('FIB12=144 COLLATZ27=111 SUM=255\n');
        expect(result.compiledElfMachine).toBe(243);
    });

    it('mvm_dispose_rejects_later_commands', async () => {
        const { client, worker } = makeClient();
        await client.status();
        await client.dispose();
        expect(worker.terminate).toHaveBeenCalledOnce();
        await expect(client.status()).rejects.toThrow(/disposed/i);
        expect(worker.postMessage).toHaveBeenCalledTimes(2);
    });

    it('mvm_history_is_memory_only_and_secret_like_input_is_refused', async () => {
        const { client, worker } = makeClient({ exec: { exitCode: 0, stdout: 'ok', stderr: '' } });
        await expect(client.exec('export GITHUB_TOKEN=do-not-persist')).rejects.toThrow(/secret|token/i);
        await expect(client.exec('printf "Authorization: Bearer abc"')).rejects.toThrow(/secret|authorization/i);
        expect(worker.postMessage).not.toHaveBeenCalled();
        expect(client.history()).toEqual([]);
        await client.exec('mvm status');
        expect(client.history()).toHaveLength(1);
        expect(client.history()[0].persisted).toBe(false);
    });

    it('mvm_worker_exports_no_secret_apis', () => {
        const wasmPath = resolve(process.cwd(), '../../crates/uor-r4-mvm/src/wasm.rs');
        const wasm = readFileSync(wasmPath, 'utf8');
        expect(wasm).toContain('pub fn mvm_dispatch');
        for (const forbidden of ['register_host_secret', 'proxy_vsock_egress', 'guest_ram_contains', 'import_cow_sectors']) {
            expect(wasm).not.toContain(`pub fn ${forbidden}`);
            expect(wasm).not.toContain(`#[wasm_bindgen]\npub fn ${forbidden}`);
        }
        const worker = readFileSync(resolve(process.cwd(), 'src/mvm/worker.ts'), 'utf8');
        expect(worker).not.toContain('register_host_secret');
        expect(worker).not.toContain('proxy_vsock_egress');
    });
});
