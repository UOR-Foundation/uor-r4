import { afterEach, describe, expect, it } from 'vitest';
// @ts-expect-error Node's filesystem API is available in the Vitest runtime but not in the browser-only type config.
import { readFileSync } from 'node:fs';

const stylesheet = readFileSync('index.css', 'utf8');

function installTheme(theme: 'dark' | 'light') {
    document.documentElement.dataset.theme = theme;
    const style = document.createElement('style');
    style.textContent = stylesheet;
    document.head.appendChild(style);
    document.body.innerHTML = '<div class="app-shell"><header class="view-head"></header><button class="primary-btn" type="button">Connect</button><button class="icon-btn" type="button">Settings</button><button class="ghost-btn" type="button">Cancel</button><article class="source-card"></article></div>';
    return style;
}

function declaration(style: HTMLStyleElement, selector: string): CSSStyleDeclaration {
    const rules = Array.from(style.sheet!.cssRules).filter((candidate) => candidate.type === 1) as CSSStyleRule[];
    const rule = rules.find((candidate) => candidate.selectorText === selector)
        || rules.find((candidate) => candidate.selectorText.split(',').some((part) => part.trim() === selector));
    if (!rule) throw new Error(`Missing rule ${selector}`);
    return rule.style;
}

afterEach(() => {
    document.querySelectorAll('style').forEach((node) => node.remove());
    document.body.innerHTML = '';
    delete document.documentElement.dataset.theme;
});

describe('Mac-inspired Studio visual system', () => {
    it('uses system type, soft radii, calm surfaces, and minimum native targets', () => {
        const style = installTheme('dark');
        const primary = document.querySelector<HTMLButtonElement>('.primary-btn')!;
        const icon = document.querySelector<HTMLButtonElement>('.icon-btn')!;
        const ghost = document.querySelector<HTMLButtonElement>('.ghost-btn')!;
        const card = document.querySelector<HTMLElement>('.source-card')!;
        expect(declaration(style, 'body').fontFamily).toMatch(/^-apple-system,/);
        expect(parseFloat(getComputedStyle(primary).borderRadius)).toBeGreaterThanOrEqual(8);
        expect(parseFloat(getComputedStyle(card).borderRadius)).toBeGreaterThanOrEqual(8);
        expect(parseFloat(getComputedStyle(primary).minBlockSize)).toBeGreaterThanOrEqual(24);
        expect(parseFloat(getComputedStyle(icon).minBlockSize)).toBeGreaterThanOrEqual(24);
        expect(parseFloat(getComputedStyle(icon).minInlineSize)).toBeGreaterThanOrEqual(24);
        expect(parseFloat(getComputedStyle(ghost).minBlockSize)).toBeGreaterThanOrEqual(24);
        expect(getComputedStyle(card).boxShadow).not.toBe('none');
        expect(getComputedStyle(document.querySelector<HTMLElement>('.app-shell')!).backgroundImage).not.toContain('linear-gradient');
    });

    it('preserves themed surfaces and visible keyboard focus', () => {
        const style = installTheme('dark');
        const darkSurface = declaration(style, 'html[data-theme="dark"]').getPropertyValue('--surface-1');
        expect(parseFloat(declaration(style, ':focus-visible').outline)).toBeGreaterThanOrEqual(2);
        document.querySelectorAll('style').forEach((node) => node.remove());
        const lightStyle = installTheme('light');
        expect(declaration(lightStyle, 'html[data-theme="light"]').getPropertyValue('--surface-1')).not.toBe(darkSurface);
        expect(declaration(lightStyle, '.view-head').background).not.toBe('');
    });

    it('keeps tab, attachment, and message action targets at least 24 pixels', () => {
        installTheme('dark');
        document.body.innerHTML += '<button class="editor-tab-close" type="button">×</button><button class="attachment-chip-remove" type="button">×</button><button class="msg-action" type="button">Copy</button>';
        for (const selector of ['.editor-tab-close', '.attachment-chip-remove', '.msg-action']) {
            const control = document.querySelector<HTMLButtonElement>(selector)!;
            expect(parseFloat(getComputedStyle(control).minBlockSize), selector).toBeGreaterThanOrEqual(24);
            expect(parseFloat(getComputedStyle(control).minInlineSize), selector).toBeGreaterThanOrEqual(24);
        }
    });
});
