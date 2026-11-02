import { beforeEach, describe, expect, it, vi } from 'vitest';
import source from '../../index.html?raw';

function section(start: string, end: string): string {
    const from = source.indexOf(start);
    const to = source.indexOf(end, from);
    if (from < 0 || to < 0) throw new Error(`Missing page hook section ${start}`);
    return source.slice(from, to);
}

function pageHooks() {
    const app = {
        worker: { postMessage: vi.fn() },
        workerState: 'ready',
        caps: { schema: 'uor-r4.lut-discrete-trunk/1', shape: { vocab: 17, context: 64 } },
        tokenizerArtifact: { sentinel: true },
        generationSerial: 0,
        activeGenerationId: null as number | null,
        chatGenerationId: null as number | null,
        activeChatRequestId: null as string | number | null,
        chatRequests: {} as Record<string, number>,
        activePredictionRequestId: null as string | null,
        predictionRequests: {},
        generating: false,
        mode: 'wasm',
        activeGenerationKind: 'wasm',
        apiConfig: null as null | { baseUrl: string },
        abortController: null as AbortController | null,
        streamingBodyEl: null,
        assistantBuffer: '',
        lastWorkerError: '',
    };
    interface CapturedApiRequest {
        onDone(result: { stopped?: boolean }): void;
        onError(error: Error): void;
    }
    const conversation = { id: 'conversation', messages: [] as Array<Record<string, unknown>>, sourceKind: '' };
    let currentConversation: typeof conversation | null = conversation;
    const apiRequests: CapturedApiRequest[] = [];
    let requestSequence = 0;
    const factory = new Function(
        'app', 'document', 'window', 'toast', 'setMetrics', 'setEngineStatus', 'encodeTextWithArtifact', 'fillPredictorSha', '$',
        'activeConversation', 'appendMessageElement', 'touchConversation', 'updateComposerState', 'requestCheckpoint',
        'renderConversationList', 'scrollChat', 'formatTps', 'setView', 'makeId', 'streamChatCompletions', 'configFromSources', 'MAX_NEW_TOKENS',
        'ensureConversation', 'autoGrowTextarea',
        `${section('function isDiscreteCapabilities', 'function setPredictorMode')}
         ${section('function renderDiscretePrediction', 'function runPrediction')}
         ${section('function runPrediction', '/* ------------------------------------------------------------ status & toasts */')}
         ${section('function nextGenerationId', 'function generationTokenLimit')}
         ${section('function generationTokenLimit', '/* ------------------------------------------------------------ composer */')}
         ${section('function sendFromComposer', '/* ------------------------------------------------------------ model sources */')}
         ${section('function onWorkerMessage', 'function activeAssistantElement')}
         return { runPrediction, renderDiscretePrediction, onWorkerMessage, sendFromComposer, sendViaWorker, sendViaApi, finishLocalGeneration, finishGenerationWithError };`,
    );
    const hooks = factory(
        app, document, window, vi.fn(), vi.fn(), vi.fn(), vi.fn(() => [1, 2]), vi.fn(), (selector: string) => document.querySelector(selector),
        () => currentConversation, vi.fn(() => null), vi.fn(), vi.fn(), vi.fn(), vi.fn(), vi.fn(), vi.fn(() => '1'), vi.fn(),
        () => `gen-${++requestSequence}`, vi.fn(async (options: CapturedApiRequest) => { apiRequests.push(options); }), vi.fn(() => null), 32,
        () => conversation, vi.fn(),
    ) as {
        runPrediction(prompt?: string): boolean;
        renderDiscretePrediction(message: unknown): void;
        onWorkerMessage(event: { data: Record<string, unknown> }): void;
        sendFromComposer(prompt?: string): number | false;
        sendViaWorker(prompt: string, target: typeof conversation): boolean;
        sendViaApi(prompt: string, target: typeof conversation): Promise<void>;
        finishLocalGeneration(message: { id?: string; fullText?: string; tokenCount?: number }): void;
        finishGenerationWithError(error: string, requestId?: string | number, generationId?: number | null): boolean | void;
    };
    return { app, hooks, conversation, apiRequests, clearConversation: () => { currentConversation = null; } };
}

function events(type: string, listener: (event: CustomEvent) => void) {
    document.addEventListener(type, listener as EventListener);
    return () => document.removeEventListener(type, listener as EventListener);
}

describe('actual page prediction hooks', () => {
    beforeEach(() => { document.body.innerHTML = ''; });

    it('ignores a diagnostic result and completes only the matching prediction request', () => {
        const { app, hooks } = pageHooks();
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.runPrediction('hello')).toBe(true);
        const request = app.worker.postMessage.mock.calls[0][0];
        expect(request.action).toBe('predict_discrete');
        hooks.onWorkerMessage({ data: { action: 'discrete_prediction_result', id: 'diagnostic-predict', top8: [1], decoded: [] } });
        expect(completed).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(1);
        hooks.onWorkerMessage({ data: { action: 'discrete_prediction_result', id: request.id, top8: [1], decoded: [] } });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'complete', generationId: 1 });
        expect(app.activeGenerationId).toBeNull();
        dispose();
    });

    it('renders diagnostic results without completing the typed prediction', () => {
        const { app, hooks } = pageHooks();
        document.body.innerHTML = '<div id="predictorResult"></div><ol id="predictorTop"></ol><span id="predictorPrediction"></span><span id="predictorMs"></span><span id="predictorStatus"></span>';
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.runPrediction('hello')).toBe(true);
        hooks.onWorkerMessage({ data: { action: 'discrete_prediction_result', id: 'diagnostic-predict', top8: [7], decoded: [{ id: 7, text: 'x' }] } });
        expect(completed).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(1);
        expect(document.documentElement.dataset.predictorTop).toBe('[7]');
        expect(document.documentElement.dataset.predictorTopCount).toBe('1');
        expect(document.documentElement.dataset.predictorDecoded).toBe('[{"id":7,"text":"x"}]');
        expect(document.getElementById('predictorTop')?.textContent).toContain('id 7');
        dispose();
    });

    it('refuses a chat send while prediction is in flight', () => {
        const { app, hooks } = pageHooks();
        app.activePredictionRequestId = 'predict-existing';
        app.activeGenerationId = 7;
        expect(hooks.sendFromComposer('chat')).toBe(false);
        expect(app.activeGenerationId).toBe(7);
        expect(app.worker.postMessage).not.toHaveBeenCalled();
    });

    it('refuses prediction while a chat generation is active', () => {
        const { app, hooks } = pageHooks();
        app.generating = true;
        app.activeGenerationId = 42;
        app.generationSerial = 42;
        expect(hooks.runPrediction('hello')).toBe(false);
        expect(app.worker.postMessage).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(42);
    });

    it('ignores late chat completion and error while prediction is active', () => {
        const { app, hooks } = pageHooks();
        app.activePredictionRequestId = 'predict-existing';
        app.activeGenerationId = 7;
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        hooks.finishLocalGeneration({ fullText: 'late', tokenCount: 1 });
        hooks.finishGenerationWithError('late error');
        expect(completed).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(7);
        dispose();
    });

    it('returns a correlated worker error without completing the active prediction', () => {
        const { app, hooks } = pageHooks();
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.runPrediction('hello')).toBe(true);
        const request = app.worker.postMessage.mock.calls[0][0];
        hooks.onWorkerMessage({ data: { action: 'operation_error', id: 'diagnostic-predict', error: 'unrelated' } });
        expect(completed).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(1);
        hooks.onWorkerMessage({ data: { action: 'operation_error', id: request.id, error: 'context full' } });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'error', generationId: 1, error: 'context full' });
        expect(app.activeGenerationId).toBeNull();
        dispose();
    });

    it('ignores stale and duplicate local terminals while a newer chat is active', () => {
        const { app, hooks, conversation } = pageHooks();
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.sendViaWorker('older', conversation)).toBe(true);
        const older = app.worker.postMessage.mock.calls[0][0];
        app.generating = false;
        expect(hooks.sendViaWorker('newer', conversation)).toBe(true);
        const newer = app.worker.postMessage.mock.calls[1][0];
        expect(app.chatGenerationId).toBe(2);
        hooks.onWorkerMessage({ data: { action: 'generate_complete', id: older.id, fullText: 'stale', tokenCount: 1 } });
        hooks.onWorkerMessage({ data: { action: 'generate_error', id: older.id, error: 'stale error' } });
        expect(completed).not.toHaveBeenCalled();
        expect(app.generating).toBe(true);
        expect(app.chatGenerationId).toBe(2);
        expect(app.activeChatRequestId).toBe(newer.id);
        hooks.onWorkerMessage({ data: { action: 'generate_complete', id: newer.id, fullText: 'current', tokenCount: 1 } });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'complete', generationId: 2 });
        expect(app.activeChatRequestId).toBeNull();
        hooks.onWorkerMessage({ data: { action: 'generate_complete', id: newer.id, fullText: 'duplicate', tokenCount: 1 } });
        expect(completed).toHaveBeenCalledTimes(1);
        dispose();
    });

    it('returns false when the local worker cannot start a composer send', () => {
        const { app, hooks } = pageHooks();
        app.worker = null as unknown as typeof app.worker;
        expect(hooks.sendFromComposer('chat')).toBe(false);
        expect(app.activeGenerationId).toBeNull();
        expect(app.chatGenerationId).toBeNull();
    });

    it('dispatches a matching local terminal when no conversation is active', () => {
        const { app, hooks, conversation, clearConversation } = pageHooks();
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.sendViaWorker('chat', conversation)).toBe(true);
        const request = app.worker.postMessage.mock.calls[0][0];
        clearConversation();
        hooks.onWorkerMessage({ data: { action: 'generate_complete', id: request.id, fullText: 'done', tokenCount: 1 } });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'complete', generationId: 1 });
        expect(app.activeChatRequestId).toBeNull();
        expect(app.generating).toBe(false);
        dispose();
    });

    it('ignores stale API completion and error callbacks', async () => {
        const { app, hooks, conversation, apiRequests } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        await hooks.sendViaApi('older', conversation);
        app.generating = false;
        await hooks.sendViaApi('newer', conversation);
        expect(apiRequests).toHaveLength(2);
        apiRequests[0].onDone({ stopped: false });
        apiRequests[0].onError(new Error('stale error'));
        expect(completed).not.toHaveBeenCalled();
        expect(app.generating).toBe(true);
        expect(app.chatGenerationId).toBe(2);
        apiRequests[1].onDone({ stopped: false });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'complete', generationId: 2 });
        apiRequests[1].onDone({ stopped: false });
        expect(completed).toHaveBeenCalledTimes(1);
        dispose();
    });
});

describe('actual page API configuration hook', () => {
    it('updates the active API config from the selected provider without navigation', () => {
        const app = { apiConfig: { baseUrl: 'https://old.example.com/v1' } };
        const ModelSources = { apiConfig: vi.fn(() => ({ baseUrl: 'https://new.example.com/v1', model: 'model-b' })) };
        const factory = new Function('app', 'ModelSources', 'updateComposerState', `${section('function onApiConfigChanged', 'function bindHeader')} return onApiConfigChanged;`);
        const onApiConfigChanged = factory(app, ModelSources, vi.fn()) as (event: { detail: { id: string } }) => void;
        onApiConfigChanged({ detail: { id: 'api-a' } });
        expect(ModelSources.apiConfig).toHaveBeenCalledWith('api-a');
        expect(app.apiConfig.baseUrl).toBe('https://new.example.com/v1');
    });
});
