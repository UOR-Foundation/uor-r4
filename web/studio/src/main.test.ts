import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { foundation } from './main.js';

const legacyWindow = window as unknown as {
    __studioUi?: unknown;
    __studioFiles?: unknown;
    __studioApp?: unknown;
};

describe('foundation public entry', () => {
    beforeEach(() => {
        vi.resetModules();
        window.history.replaceState(null, '', '');
        delete legacyWindow.__studioUi;
        delete legacyWindow.__studioFiles;
        delete legacyWindow.__studioApp;
    });

    afterEach(() => {
        vi.useRealTimers();
        vi.restoreAllMocks();
        vi.unstubAllGlobals();
        window.history.replaceState(null, '', '');
    });

    it('exposes the foundation API object', () => {
        expect(foundation).toBeTypeOf('object');
    });

    it('does not mount or mutate a pre-existing preview root without typed=1', async () => {
        const root = document.createElement('div');
        root.id = 'studio-typed-root';
        root.hidden = true;
        document.body.appendChild(root);
        await import('./main.js');
        expect(root.hidden).toBe(true);
        expect(root.dataset.active).toBeUndefined();
        expect(root.querySelector('.studio-shell')).toBeNull();
    });

    it('shows initialization errors visibly in the preview root', async () => {
        const { showMountError } = await import('./main.js');
        const root = document.createElement('div');
        root.id = 'studio-typed-root';
        root.hidden = true;
        document.body.appendChild(root);
        showMountError(root, new Error('init failed'));
        expect(root.hidden).toBe(false);
        expect(root.dataset.active).toBe('true');
        expect(root.textContent).toContain('The typed Studio could not start');
        expect(root.textContent).toContain('init failed');
        expect(getComputedStyle(root).display).not.toBe('none');
    });
});
