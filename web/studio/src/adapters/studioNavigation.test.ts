import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { StudioUiApi } from './workflow.js';

const studioUiModule = '../../assets/js/studio_ui.js';

beforeEach(() => {
    vi.useFakeTimers();
    window.localStorage.clear();
    window.location.hash = '';
    window.matchMedia = vi.fn((query: string) => ({ matches: query === '(max-width: 900px)', addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
    document.body.innerHTML = '<div id="app" data-nav="collapsed"><header><button id="btnMobileNav" aria-expanded="false">Menu</button></header><div class="shell-body"><aside id="navSidebar"><button id="navAgent" data-section="agent">Agent</button><button id="navWorkspace" data-section="workspace">Workspace</button></aside><button id="btnNavScrim" aria-label="Close navigation" hidden></button><main id="mainColumn"><button id="viewBtnChat">Chat</button><button id="viewBtnModel">Models</button><button id="viewBtnEditor">Workspace</button></main></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div><textarea id="editorText"></textarea></div>';
    vi.resetModules();
});

afterEach(() => {
    vi.clearAllTimers();
    vi.restoreAllMocks();
    vi.useRealTimers();
    document.body.innerHTML = '';
    document.documentElement.classList.remove('nav-open');
    window.localStorage.clear();
    delete (window as unknown as { __studioUi?: StudioUiApi }).__studioUi;
});

describe('actual mobile navigation dismissal', () => {
    it('coordinates scrim, ARIA, section, and Escape dismissal', async () => {
        await import(studioUiModule);
        if (document.readyState === 'loading') document.dispatchEvent(new Event('DOMContentLoaded'));
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const app = document.getElementById('app')!;
        const menu = document.getElementById('btnMobileNav')!;
        const scrim = document.getElementById('btnNavScrim')!;
        const main = document.getElementById('mainColumn')!;
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(scrim.hidden).toBe(true);
        menu.click();
        expect(app.getAttribute('data-nav')).toBe('expanded');
        expect(menu.getAttribute('aria-expanded')).toBe('true');
        expect(scrim.hidden).toBe(false);
        expect(main.inert).toBe(true);
        scrim.click();
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
        menu.click();
        document.getElementById('navWorkspace')!.click();
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
        menu.click();
        expect(scrim.hidden).toBe(false);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(menu.getAttribute('aria-expanded')).toBe('false');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
    });
});
