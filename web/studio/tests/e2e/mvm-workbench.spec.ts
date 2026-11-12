import { expect, test } from '@playwright/test';

const command = 'input[aria-label="MVM command"]';

test('mvm workbench has a real three-pane layout and executable terminal', async ({ page }) => {
    await page.goto('/?typed=1&studioTest=1');
    await expect(page.locator('[data-pane="explorer"]')).toBeVisible();
    await expect(page.locator('[data-pane="editor"]')).toBeVisible();
    await expect(page.locator('[data-pane="assistant"]')).toBeVisible();
    await expect(page.locator('[data-pane="terminal"]')).toBeVisible();
    await expect(page.locator('[data-testid="mvm-terminal"]')).toBeVisible();

    await page.locator(command).fill('mvm status');
    await page.locator(command).press('Enter');
    await expect(page.locator('[data-testid="mvm-terminal"]')).toContainText('riscv64-rv64imac');

    await page.evaluate(() => window.__studioMvmTest!.write('/workspace/src/bench.c', '#include <stdio.h>\nint main(void){printf("BENCH=%d\\n", 42);return 0;}'));
    await page.locator(command).fill('cc /workspace/src/bench.c -o /workspace/bench && /workspace/bench');
    await page.locator(command).press('Enter');
    await expect(page.locator('[data-testid="mvm-terminal"]')).toContainText('BENCH=42');
    await expect(page.locator('[data-pane="explorer"]')).toContainText('bench.c');

    await page.locator(command).fill('history');
    await page.locator(command).press('Enter');
    await expect(page.locator('[data-testid="mvm-terminal"]')).toContainText('cc /workspace/src/bench.c');
    await page.getByRole('button', { name: 'Clear terminal' }).click();
    await expect(page.locator('[data-testid="mvm-terminal"] pre')).toHaveText('');

    await page.getByRole('button', { name: 'Models', exact: true }).click();
    await page.getByRole('button', { name: 'Agent', exact: true }).click();
    await page.locator(command).fill('mvm status');
    await page.locator(command).press('Enter');
    await expect(page.locator('[data-testid="mvm-terminal"]')).toContainText('riscv64-rv64imac');
});

test('mvm responsive boundaries, IME and RTL preserve the workbench', async ({ page }) => {
    for (const width of [390, 1023, 1024, 1440]) {
        await page.setViewportSize({ width, height: 900 });
        await page.goto('/?typed=1&studioTest=1');
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await expect(page.locator('[data-pane="editor"]')).toBeVisible();
        await expect(page.locator('[data-pane="terminal"]')).toBeVisible();
        expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
        if (width === 1023) await expect(page.locator('[data-region="task-rail"]')).toBeHidden();
        if (width >= 1024) await expect(page.locator('[data-region="task-rail"]')).toBeVisible();
    }
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto('/?typed=1&studioTest=1');
    await expect(page.locator('[data-pane="explorer"]')).toHaveAttribute('data-open', 'false');
    await expect(page.locator('[data-pane="assistant"]')).toHaveAttribute('data-open', 'false');
    await page.getByRole('button', { name: 'Explorer', exact: true }).click();
    await expect(page.locator('[data-pane="explorer"]')).toHaveAttribute('data-open', 'true');
    await page.getByRole('button', { name: 'Explorer', exact: true }).click();
    await expect(page.locator('[data-pane="explorer"]')).toHaveAttribute('data-open', 'false');
    await page.getByRole('button', { name: 'Assistant', exact: true }).click();
    await expect(page.locator('[data-pane="assistant"]')).toHaveAttribute('data-open', 'true');
    await page.getByRole('button', { name: 'Assistant', exact: true }).click();
    await expect(page.locator('[data-pane="assistant"]')).toHaveAttribute('data-open', 'false');

    const input = page.locator(command);
    await input.fill('mvm status');
    await input.dispatchEvent('compositionstart');
    await input.press('Enter');
    await expect(page.locator('[data-testid="mvm-terminal"] pre')).toHaveText('');
    await input.dispatchEvent('compositionend');
    await input.press('Enter');
    await expect(page.locator('[data-testid="mvm-terminal"] pre')).toContainText('"architecture": "riscv64-rv64imac"');
    await page.evaluate(() => { document.documentElement.dir = 'rtl'; });
    await expect(page.locator('[data-pane="editor"]')).toBeVisible();
    await expect(page.locator('[data-pane="terminal"]')).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
});


test('mvm virtual git persists local staging and refuses unavailable rust tools', async ({ page }) => {
    await page.goto('/?typed=1&studioTest=1');
    await page.waitForFunction(() => window.__studioMvmTest !== undefined);
    await page.evaluate(() => window.__studioMvmTest!.write('/workspace/fix.txt', 'before\n'));
    const output = page.locator('[data-testid="mvm-terminal"] pre');
    const run = async (cmd: string, expected?: string) => {
        const input = page.locator(command);
        await expect(input).toBeEnabled();
        await input.fill(cmd);
        await input.press('Enter');
        if (expected) await expect(output).toContainText(expected);
    };

    await run('git init');
    await run('git add /workspace/fix.txt');
    await run('git commit -m "initial fixture"', 'local staging initial fixture');
    await page.evaluate(() => window.__studioMvmTest!.write('/workspace/fix.txt', 'after\n'));
    await run('git diff', '-before');
    await expect(output).toContainText('+after');
    await run('git add /workspace/fix.txt');
    await run('git commit -m "repair fixture"', 'local staging repair fixture');

    await page.reload();
    await run('git log --oneline', 'local staging initial fixture');
    await expect(output).toContainText('local staging repair fixture');
    await run('git status', 'working tree clean');
    await run('cargo check', 'no genuine guest Rust toolchain');
    await run('rustc --version', 'no genuine guest Rust toolchain');
});
