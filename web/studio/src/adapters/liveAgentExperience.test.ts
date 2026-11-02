import { beforeEach, describe, expect, it, vi } from 'vitest';
import source from '../../index.html?raw';

function section(start: string, end: string): string {
    const from = source.indexOf(start);
    const to = source.indexOf(end, from);
    if (from < 0 || to < 0) throw new Error(`Missing live Agent section ${start}`);
    return source.slice(from, to);
}

function pageHooks() {
    const app = {
        mode: 'wasm',
        caps: null as null | { schema: string },
        apiConfig: null as null | { baseUrl: string },
        activePredictionRequestId: null as string | null,
        activeGenerationId: 7 as number | null,
        generating: false,
    };
    const conversation = { messages: [] as Array<Record<string, unknown>>, sourceKind: '' };
    const factory = new Function(
        'app', 'document', 'window', 'toast', 'setView', 'ensureConversation', 'touchConversation',
        'appendMessageElement', 'renderConversationList', 'scrollChat', 'sendViaWorker', 'sendViaApi',
        'autoGrowTextarea', 'startNewChat', '$',
        `${section('function isDiscreteCapabilities', 'function setPredictorMode')}
         ${section('function chatReadiness', '/* ------------------------------------------------------------ metrics */')}
         ${section('function renderEmptyState', 'function listItem')}
         ${section('function sendFromComposer', '/* ------------------------------------------------------------ model sources */')}
         return { updateComposerState, renderEmptyState, sendFromComposer };`,
    );
    const hooks = factory(
        app, document, window, vi.fn(), vi.fn(), () => conversation, vi.fn(), vi.fn(() => null), vi.fn(), vi.fn(),
        vi.fn(() => true), vi.fn(() => true), vi.fn(), vi.fn(() => true), (selector: string) => document.querySelector(selector),
    ) as {
        updateComposerState(): void;
        renderEmptyState(container: HTMLElement): void;
        sendFromComposer(prompt?: string): number | false;
    };
    return { app, hooks };
}

function sendButton(): HTMLButtonElement {
    return document.querySelector<HTMLButtonElement>('#btnSend')!;
}

describe('live Agent source readiness', () => {
    beforeEach(() => {
        document.body.innerHTML = '<button id="btnSend"></button><button id="btnStop"></button><textarea id="chatInput"></textarea><div id="chatMessages"></div>';
    });

    it('disables Send with no source and refuses the shared send path', () => {
        const { hooks } = pageHooks();
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(true);
        expect(sendButton().title).toContain('Models');
        expect(hooks.sendFromComposer('hello')).toBe(false);
    });

    it('disables chat for discrete M1 without disabling its prediction path', () => {
        const { app, hooks } = pageHooks();
        app.caps = { schema: 'uor-r4.lut-discrete-trunk/1' };
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(true);
        expect(hooks.sendFromComposer('hello')).toBe(false);
    });

    it('enables labeled synthetic continuation for a supported non-discrete stack', () => {
        const { app, hooks } = pageHooks();
        app.caps = { schema: 'uor-r4.lut-stack/7' };
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(false);
        expect(hooks.sendFromComposer('hello')).toBe(7);
    });

    it('enables API chat from an HTTP(S) config without a reachability claim', () => {
        const { app, hooks } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(false);
        expect(hooks.sendFromComposer('hello')).toBe(7);
    });

    it('renders distinct unavailable, API-ready, and local-ready empty states', () => {
        const { app, hooks } = pageHooks();
        const container = document.querySelector<HTMLElement>('#chatMessages')!;
        hooks.renderEmptyState(container);
        expect(container.textContent).toContain('Build with your files. Connect an agent when ready.');
        expect(container.textContent).toContain('No conversational model is configured');
        expect(container.textContent).toContain('Explore demo workspace');
        container.innerHTML = '';
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        hooks.renderEmptyState(container);
        expect(container.textContent).toContain('Agent connected — external baseline mode.');
        expect(container.textContent).not.toContain('No conversational model is configured');
        container.innerHTML = '';
        app.mode = 'wasm';
        app.caps = { schema: 'uor-r4.lut-stack/7' };
        hooks.renderEmptyState(container);
        expect(container.textContent).toContain('Local synthetic continuation ready.');
        expect(container.textContent).not.toContain('No conversational model is configured');
    });
});
