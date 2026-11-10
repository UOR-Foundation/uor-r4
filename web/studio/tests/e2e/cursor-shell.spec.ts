import { test, expect } from '@playwright/test';
import { openTypedAt } from './support';

const darkTokens = {
    canvas: '#141413',
    surface: '#1c1b19',
    surfaceRaised: '#242320',
    border: '#33312c',
    text: '#f5f3ee',
    muted: '#a8a49a',
    accent: '#d97757',
    focus: '#f5f3ee',
    radiusSm: '4px',
    radiusMd: '8px',
};

const lightTokens = {
    canvas: '#faf9f5',
    surface: '#ffffff',
    surfaceRaised: '#f1efe8',
    border: '#d9d5c9',
    text: '#1f1e1a',
    muted: '#6b675e',
    accent: '#b85a3c',
    focus: '#1f1e1a',
    radiusSm: '4px',
    radiusMd: '8px',
};

async function uorTokens(page: import('@playwright/test').Page) {
    return page.locator('#studio-typed-root .task-workspace').evaluate((element) => {
        const style = getComputedStyle(element);
        const read = (name: string) => style.getPropertyValue(name).trim();
        return {
            canvas: read('--uor-canvas'),
            surface: read('--uor-surface'),
            surfaceRaised: read('--uor-surface-raised'),
            border: read('--uor-border'),
            text: read('--uor-text'),
            muted: read('--uor-muted'),
            accent: read('--uor-accent'),
            focus: read('--uor-focus'),
            radiusSm: read('--uor-radius-sm'),
            radiusMd: read('--uor-radius-md'),
        };
    });
}

function normalizeTokens(tokens: Record<string, string>) {
    return Object.fromEntries(Object.entries(tokens).map(([key, value]) => [key, value.replace(/^#([0-9a-f])([0-9a-f])([0-9a-f])$/i, (_match, red: string, green: string, blue: string) => `#${red}${red}${green}${green}${blue}${blue}`.toLowerCase())]));
}

test('renders_task_rail_and_transcript_first_detail_at_1440', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    const shell = root.locator('.task-workspace');
    const rail = root.locator('[data-region="task-rail"]');
    const detail = root.locator('main[data-region="task-detail"]');

    await expect(shell).toBeVisible();
    await expect(rail).toBeVisible();
    await expect(detail).toBeVisible();
    await expect(rail.locator('.project-sidebar')).toBeVisible();
    await expect(detail.locator('.agent-workspace')).toBeVisible();
    await expect(detail.locator('.agent-transcript')).toBeVisible();
    await expect(detail.locator('.agent-composer')).toBeVisible();

    const railBox = await rail.boundingBox();
    const detailBox = await detail.boundingBox();
    expect(railBox).not.toBeNull();
    expect(detailBox).not.toBeNull();
    expect(railBox!.width).toBeGreaterThan(200);
    expect(railBox!.x + railBox!.width).toBeLessThanOrEqual(detailBox!.x + 1);
    expect(detailBox!.width).toBeGreaterThan(railBox!.width);

    await expect(root.locator('h1:visible')).toHaveCount(1);
    await expect(root.locator('h1:visible')).toHaveText('Agent');
});

test('uses_uor_dark_tokens_and_no_green_placeholder_treatment', async ({ page }) => {
    await openTypedAt(page, 1440);
    expect(normalizeTokens(await uorTokens(page))).toEqual(darkTokens);
    const shellStyle = await page.locator('#studio-typed-root .task-workspace').evaluate((element) => {
        const style = getComputedStyle(element);
        return { background: style.backgroundColor, color: style.color, classes: element.className };
    });
    expect(shellStyle.background).toBe('rgb(20, 20, 19)');
    expect(shellStyle.classes.split(/\s+/)).toEqual(['harness-shell', 'studio-shell', 'task-workspace']);
    const values = Object.values(normalizeTokens(await uorTokens(page))).map((value) => value.toLowerCase());
    for (const green of ['#111512', '#1a211b', '#8fd3a7']) expect(values).not.toContain(green);
});

test('uses_uor_light_tokens', async ({ page }) => {
    await openTypedAt(page, 1440, 900, 'light');
    expect(normalizeTokens(await uorTokens(page))).toEqual(lightTokens);
    const background = await page.locator('#studio-typed-root .task-workspace').evaluate((element) => getComputedStyle(element).backgroundColor);
    expect(background).toBe('rgb(250, 249, 245)');
});

test('project_home_shows_project_environment_and_empty_tasks_without_runtime_claim', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    const home = root.locator('[data-region="task-rail"] section[aria-label="Project home"]');
    await expect(home).toBeVisible();
    await expect(home).toContainText('uor-r4');
    await expect(home).toContainText('Local browser');
    await expect(home).toContainText('No tasks yet');
    await expect(root).not.toContainText('runtime_unavailable');
    await expect(root).not.toContainText('runtime is not initialized');
});

test('work_surface_region_appears_for_the_split_pane', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    await expect(root.locator('[data-region="work-surface"]')).toHaveCount(0);
    await root.locator('[data-action="split-pane"]').click();
    await expect(root.locator('.pane-secondary [data-region="work-surface"]')).toBeVisible();
    await root.locator('[data-action="close-split"]').click();
    await expect(root.locator('[data-region="work-surface"]')).toHaveCount(0);
});

test('each_typed_view_keeps_exactly_one_h1', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    const navigation = root.locator('nav[aria-label="Studio"]');
    for (const label of ['Agent', 'Workspace', 'Changes', 'Terminal', 'Models', 'Guide']) {
        await navigation.getByRole('button', { name: label, exact: true }).click();
        await expect(root.locator('h1:visible'), `${label} must keep exactly one h1`).toHaveCount(1);
    }
});

test('390_has_no_horizontal_overflow', async ({ page }) => {
    await openTypedAt(page, 390, 852);
    const root = page.locator('#studio-typed-root');
    const rail = root.locator('[data-region="task-rail"]');
    await expect(rail).toBeHidden();
    await expect(root.locator('main[data-region="task-detail"]')).toBeVisible();
    await expect(root.locator('.agent-composer')).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(0);
    expect(await page.evaluate(() => document.body.scrollWidth - document.body.clientWidth)).toBeLessThanOrEqual(0);
    await root.locator('[data-action="toggle-projects"]').click();
    await expect(rail).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(0);
    expect(await page.evaluate(() => document.body.scrollWidth - document.body.clientWidth)).toBeLessThanOrEqual(0);
});

test('1023_rail_defaults_closed_and_reopens', async ({ page }) => {
    await openTypedAt(page, 1023);
    const root = page.locator('#studio-typed-root');
    const rail = root.locator('[data-region="task-rail"]');
    const toggle = root.locator('[data-action="toggle-projects"]');
    await expect(rail).toBeHidden();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await toggle.click();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await toggle.click();
    await expect(rail).toBeHidden();
    await toggle.click();
    await expect(rail).toBeVisible();
});

test('1024_rail_defaults_open_and_manual_close_stays_closed', async ({ page }) => {
    await openTypedAt(page, 1024);
    const root = page.locator('#studio-typed-root');
    const rail = root.locator('[data-region="task-rail"]');
    const toggle = root.locator('[data-action="toggle-projects"]');
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await toggle.click();
    await expect(rail).toBeHidden();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await page.waitForTimeout(200);
    await expect(rail).toBeHidden();
    await toggle.click();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
});
