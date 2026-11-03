import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const studioUiModule = '../../assets/js/studio_ui.js';
const addDocumentListener = document.addEventListener;
let keydownListeners: Array<{ listener: EventListenerOrEventListenerObject; options?: boolean | AddEventListenerOptions }>;

beforeEach(() => {
    keydownListeners = [];
    vi.spyOn(document, 'addEventListener').mockImplementation((type: string, listener: EventListenerOrEventListenerObject, options?: boolean | AddEventListenerOptions) => {
        if (type === 'keydown') keydownListeners.push({ listener, options });
        addDocumentListener.call(document, type, listener, options);
    });
    vi.useFakeTimers();
    window.localStorage.clear();
    window.location.hash = '';
    window.matchMedia = vi.fn((query: string) => query === '(max-width: 900px)' ? { matches: true, addEventListener: vi.fn() } : { matches: false, addEventListener: vi.fn() }) as unknown as typeof window.matchMedia;
    vi.spyOn(Element.prototype, 'getClientRects').mockImplementation(() => [new DOMRect()] as unknown as DOMRectList);
    Element.prototype.scrollIntoView = vi.fn();
    document.body.innerHTML = '<div id="app" data-nav="collapsed"><header><button id="btnMobileNav" aria-expanded="false">Menu</button></header><aside id="navSidebar"><button class="nav-item" id="navAgent" data-section="agent">Agent</button><button class="nav-item" id="navWorkspace" data-section="workspace">Workspace</button><button class="nav-item" id="navModels" data-section="models">Connections</button></aside><button id="btnNavScrim" hidden></button><main id="mainColumn"><input id="chatInput"><section id="modelView" data-view="model"></section></main></div><div id="settingsDialog" class="modal-backdrop" hidden><button id="btnCloseSettings">Close</button></div><div id="helpDialog" class="modal-backdrop" hidden><button id="btnCloseHelp">Close help</button></div><div id="commandPalette" hidden><input id="paletteInput"><div id="paletteList"></div></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div>';
    vi.resetModules();
});

afterEach(() => {
    keydownListeners.forEach(({ listener, options }) => document.removeEventListener('keydown', listener, options));
    vi.clearAllTimers();
    vi.restoreAllMocks();
    vi.useRealTimers();
    document.body.innerHTML = '';
    document.documentElement.classList.remove('nav-open');
    window.localStorage.clear();
});

describe('overlay and view-state coordination', () => {
    it('keeps overlays exclusive, traps palette Tab, and closes the top overlay first', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const ui = (window as unknown as { __studioUi: { openSettings(): void; openPalette(): void } }).__studioUi;
        const menu = document.getElementById('btnMobileNav')!;
        const settings = document.getElementById('settingsDialog')!;
        const palette = document.getElementById('commandPalette')!;
        menu.click();
        ui.openSettings();
        expect(settings.hidden).toBe(false);
        ui.openPalette();
        expect(settings.hidden).toBe(true);
        expect(palette.hidden).toBe(false);
        ui.openSettings();
        expect(palette.hidden).toBe(true);
        expect(settings.hidden).toBe(false);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(settings.hidden).toBe(true);
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('expanded');
        ui.openPalette();
        const input = document.getElementById('paletteInput') as HTMLInputElement;
        const commands = document.querySelectorAll<HTMLButtonElement>('#paletteList button');
        const lastCommand = commands[commands.length - 1];
        lastCommand.focus();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(input);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(lastCommand);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(palette.hidden).toBe(true);
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('expanded');
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('collapsed');
        expect(document.activeElement).toBe(menu);
    });

    it('synchronizes nav current state for programmatic view changes', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        document.dispatchEvent(new CustomEvent('uor-view-changed', { detail: { view: 'model' } }));
        expect(document.getElementById('navModels')!.getAttribute('aria-current')).toBe('page');
        expect(document.getElementById('navAgent')!.getAttribute('aria-current')).toBe('false');
        document.dispatchEvent(new CustomEvent('uor-view-changed', { detail: { view: 'workspace' } }));
        expect(document.getElementById('navWorkspace')!.getAttribute('aria-current')).toBe('page');
        expect(document.getElementById('navModels')!.getAttribute('aria-current')).toBe('false');
        document.dispatchEvent(new CustomEvent('uor-view-changed', { detail: { view: 'chat' } }));
        expect(document.getElementById('navAgent')!.getAttribute('aria-current')).toBe('page');
    });

    it('keeps real labels available and prevents boot autofocus', async () => {
        // @ts-expect-error Node's filesystem API is available in the Vitest runtime.
        const { readFileSync } = await import('node:fs');
        const html = readFileSync('index.html', 'utf8');
        expect(html).toMatch(/class="sr-only" for="chatInput"/);
        expect(html).toMatch(/class="sr-only" for="paletteInput"/);
        expect(html).toContain("setView('chat', { focus: false })");
        expect(html).not.toContain('under Model sources');
        expect(html).not.toContain("when Claude's API endpoint is available");
        const readme = readFileSync('README.md', 'utf8');
        expect(readme).not.toContain('**Models** is four labeled sections');
    });

    it('closes other modals and blocks background navigation shortcuts', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const ui = (window as unknown as { __studioUi: { openSettings(): void; openHelp(): void } }).__studioUi;
        const settings = document.getElementById('settingsDialog')!;
        const help = document.getElementById('helpDialog')!;
        ui.openSettings();
        ui.openHelp();
        expect(settings.hidden).toBe(true);
        expect(help.hidden).toBe(false);
        ui.openSettings();
        expect(help.hidden).toBe(true);
        expect(settings.hidden).toBe(false);
        await vi.advanceTimersByTimeAsync(20);
        expect(document.activeElement?.id).toBe('btnCloseSettings');
        document.body.dataset.view = 'model';
        document.dispatchEvent(new KeyboardEvent('keydown', { key: '1', metaKey: true, bubbles: true, cancelable: true }));
        expect(document.body.dataset.view).toBe('model');
        expect(document.activeElement?.id).not.toBe('chatInput');
        ui.openHelp();
        expect(settings.hidden).toBe(true);
        expect(help.hidden).toBe(false);
        await vi.advanceTimersByTimeAsync(20);
        expect(document.activeElement?.id).toBe('btnCloseHelp');
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(help.hidden).toBe(true);
        expect(document.querySelectorAll('.modal-backdrop:not([hidden])')).toHaveLength(0);
    });

    it('restores the opener after switching overlays and closes transient menus', async () => {
        const rows = document.createElement('div');
        rows.id = 'sessionRowsV2';
        document.body.appendChild(rows);
        (window as unknown as { __studioUi: { listSessions(): unknown[] } }).__studioUi = {
            listSessions: () => [{ id: 's1', title: 'Session', createdAt: Date.now(), updatedAt: Date.now(), messageCount: 1, preview: 'preview' }],
        };
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const ui = (window as unknown as { __studioUi: { openSettings(): void; openPalette(): void; openHelp(): void } }).__studioUi;
        const opener = document.createElement('button');
        opener.id = 'btnSettings';
        document.body.appendChild(opener);
        opener.focus();
        document.querySelector<HTMLButtonElement>('.session-row .conv-menu')!.click();
        expect(document.querySelector('.session-popover')).not.toBeNull();
        const templates = document.createElement('div');
        templates.id = 'templateMenu';
        document.body.appendChild(templates);
        ui.openSettings();
        ui.openPalette();
        expect(document.querySelector('.session-popover')).toBeNull();
        expect(templates.hidden).toBe(true);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(opener);
        opener.focus();
        ui.openSettings();
        ui.openHelp();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(opener);
    });
});
