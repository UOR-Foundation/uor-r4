import { expect, test, type Page } from '@playwright/test';
import { openTyped } from './support';

async function blurActive(page: Page) {
    await page.evaluate(() => {
        const active = document.activeElement;
        if (active instanceof HTMLElement) active.blur();
    });
}

test('send_modes_stay_disabled_without_a_task_runtime', async ({ page }) => {
    await openTyped(page);
    const root = page.locator('#studio-typed-root');
    await expect(root.getByRole('button', { name: 'Send', exact: true })).toHaveCount(1);
    const modes = root.getByRole('group', { name: 'Run message modes' });
    for (const name of ['Queue', 'Steer', 'Force send', 'Stop run']) {
        const control = modes.getByRole('button', { name, exact: true });
        await expect(control, `${name} must be visible`).toBeVisible();
        await expect(control, `${name} must be disabled without a selected task run`).toBeDisabled();
    }
});

test('scoped_palette_shortcut_matches_the_toolbar_hint', async ({ page }) => {
    await openTyped(page);
    const root = page.locator('#studio-typed-root');
    const command = root.locator('.toolbar-command');
    await expect(command).toHaveAttribute('data-shortcut', '⌘⇧P');
    await expect(command).toHaveAttribute('aria-keyshortcuts', 'Meta+Shift+P Control+Shift+P');
    await blurActive(page);
    await page.keyboard.press('ControlOrMeta+Shift+P');
    const palette = page.getByRole('dialog', { name: 'Command palette' });
    await expect(palette).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(palette).toBeHidden();
});

test('scoped_settings_shortcut_opens_the_typed_settings', async ({ page }) => {
    await openTyped(page);
    await blurActive(page);
    await page.keyboard.press('Control+,');
    await expect(page.getByRole('dialog', { name: 'Studio settings', exact: true })).toBeVisible();
});

test('scoped_shortcuts_are_ignored_inside_the_prompt', async ({ page }) => {
    await openTyped(page);
    const prompt = page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
    await prompt.focus();
    await page.keyboard.press('Control+Shift+P');
    await expect(page.locator('dialog')).toHaveCount(0);
    await expect(prompt).toBeFocused();
    await page.keyboard.press('Control+,');
    await expect(page.locator('dialog')).toHaveCount(0);
});

test('tab_moves_focus_normally_between_toolbar_actions', async ({ page }) => {
    await openTyped(page);
    const root = page.locator('#studio-typed-root');
    await root.locator('.toolbar-command').focus();
    await page.keyboard.press('Tab');
    await expect(root.locator('.session-toolbar-actions button[aria-label="Settings"]')).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(root.locator('.session-toolbar-actions button[aria-label="Inspector"]')).toBeFocused();
});

test('escape_closes_the_task_menu_and_returns_focus_to_the_prompt', async ({ page }) => {
    await openTyped(page);
    const root = page.locator('#studio-typed-root');
    const prompt = root.locator('textarea[aria-label="Prompt"]');
    await prompt.fill('/');
    await expect(root.locator('[role="listbox"][aria-label="Build task commands"]')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(root.locator('[role="listbox"][aria-label="Build task commands"]')).toHaveCount(0);
    await expect(prompt).toBeFocused();
    await expect(prompt).toHaveValue('/');
});

test('cmd_k_does_not_open_the_hidden_legacy_palette', async ({ page }) => {
    await openTyped(page);
    await blurActive(page);
    await page.keyboard.press('ControlOrMeta+k');
    await page.waitForTimeout(100);
    await expect(page.locator('#commandPalette')).toBeHidden();
    await expect(page.locator('dialog')).toHaveCount(0);
});

test('typed_mode_suppresses_both_palette_hashes', async ({ page }) => {
    for (const hash of ['#command-palette', '#palette']) {
        await page.goto(`/?typed=1${hash}`);
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await page.waitForTimeout(250);
        expect(new URL(page.url()).hash).toBe('');
        await expect(page.locator('#commandPalette')).toBeHidden();
    }
});

test('explicit_legacy_mode_keeps_the_legacy_palette_route', async ({ page }) => {
    await page.goto('/?legacy=1#command-palette');
    await expect(page.locator('#app')).toBeVisible();
    await expect(page.locator('#studio-typed-root')).toBeHidden();
    await expect(page.locator('#commandPalette')).toBeVisible({ timeout: 2000 });
});
