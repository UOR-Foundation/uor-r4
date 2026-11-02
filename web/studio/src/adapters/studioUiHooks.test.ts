import { afterAll, beforeAll, describe, expect, it, vi } from 'vitest';
import type { StudioUiApi } from './workflow.js';

let ui: StudioUiApi;
const legacyWindow = window as unknown as { __studioUi: StudioUiApi };
const counts: number[] = [];

beforeAll(async () => {
    window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
    legacyWindow.__studioUi = { getMode: () => 'api' } as unknown as StudioUiApi;
    document.body.innerHTML = '<div id="app"></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div>';
    document.addEventListener('uor-attachments-changed', (event) => {
        counts.push((event as CustomEvent<{ count: number }>).detail.count);
    });
    await import('../../assets/js/studio_ui.js');
    await vi.waitFor(() => expect(legacyWindow.__studioUi.addAttachment).toBeTypeOf('function'));
    counts.length = 0;
    ui = legacyWindow.__studioUi;
});

afterAll(() => {
    vi.clearAllTimers();
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioUi?: StudioUiApi }).__studioUi;
});

describe('actual studio UI attachment hooks', () => {
    it('emits attachment changes on add, remove, and clear', () => {
        ui.addAttachment({ name: 'a.md', content: 'a', source: 'editor' });
        ui.removeAttachment('a.md');
        ui.addAttachment({ name: 'b.md', content: 'b', source: 'editor' });
        ui.clearAttachments();
        expect(counts).toEqual([1, 0, 1, 0]);
        expect(ui.getAttachments()).toEqual([]);
    });
});
