/**
 * Studio API client — OpenAI-compatible chat completions for the UOR-R4 Studio.
 *
 * The browser Studio is offline-first: the default path is the local WASM engine in
 * assets/js/uor_model_worker.js. This module is only used when the user explicitly
 * selects an OpenAI-compatible API model source in the Model sources panel. Nothing
 * here runs, fetches or imports anything at page load.
 *
 * Configuration (base URL, model, API key) is owned by assets/js/model_sources.js and
 * persisted in this browser's localStorage under "uor.studio.modelSources.v1". API keys
 * are never bundled in the repository and never leave this browser except as the
 * Authorization header of a request the user triggered.
 *
 * Public interface (also available as window.UORStudioApi):
 *   normalizeBaseUrl(baseUrl)                  -> string (validated, trailing slash removed)
 *   configFromSources()                        -> { baseUrl, model, apiKey } | null
 *   listModels(config, options)                -> Promise<{ ok, status, models, detail }>
 *   testConnection(config)                     -> Promise<{ ok, status, models, detail }>
 *   streamChatCompletions({ config, messages, signal, onDelta, onDone, onError, options })
 *   completeChatCompletions({ config, messages, options })
 */

const DEFAULT_TIMEOUT_MS = 10000;
const STREAM_HEADER = 'text/event-stream';

function failure(message, extra = {}) {
    const error = new Error(message);
    Object.assign(error, extra);
    return error;
}

export function normalizeBaseUrl(value) {
    const raw = String(value == null ? '' : value).trim();
    if (!raw) throw failure('An API base URL is required.');
    let parsed;
    try {
        parsed = new URL(raw);
    } catch (_error) {
        throw failure('The API base URL is not a valid URL. Example: http://127.0.0.1:8000/v1');
    }
    if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
        throw failure('Only http:// and https:// API base URLs are supported.');
    }
    let path = parsed.pathname.replace(/\/+$/, '');
    path = path.replace(/\/chat\/completions$/i, '').replace(/\/models$/i, '');
    parsed.pathname = path || '';
    parsed.search = '';
    parsed.hash = '';
    const normalised = parsed.toString().replace(/\/$/, '');
    if (!normalised) throw failure('The API base URL is empty after normalisation.');
    return normalised;
}

function endpoint(baseUrl, path) {
    return normalizeBaseUrl(baseUrl) + path;
}

function configFromObject(config) {
    if (!config || typeof config !== 'object') return null;
    const baseUrl = config.baseUrl || config.base_url || config.url;
    if (!baseUrl) return null;
    return {
        id: config.id || null,
        label: config.label || config.name || null,
        baseUrl: String(baseUrl),
        model: String(config.model || config.modelId || config.model_id || ''),
        apiKey: typeof config.apiKey === 'string' ? config.apiKey : (typeof config.api_key === 'string' ? config.api_key : ''),
        maxTokens: Number(config.maxTokens) > 0 ? Math.min(4096, Number(config.maxTokens)) : 768,
        temperature: Number.isFinite(Number(config.temperature)) ? Math.max(0, Math.min(2, Number(config.temperature))) : 0.7,
    };
}

/** Active API configuration from the model-source manager (or a small standalone fallback). */
export function configFromSources() {
    try {
        const source = globalThis.UORModelSources;
        if (source && typeof source.apiConfig === 'function') {
            const config = source.apiConfig();
            if (config) return configFromObject(config);
        }
    } catch (_error) {
        /* fall through to standalone storage */
    }
    try {
        const raw = globalThis.localStorage && globalThis.localStorage.getItem('uor.studio.modelSources.v1');
        if (!raw) return null;
        const state = JSON.parse(raw);
        const active = state && state.active;
        if (!active || active.kind !== 'api') return null;
        const config = (state.api || []).find((item) => item.id === active.id);
        return configFromObject(config);
    } catch (_error) {
        return null;
    }
}

function requireConfig(config) {
    const resolved = configFromObject(config) || configFromSources();
    if (!resolved) {
        throw failure('Select an OpenAI-compatible API model source before using API mode.');
    }
    resolved.baseUrl = normalizeBaseUrl(resolved.baseUrl);
    return resolved;
}

function requestHeaders(config) {
    const headers = { 'Content-Type': 'application/json', Accept: 'application/json, text/event-stream' };
    if (config.apiKey) headers.Authorization = 'Bearer ' + config.apiKey;
    return headers;
}

function timeoutSignal(parentSignal, timeoutMs) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(failure('The API request timed out.')), timeoutMs);
    const signal = parentSignal
        ? anySignal([parentSignal, controller.signal])
        : controller.signal;
    return { signal, dispose: () => clearTimeout(timer) };
}

function anySignal(signals) {
    if (typeof AbortSignal !== 'undefined' && typeof AbortSignal.any === 'function') {
        return AbortSignal.any(signals.filter(Boolean));
    }
    const controller = new AbortController();
    for (const signal of signals.filter(Boolean)) {
        if (signal.aborted) {
            controller.abort(signal.reason);
            break;
        }
        signal.addEventListener('abort', () => controller.abort(signal.reason), { once: true });
    }
    return controller.signal;
}

function describeHttpError(response, bodyText) {
    const body = String(bodyText || '').trim();
    let detail = body.slice(0, 400);
    try {
        const parsed = JSON.parse(body);
        const message = parsed && (parsed.error?.message || parsed.error || parsed.message || parsed.detail);
        if (typeof message === 'string' && message.trim()) detail = message.trim().slice(0, 400);
    } catch (_error) {
        /* keep raw text excerpt */
    }
    return failure('API request failed with HTTP ' + response.status + (detail ? ': ' + detail : '.'), {
        status: response.status,
    });
}

/** GET {base}/models. Returns the model ids when the endpoint exposes them. */
export async function listModels(config, options = {}) {
    const resolved = requireConfig(config);
    const timeoutMs = Number(options.timeoutMs) > 0 ? Number(options.timeoutMs) : DEFAULT_TIMEOUT_MS;
    const { signal, dispose } = timeoutSignal(options.signal, timeoutMs);
    try {
        const response = await fetch(endpoint(resolved.baseUrl, '/models'), {
            method: 'GET',
            headers: requestHeaders(resolved),
            signal,
            cache: 'no-store',
        });
        const bodyText = await response.text();
        if (!response.ok) throw describeHttpError(response, bodyText);
        let models = [];
        try {
            const parsed = JSON.parse(bodyText);
            const data = parsed && Array.isArray(parsed.data) ? parsed.data : [];
            models = data.map((item) => item && (item.id || item.model || item.name)).filter(Boolean).map(String);
        } catch (_error) {
            /* expose the endpoint as reachable even when /models is not JSON */
        }
        return { ok: true, status: response.status, models, detail: 'Endpoint reachable.' };
    } catch (error) {
        if (error && error.name === 'AbortError') throw failure('The API connection test timed out.');
        throw error;
    } finally {
        dispose();
    }
}

/** Test an API configuration without sending a prompt: GET /models. */
export async function testConnection(config) {
    const resolved = requireConfig(config);
    try {
        const result = await listModels(resolved);
        return {
            ok: true,
            status: result.status,
            models: result.models,
            detail: result.models.length
                ? 'Reachable. Models: ' + result.models.slice(0, 6).join(', ') + (result.models.length > 6 ? '…' : '')
                : 'Reachable (the endpoint did not list models).',
        };
    } catch (error) {
        return { ok: false, status: error && error.status ? error.status : 0, models: [], detail: String(error && error.message ? error.message : error) };
    }
}

function normaliseMessages(messages) {
    if (!Array.isArray(messages) || messages.length === 0) throw failure('At least one chat message is required.');
    return messages
        .filter((message) => message && typeof message.content === 'string' && message.content.length > 0)
        .map((message) => ({ role: ['system', 'user', 'assistant'].includes(message.role) ? message.role : 'user', content: message.content }));
}

/** Parse one SSE event block ("data: {...}") into an OpenAI streaming chunk. */
function parseSseBlock(block) {
    const lines = block.split(/\r?\n/);
    const dataLines = lines
        .filter((line) => line.startsWith('data:'))
        .map((line) => line.slice(5).trim());
    if (dataLines.length === 0) return null;
    const payload = dataLines.join('\n');
    if (payload === '[DONE]') return { done: true };
    try {
        return { chunk: JSON.parse(payload) };
    } catch (_error) {
        return null;
    }
}

/**
 * Stream an OpenAI-compatible chat completion.
 * Calls onDelta(text, meta) for each content fragment and resolves when the stream ends.
 */
export async function streamChatCompletions({ config, messages, signal, onDelta, onDone, onError, options = {} } = {}) {
    let resolved;
    try {
        resolved = requireConfig(config);
    } catch (error) {
        if (typeof onError === 'function') onError(error);
        throw error;
    }
    const body = {
        model: resolved.model || 'default',
        messages: normaliseMessages(messages),
        stream: true,
        temperature: Number.isFinite(Number(options.temperature)) ? Number(options.temperature) : resolved.temperature,
        max_tokens: Number(options.maxTokens) > 0 ? Math.min(4096, Number(options.maxTokens)) : resolved.maxTokens,
    };
    let response;
    try {
        response = await fetch(endpoint(resolved.baseUrl, '/chat/completions'), {
            method: 'POST',
            headers: requestHeaders(resolved),
            signal,
            body: JSON.stringify(body),
        });
    } catch (error) {
        if (error && error.name === 'AbortError') {
            if (typeof onDone === 'function') onDone({ stopped: true, text: '' });
            return { ok: false, stopped: true };
        }
        if (typeof onError === 'function') onError(error);
        throw error;
    }
    if (!response.ok) {
        let bodyText = '';
        try {
            bodyText = await response.text();
        } catch (_error) {
            /* ignore */
        }
        const error = describeHttpError(response, bodyText);
        if (typeof onError === 'function') onError(error);
        throw error;
    }

    let text = '';
    const emit = (delta, meta) => {
        if (!delta) return;
        text += delta;
        if (typeof onDelta === 'function') onDelta(delta, { ...meta, text });
    };

    const contentType = String(response.headers.get('content-type') || '').toLowerCase();
    if (!contentType.includes(STREAM_HEADER) || !response.body) {
        // Some gateways ignore stream=true and return one JSON object.
        const bodyText = await response.text();
        let parsed = null;
        try {
            parsed = JSON.parse(bodyText);
        } catch (_error) {
            throw failure('The API returned a non-JSON chat completion.');
        }
        const content = parsed?.choices?.[0]?.message?.content ?? parsed?.choices?.[0]?.text ?? '';
        emit(String(content || ''));
        if (typeof onDone === 'function') onDone({ stopped: false, text });
        return { ok: true, text };
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';
    try {
        for (;;) {
            const { value, done } = await reader.read();
            if (done) break;
            buffer += decoder.decode(value, { stream: true });
            let boundary = buffer.indexOf('\n\n');
            while (boundary !== -1) {
                const block = buffer.slice(0, boundary);
                buffer = buffer.slice(boundary + 2);
                const parsed = parseSseBlock(block);
                if (parsed && parsed.done) {
                    if (typeof onDone === 'function') onDone({ stopped: false, text });
                    return { ok: true, text };
                }
                const choice = parsed && parsed.chunk && parsed.chunk.choices && parsed.chunk.choices[0];
                if (choice) {
                    const delta = choice.delta && typeof choice.delta.content === 'string'
                        ? choice.delta.content
                        : (choice.message && typeof choice.message.content === 'string' ? choice.message.content : '');
                    emit(delta, { finishReason: choice.finish_reason || null, model: parsed.chunk.model || resolved.model });
                }
                boundary = buffer.indexOf('\n\n');
            }
        }
        if (buffer.trim()) {
            const parsed = parseSseBlock(buffer.trim());
            const choice = parsed && parsed.chunk && parsed.chunk.choices && parsed.chunk.choices[0];
            if (choice && choice.delta && typeof choice.delta.content === 'string') emit(choice.delta.content);
        }
        if (typeof onDone === 'function') onDone({ stopped: false, text });
        return { ok: true, text };
    } catch (error) {
        if (error && error.name === 'AbortError') {
            if (typeof onDone === 'function') onDone({ stopped: true, text });
            return { ok: false, stopped: true, text };
        }
        if (typeof onError === 'function') onError(error);
        throw error;
    } finally {
        try {
            reader.releaseLock();
        } catch (_error) {
            /* ignore */
        }
    }
}

/** Non-streaming completion (used by diagnostics; chat uses the streaming path). */
export async function completeChatCompletions({ config, messages, options = {} } = {}) {
    const resolved = requireConfig(config);
    const response = await fetch(endpoint(resolved.baseUrl, '/chat/completions'), {
        method: 'POST',
        headers: requestHeaders(resolved),
        body: JSON.stringify({
            model: resolved.model || 'default',
            messages: normaliseMessages(messages),
            stream: false,
            temperature: Number.isFinite(Number(options.temperature)) ? Number(options.temperature) : resolved.temperature,
            max_tokens: Number(options.maxTokens) > 0 ? Math.min(4096, Number(options.maxTokens)) : resolved.maxTokens,
        }),
    });
    const bodyText = await response.text();
    if (!response.ok) throw describeHttpError(response, bodyText);
    const parsed = JSON.parse(bodyText);
    const content = parsed?.choices?.[0]?.message?.content ?? parsed?.choices?.[0]?.text ?? '';
    return { ok: true, text: String(content || ''), raw: parsed };
}

const api = {
    normalizeBaseUrl,
    configFromSources,
    listModels,
    testConnection,
    streamChatCompletions,
    completeChatCompletions,
};

if (typeof globalThis !== 'undefined') {
    globalThis.UORStudioApi = api;
}

export default api;
