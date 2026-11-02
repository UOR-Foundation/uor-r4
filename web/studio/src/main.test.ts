import { describe, expect, it } from 'vitest';
import { foundation } from './main.js';

describe('foundation public entry', () => {
    it('exposes the foundation API object', () => {
        expect(foundation).toBeTypeOf('object');
    });
});
