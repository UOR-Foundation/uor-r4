import { expect, test } from '@playwright/test';

declare global {
    interface Window {
        __studioMvmTest?: {
            status(): Promise<{ architecture: string; guestCid: number; cowStateSha256: string }>;
            exec(cmd: string, cwd?: string): Promise<{ exitCode: number; stdout: string; stderr: string; compiledElfMachine?: number }>;
            write(path: string, text: string): Promise<unknown>;
            read(path: string): Promise<{ text: string }>;
            snapshot(): Promise<{ snapshotId: string; digest: string }>;
            restore(snapshot: { snapshotId: string; digest: string }): Promise<unknown>;
            dispose(): Promise<unknown>;
        };
    }
}

const SOURCE = `#include <stdio.h>
long fib(long n) { if (n <= 1) return n; return fib(n - 1) + fib(n - 2); }
long collatz(long x) { long s = 0; while (x > 1) { if ((x % 2) == 0) x = x / 2; else x = 3 * x + 1; s++; } return s; }
int main(void) { long f = fib(12); long c = collatz(27); printf("FIB12=%ld COLLATZ27=%ld SUM=%ld\\n", f, c, f + c); return 0; }`;

test('mvm real browser WASM compiles, executes, persists and refuses disposal', async ({ page }) => {
    await page.goto('/?typed=1&studioTest=1');
    await page.waitForFunction(() => window.__studioMvmTest !== undefined);
    const status = await page.evaluate(() => window.__studioMvmTest!.status());
    expect(status.architecture).toBe('riscv64-rv64imac');
    expect(status.guestCid).toBe(3);

    await page.evaluate((source) => window.__studioMvmTest!.write('/workspace/src/bench.c', source), SOURCE);
    const executed = await page.evaluate(() => window.__studioMvmTest!.exec('cc /workspace/src/bench.c -o /workspace/bench && /workspace/bench', '/workspace'));
    expect(executed.exitCode).toBe(0);
    expect(executed.compiledElfMachine).toBe(243);
    expect(executed.stdout).toContain('FIB12=144 COLLATZ27=111 SUM=255');

    const saved = await page.evaluate(() => window.__studioMvmTest!.snapshot());
    expect(saved.snapshotId).toMatch(/^sha256:/);
    expect(saved.digest).toMatch(/^sha256:/);
    await page.reload();
    await page.waitForFunction(() => window.__studioMvmTest !== undefined);
    const restored = await page.evaluate(() => window.__studioMvmTest!.read('/workspace/src/bench.c'));
    expect(restored.text).toBe(SOURCE);
    await page.evaluate((saved) => window.__studioMvmTest!.restore(saved), saved);
    await page.evaluate(() => window.__studioMvmTest!.dispose());
    await expect(page.evaluate(() => window.__studioMvmTest!.status())).rejects.toThrow(/disposed/i);
});

test('mvm corrupt persistence is refused rather than replaced', async ({ page }) => {
    await page.goto('/?typed=1&studioTest=1');
    await page.waitForFunction(() => window.__studioMvmTest !== undefined);
    await page.evaluate(() => window.__studioMvmTest!.write('/workspace/keep.txt', 'keep'));
    await page.evaluate(() => new Promise<void>((resolve, reject) => {
        const request = indexedDB.open('uor-r4.studio.mvm', 1);
        request.onsuccess = () => {
            const db = request.result;
            const transaction = db.transaction('sandboxes', 'readwrite');
            transaction.objectStore('sandboxes').put({ version: 99 }, 'corrupt-tab');
            transaction.oncomplete = () => { db.close(); resolve(); };
            transaction.onerror = () => reject(transaction.error);
        };
        request.onerror = () => reject(request.error);
    }));
    await page.evaluate(() => {
        const key = Object.keys(sessionStorage).find((name) => name.includes('mvm'));
        if (key) sessionStorage.setItem(key, 'corrupt-tab');
    });
    await page.reload();
    await page.waitForFunction(() => window.__studioMvmTest !== undefined);
    await expect(page.evaluate(() => window.__studioMvmTest!.status())).rejects.toThrow(/version|corrupt/i);
});
