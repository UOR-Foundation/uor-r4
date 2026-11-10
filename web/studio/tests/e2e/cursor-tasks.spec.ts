import { expect, test, type Page } from '@playwright/test';

const TEST_URL = '/?typed=1&studioTest=1';

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

interface LiveTaskSeed {
    taskId: string;
    runId: string;
    generation: number;
    title: string;
    prompt: string;
}

interface LiveRunSnapshot {
    taskId: string;
    title: string;
    taskState: string;
    latestRunId: string | null;
    runId: string | null;
    runState: string | null;
    retryOfRunId?: string | null;
    messages: string[];
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

function detail(page: Page) {
    return page.locator('#studio-typed-root main[data-region="task-detail"]');
}

test.describe('Task 14 browser task detail, timeline, approvals and errors', () => {
    test('replay_detail_collapses_deltas_pairs_tools_and_never_fabricates_output', async ({ page }) => {
        await openStudio(page);
        expect((await call<Outcome>(page, 'open')).ok).not.toBe(false);
        const seeded = await call<{ taskId: string }>(page, 'createTaskAndEvents', {
            mode: 'detail',
            title: 'Replay detail task',
        });
        expect(seeded.taskId).toMatch(/^tsk_[0-9a-f]{16}$/);
        const taskId = seeded.taskId;

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, taskId);
        const view = detail(page);

        await expect(view.locator('[data-task-detail-header]')).toContainText('Finished');
        await expect(view.locator('[data-task-detail-header]')).toContainText('Replay detail task');
        await expect(view.locator('[data-timeline-kind="assistant"]')).toHaveCount(1);
        await expect(view.locator('[data-timeline-kind="assistant"]')).toContainText('Hello from replay');
        await expect(view.locator('[data-timeline-kind="tool"]')).toHaveCount(1);
        await expect(view.locator('[data-tool-call-id="tool_detail_1"]')).toContainText('{"path":"notes.md"}');
        await expect(view.locator('[data-tool-call-id="tool_detail_1"]')).toContainText('wrote notes');
        await expect(view.locator('[data-timeline-kind="unknown"]')).toContainText('host.instruction');
        await expect(view).not.toContainText('hostile');
        await expect(view.locator('[data-task-error="run"]')).toContainText('malformed_event');
        await expect(view.locator('[data-task-error="run"]')).toContainText('The event stream contained an invalid event. The transcript was not changed.');
        await expect(view.locator('[data-timeline-kind="usage"]')).toContainText('12 input tokens');
        await expect(view.locator('[data-task-plan]')).toContainText('Replay plan');
        await expect(view.locator('[data-task-plan]')).toContainText('Read request');
        await expect(view.locator('[data-task-plan]')).toContainText('Write answer');
        await expect(view).toContainText('Synthetic replay');
        await expect(view).toContainText('not an R4 model');
        await expect(view.locator('.agent-composer')).toBeVisible();
        await expect(view.locator('textarea[aria-label="Prompt"]')).toBeVisible();

        await expect(view.locator('.run-status').getByRole('button', { name: 'Stop', exact: true })).toHaveCount(0);
        await expect(view.getByRole('button', { name: 'Retry', exact: true })).toHaveCount(0);
        await expect(view.getByRole('button', { name: 'Restore Checkpoint', exact: true })).toHaveCount(0);
        await expect(view.getByRole('button', { name: 'Pause', exact: true })).toHaveCount(0);
        await expect(view.getByRole('button', { name: 'Delete', exact: true })).toHaveCount(0);
        await expect(view.getByRole('tab', { name: 'Logs' })).toHaveCount(0);
        await expect(view).not.toContainText(/\$\s?\d/);
    });

    test('approval_card_shows_tool_scope_expiry_and_static_host_without_page_authority', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<Outcome>(page, 'seedPendingApprovalForTest');
        expect(seeded.ok, JSON.stringify(seeded)).toBe(true);
        const taskId = (seeded.record as { taskId: string }).taskId;

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, taskId);
        const card = detail(page).locator('[data-approval-id="appr_reload_approval"]');
        await expect(card).toBeVisible();
        await expect(card).toContainText('write_file');
        await expect(card).toContainText('tool:write_file');
        await expect(card.locator('time')).toHaveAttribute('datetime', new Date(1_794_283_800_000).toISOString());
        await expect(card).toContainText('Host: not connected');
        await expect(card).toContainText('This action needs host approval. Approve it in the local companion.');
        await expect(card.getByRole('button', { name: 'Approve', exact: true })).toBeDisabled();
        await expect(card.getByRole('button', { name: 'Deny', exact: true })).toBeDisabled();
        await expect(detail(page).locator('[data-task-detail-header]')).toContainText('Waiting for approval');
    });

    test('stop_is_active_only_and_delivery_errors_stay_message_scoped', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const started = await call<Outcome>(page, 'startTwoLiveTasksForTest');
        expect(started.ok, JSON.stringify(started)).toBe(true);
        const [task] = started.record as LiveTaskSeed[];

        await selectTask(page, task.taskId);
        const queued = await call<Outcome>(page, 'queueTaskMessageForTest', task.taskId, 'queued follow-up');
        expect(queued.ok, JSON.stringify(queued)).toBe(true);
        const messageId = (queued.record as { messageId: string }).messageId;

        const stop = detail(page).locator('.run-status').getByRole('button', { name: 'Stop', exact: true });
        await expect(stop).toBeEnabled();
        await stop.click();

        await expect(detail(page).locator('.run-status')).toContainText('Cancelled');
        await expect(detail(page).locator('.run-status').getByRole('button', { name: 'Stop', exact: true })).toHaveCount(0);
        await expect(detail(page).getByRole('button', { name: 'Pause', exact: true })).toHaveCount(0);
        const delivery = detail(page).locator('[data-timeline-kind="delivery"]');
        await expect(delivery).toHaveCount(1);
        await expect(delivery).toContainText(messageId);
        await expect(delivery).toContainText('was not delivered');
        await expect(detail(page).locator('[data-task-error="run"]')).toHaveCount(0);
        await expect(detail(page)).not.toContainText('message_not_delivered:');
    });

    test('retry_appears_only_for_retryable_failure_and_uses_the_real_runtime', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const started = await call<Outcome>(page, 'startTwoLiveTasksForTest');
        expect(started.ok, JSON.stringify(started)).toBe(true);
        const [task] = started.record as LiveTaskSeed[];

        const failed = await call<Outcome>(page, 'failLiveRunForTest', task.runId);
        expect(failed.ok, JSON.stringify(failed)).toBe(true);
        await selectTask(page, task.taskId);
        await expect(detail(page).locator('[data-task-error="run"]')).toContainText('model_unavailable');
        await expect(detail(page).locator('[data-task-error="run"]')).toContainText('synthetic retryable failure');
        const retry = detail(page).getByRole('button', { name: 'Retry', exact: true });
        await expect(retry).toBeVisible();
        await retry.click();

        await expect(detail(page).locator('.run-status')).toContainText('Running');
        await expect(detail(page).getByRole('button', { name: 'Retry', exact: true })).toHaveCount(0);
        const snapshot = (await call<Outcome>(page, 'taskSnapshotForTest', task.taskId))
            .record as LiveRunSnapshot;
        expect(snapshot.runId).not.toBe(task.runId);
        expect(snapshot.retryOfRunId).toBe(task.runId);
        expect(snapshot.runState).toBe('running');
    });

    test('the_gated_api_and_the_ui_share_one_runtime_owner_across_a_reload', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const fixture = await call<{ taskId: string; runId: string }>(page, 'createClientRun');
        expect(fixture.taskId).toMatch(/^tsk_[0-9a-f]{16}$/);
        expect(fixture.runId).toMatch(/^run_[0-9a-f]{16}$/);

        await expect(page.locator(`#studio-typed-root [data-task-id="${fixture.taskId}"]`)).toBeVisible();
        await selectTask(page, fixture.taskId);
        const text = await call<Outcome>(page, 'liveTextForTest', fixture.runId, 'shared owner replay text');
        expect(text.ok, JSON.stringify(text)).toBe(true);
        await expect(detail(page).locator('[data-timeline-kind="assistant"]')).toContainText('shared owner replay text');

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, fixture.taskId);
        await expect(detail(page).locator('[data-timeline-kind="assistant"]')).toContainText('shared owner replay text');
    });
});
