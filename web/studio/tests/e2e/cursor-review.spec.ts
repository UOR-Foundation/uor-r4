import { expect, test, type Page } from '@playwright/test';

const TEST_URL = '/?typed=1&studioTest=1';
const DB_NAME = 'uor-r4.studio.agent';

interface StudioTestCall {
    method: string;
    args: unknown[];
}

interface Outcome {
    ok: boolean;
    code?: string;
    message?: string;
    record?: unknown;
}

interface FixtureSeed {
    taskId: string;
    runId: string;
}

interface StoredComment {
    key: IDBValidKey;
    record: Record<string, unknown>;
}

async function openStudio(page: Page): Promise<void> {
    await page.goto(TEST_URL);
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
}

async function call<T>(page: Page, method: string, ...args: unknown[]): Promise<T> {
    const payload: StudioTestCall = { method, args };
    return page.evaluate(({ method, args }) => {
        const api = (window as unknown as {
            __studioTest?: Record<string, (...callArgs: unknown[]) => Promise<unknown>>;
        }).__studioTest;
        if (!api) throw new Error('window.__studioTest is missing');
        return api[method]!(...args);
    }, payload) as Promise<T>;
}

async function selectTask(page: Page, taskId: string): Promise<void> {
    const row = page.locator(`#studio-typed-root [data-task-id="${taskId}"] .task-select`);
    await expect(row).toBeVisible();
    await row.click();
    await expect(page.locator('#studio-typed-root [data-task-detail-header]')).toBeVisible();
}

async function openChanges(page: Page): Promise<void> {
    const navigation = page.locator('#studio-typed-root #studio-navigation');
    await navigation.getByRole('button', { name: 'Changes', exact: true }).click();
    await expect(page.locator('#studio-typed-root [data-diff-review]')).toBeVisible();
}

async function readStoredComments(page: Page): Promise<StoredComment[]> {
    return page.evaluate(async (dbName) => {
        const request = <T>(input: IDBRequest<T>): Promise<T> =>
            new Promise<T>((resolve, reject) => {
                input.onsuccess = () => resolve(input.result);
                input.onerror = () => reject(input.error);
            });
        const done = (tx: IDBTransaction): Promise<void> =>
            new Promise<void>((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onabort = () => reject(tx.error ?? new Error('transaction aborted'));
                tx.onerror = () => reject(tx.error ?? new Error('transaction failed'));
            });
        const db = await new Promise<IDBDatabase>((resolve, reject) => {
            const open = indexedDB.open(dbName);
            open.onsuccess = () => resolve(open.result);
            open.onerror = () => reject(open.error);
        });
        try {
            const tx = db.transaction('comments', 'readonly');
            const store = tx.objectStore('comments');
            const keys = await request(store.getAllKeys());
            const records = await request(store.getAll());
            await done(tx);
            return keys.map((key, index) => ({ key, record: records[index] as Record<string, unknown> }));
        } finally {
            db.close();
        }
    }, DB_NAME);
}

test.describe('Task 21 changes review and local comments', () => {
    test('diff_comment_survives_reload', async ({ page }) => {
        await openStudio(page);
        await call<{ phase: string }>(page, 'open');
        const seeded = await call<FixtureSeed>(page, 'createTaskAndEvents', {
            mode: 'message',
            title: 'Review persistence task',
        });
        const appended = await call<Outcome>(page, 'appendChanges');
        expect(appended.ok, JSON.stringify(appended)).toBe(true);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await expect.poll(async () => (await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        await selectTask(page, seeded.taskId);
        await openChanges(page);
        const review = page.locator('#studio-typed-root [data-diff-review]');
        await expect(review.getByRole('button', { name: 'Last agent turn', exact: true })).toHaveAttribute('aria-pressed', 'true');
        await expect(review).toContainText('Browser last turn');
        await expect(review).toContainText('src/a.ts');

        const text = 'keep this note across reload';
        await review.getByLabel('Comment', { exact: true }).fill(text);
        await review.getByRole('button', { name: 'Add local comment', exact: true }).click();
        await expect(review.locator('[data-diff-current-comments]')).toContainText(text);
        await expect(review.locator('[data-diff-stale-comments]')).toHaveCount(0);

        const stored = await readStoredComments(page);
        expect(stored).toHaveLength(1);
        expect(stored[0]!.key).toEqual([seeded.taskId, 'src/a.ts', 0, 'turn:3:fixture']);
        expect(stored[0]!.record).toMatchObject({
            task_id: seeded.taskId,
            path: 'src/a.ts',
            line: 0,
            body: text,
            head_digest: 'turn:3:fixture',
            scope: 'last_turn',
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await expect.poll(async () => (await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        await selectTask(page, seeded.taskId);
        await openChanges(page);

        const reloaded = page.locator('#studio-typed-root [data-diff-review]');
        await expect(reloaded).toContainText('src/a.ts');
        await expect(reloaded.locator('[data-diff-current-comments]')).toContainText(text);
        await expect(reloaded.locator('[data-diff-stale-comments]')).toHaveCount(0);
        await expect(reloaded).not.toContainText(/\bmerge\b/i);
        await expect(reloaded).not.toContainText(/restore checkpoint|checkpoint/i);
    });

    test('static_browser_reviews_uploaded_file_locally', async ({ page }) => {
        await openStudio(page);
        await openChanges(page);
        const review = page.locator('#studio-typed-root [data-diff-review]');
        const picker = page.locator('#studio-typed-root .diff-upload input[type="file"]');
        await picker.setInputFiles({
            name: 'notes.diff',
            mimeType: 'text/plain',
            buffer: Buffer.from('diff --git a/notes.md b/notes.md\n+local review line\n'),
        });

        await expect(review.getByRole('button', { name: 'Branch', exact: true })).toHaveAttribute('aria-pressed', 'true');
        await expect(review).toContainText('Uploaded file');
        await expect(review).toContainText('notes.diff');
        await expect(review).toContainText('+local review line');
        await expect(review).toContainText('stored in this browser');
        await expect(review).toContainText('Select a browser task to add a local comment.');
    });
});
