import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const studioUiModule = '../../assets/js/studio_ui.js';
const modelSourcesModule = '../../assets/js/model_sources.js';
let unmount: (() => void) | undefined;

beforeEach(() => {
    vi.useFakeTimers();
    window.localStorage.clear();
    window.location.hash = '';
    window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false }));
    document.body.innerHTML = '<div id="app" data-nav="collapsed"><main id="mainColumn"><section id="modelView"><div id="modelSourcesPanel"><div class="providers-head"></div><div id="modelSourceMount"></div></div></section></main><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div></div>';
    unmount = undefined;
    vi.resetModules();
});

afterEach(() => {
    if (unmount) unmount();
    vi.clearAllTimers();
    vi.restoreAllMocks();
    vi.useRealTimers();
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioUi?: unknown }).__studioUi;
});

describe('URL artifact diagnostics after source regrouping', () => {
    it('reads the moved URL input instead of failing on the old mount', async () => {
        const { mountModelSources } = await import(modelSourcesModule);
        unmount = mountModelSources(document.getElementById('modelSourceMount')!).unmount;
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(450);
        const input = document.getElementById('modelUrlInput') as HTMLInputElement | null;
        expect(input).not.toBeNull();
        expect(document.querySelector('#artifactDiagnostics')!.contains(input)).toBe(true);
        document.querySelector<HTMLButtonElement>('[data-action="load-url"]')!.click();
        await vi.advanceTimersByTimeAsync(0);
        expect(document.body.textContent).toContain('Enter a model file URL.');
        expect(document.body.textContent).not.toContain('undefined');
        const status = document.getElementById('modelsVerifyRow')!;
        expect(status.getAttribute('role')).toBe('status');
        expect(status.getAttribute('aria-live')).toBe('polite');
        expect(status.textContent).toContain('Enter a model file URL.');
        expect(document.getElementById('modelsVerifyNote')!.textContent).toContain('Downloads, sizes, hashes and tokenizer bindings');
    });
});
