import { describe, expect, it } from 'vitest';
import { foundation } from './main.js';

describe('foundation public entry', () => {
    it('exposes the foundation API object', () => {
        expect(foundation).toBeTypeOf('object');
    });

    it('does not mount or mutate the preview root without typed=1', () => {
        const root = document.createElement('div');
        root.id = 'studio-typed-root';
        root.hidden = false;
        document.body.appendChild(root);
        expect(root.hidden).toBe(false);
        expect(root.dataset.active).toBeUndefined();
        expect(root.querySelector('.studio-shell')).toBeNull();
    });
});
