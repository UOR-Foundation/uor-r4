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
        activePredictionRequestId: null as string | null,
        predictionRequests: {},
        generating: false,
        mode: 'wasm',
        lastWorkerError: '',
    };
    const factory = new Function('app', 'document', 'window', 'toast', 'setMetrics', 'setEngineStatus', 'encodeTextWithArtifact', 'fillPredictorSha', '$',
        `${section('function isDiscreteCapabilities', 'function setPredictorMode')}
         ${section('function renderDiscretePrediction', 'function runPrediction')}
         ${section('function runPrediction', '/* ------------------------------------------------------------ status & toasts */')}
         ${section('function nextGenerationId', 'function beginGeneration')}
         ${section('function sendFromComposer', '/* ------------------------------------------------------------ model sources */')}
         ${section('function finishLocalGeneration', 'function generationTokenLimit')}
         ${section('function finishGenerationWithError', 'function generationTokenLimit')}
         ${section('function onWorkerMessage', 'function activeAssistantElement')}
         return { runPrediction, renderDiscretePrediction, onWorkerMessage, sendFromComposer, finishLocalGeneration, finishGenerationWithError };`);
    const hooks = factory(app, document, window, vi.fn(), vi.fn(), vi.fn(), vi.fn(() => [1, 2]), vi.fn(), (selector: string) => document.querySelector(selector)) as {
        runPrediction(prompt?: string): boolean;
        renderDiscretePrediction(message: unknown): void;
        onWorkerMessage(event: { data: Record<string, unknown> }): void;
        sendFromComposer(prompt?: string): number | false;
        finishLocalGeneration(message: { fullText?: string; tokenCount?: number }): void;
        finishGenerationWithError(error: string): void;
    };
    return { app, hooks };
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
});
