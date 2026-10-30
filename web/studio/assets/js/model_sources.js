/**
 * Studio model-source manager.
 *
 * Owns the "where does generation run" choice for the Studio:
 *   - Local WASM engine: a UORLUT01 stack artifact loaded into the existing Web Worker
 *     (bundled synthetic demo, a local .lut/.bin file, or a user-initiated URL fetch).
 *   - OpenAI-compatible API: an HTTP endpoint the user configures. Only used when the user
 *     explicitly selects it; requests are user-initiated and never happen at page load.
 *
 * The page makes no external/off-origin request at load: it reads only the same-origin
 * ./models.json catalog and its bundled assets (the static scan and runtime trap enforce
 * external []). API keys are never bundled and are
 * stored only in this browser's localStorage. The exported object is also available at
 * window.UORModelSources for other Studio modules; every state change dispatches a
 * "uor-model-source-changed" CustomEvent on the document.
 *
 * Public interface:
 *   list()                              -> { active, wasm, api }
 *   active()                            -> { kind, id }
 *   apiConfig(id?)                      -> config | null
 *   loadLocalFile(file)                 -> Promise<{ bytes, fileName, header, meta, source }>
 *   loadFromUrl(url)                    -> Promise<{ bytes, fileName, header, meta, source }>
 *   save()                              -> persisted list()
 *   testApi(id?)                        -> Promise<{ ok, detail, models }>
 *   setActive({ kind, id })             -> list()
 *   addApi(config)                      -> config
 *   updateApi(id, patch)                -> config
 *   removeApi(id)                       -> list()
 *   mount(containerElement)             -> { render, unmount }
 */

const STORAGE_KEY = 'uor.studio.modelSources.v1';
const EVENT_NAME = 'uor-model-source-changed';
const MAX_WASM_BYTES = 64 * 1024 * 1024; // matches the worker's MAX_ARTIFACT_BYTES
const BUNDLED_DEMO_URL = './pkg/synthetic-stack.lut';
const BUNDLED_DEMO_LABEL = 'Synthetic demo (seed 5)';
const MODELS_CATALOG_URL = './models.json';
const MODELS_CATALOG_SCHEMA = 'uor-r4.studio-models/1';

// Optional same-origin publish catalog written next to index.html by the stage helper.
// Shape: { schema: "uor-r4.studio-models/1", bundled?: { label?, url? },
//          models?: [{ id?, label?, url, note?, sha256? }] }.
// A missing or invalid catalog is not an error: the hardcoded bundled demo stays.
const catalog = {
    bundled: { label: BUNDLED_DEMO_LABEL, url: BUNDLED_DEMO_URL },
    models: [],
};

export const API_PRESETS = Object.freeze([
    {
        preset: 'local-uor',
        label: 'Local uor-r4 API',
        baseUrl: 'http://127.0.0.1:8000/v1',
        model: 'uor-r4',
        note: 'An OpenAI-compatible gateway running on this machine.',
    },
    {
        preset: 'ollama',
        label: 'Ollama (local)',
        baseUrl: 'http://127.0.0.1:11434/v1',
        model: 'llama3.2',
        note: 'Ollama exposes an OpenAI-compatible endpoint at /v1.',
    },
    {
        preset: 'custom',
        label: 'Custom OpenAI-compatible API',
        baseUrl: '',
        model: '',
        note: 'Any endpoint that implements /chat/completions (and optionally /models).',
    },
]);

function emptyState() {
    return {
        wasm: {
            kind: 'bundled',
            id: 'bundled-demo',
            label: BUNDLED_DEMO_LABEL,
            fileName: 'synthetic-stack.lut',
            schema: null,
            synthetic: true,
            byteLength: null,
        },
        api: [
            apiEntry({ preset: 'local-uor', label: 'Local uor-r4 API', baseUrl: 'http://127.0.0.1:8000/v1', model: 'uor-r4', apiKey: '' }),
            apiEntry({ preset: 'ollama', label: 'Ollama (local)', baseUrl: 'http://127.0.0.1:11434/v1', model: 'llama3.2', apiKey: '' }),
        ],
        active: { kind: 'wasm', id: 'bundled-demo' },
    };
}

let idCounter = 0;
function nextId(prefix) {
    idCounter += 1;
    return prefix + '-' + Date.now().toString(36) + '-' + idCounter.toString(36);
}

function apiEntry(input) {
    return {
        id: input.id || nextId('api'),
        preset: input.preset || 'custom',
        label: String(input.label || 'API endpoint').trim(),
        baseUrl: String(input.baseUrl || '').trim(),
        model: String(input.model || '').trim(),
        apiKey: typeof input.apiKey === 'string' ? input.apiKey : '',
        createdAt: input.createdAt || Date.now(),
    };
}

function clone(value) {
    if (typeof structuredClone === 'function') {
        try {
            return structuredClone(value);
        } catch (_error) {
            /* fall through to JSON copy */
        }
    }
    return JSON.parse(JSON.stringify(value));
}

function loadState() {
    try {
        const raw = globalThis.localStorage && globalThis.localStorage.getItem(STORAGE_KEY);
        if (!raw) return emptyState();
        const parsed = JSON.parse(raw);
        if (!parsed || typeof parsed !== 'object') return emptyState();
        const fallback = emptyState();
        const wasm = Object.assign({}, fallback.wasm, parsed.wasm || {});
        if (!wasm.id) wasm.id = fallback.wasm.id;
        const api = Array.isArray(parsed.api) ? parsed.api.filter((item) => item && item.baseUrl !== undefined).map(apiEntry) : fallback.api;
        let active = parsed.active && (parsed.active.kind === 'wasm' || parsed.active.kind === 'api')
            ? { kind: parsed.active.kind, id: parsed.active.id || null }
            : fallback.active;
        if (active.kind === 'api' && !api.some((item) => item.id === active.id)) {
            active = fallback.active;
        }
        return { wasm, api, active };
    } catch (_error) {
        return emptyState();
    }
}

const state = loadState();
let mounted = null; // { container, render }
let message = { tone: 'muted', text: '' };
let editing = null; // { id: string|null, values: {...} }

function persist() {
    try {
        globalThis.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch (_error) {
        setMessage('error', 'Could not persist model sources: browser storage is unavailable or full.');
    }
}

function setMessage(tone, text) {
    message = { tone, text };
    updateMessageNode();
}

function updateMessageNode() {
    if (!mounted) return;
    const node = mounted.container.querySelector('[data-role="status"]');
    if (!node) return;
    node.textContent = message.text || '';
    node.className = 'source-status source-status-' + (message.tone || 'muted');
}

function emit(detail) {
    const source = publicList();
    const event = new CustomEvent(EVENT_NAME, { detail: Object.assign({ source }, detail) });
    document.dispatchEvent(event);
}

function publicList() {
    return { active: clone(state.active), wasm: clone(state.wasm), api: clone(state.api) };
}

function findApi(id) {
    if (!id) return null;
    return state.api.find((item) => item.id === id) || null;
}

function activeApi() {
    if (state.active.kind !== 'api') return null;
    return findApi(state.active.id);
}

function publicConfig(entry) {
    if (!entry) return null;
    return { id: entry.id, label: entry.label, preset: entry.preset, baseUrl: entry.baseUrl, model: entry.model, apiKey: entry.apiKey };
}

function setActive(next) {
    if (!next || (next.kind !== 'wasm' && next.kind !== 'api')) {
        throw new Error('Unknown model source kind.');
    }
    if (next.kind === 'api' && !findApi(next.id)) {
        throw new Error('That API endpoint no longer exists.');
    }
    state.active = { kind: next.kind, id: next.kind === 'api' ? next.id : (next.id || 'bundled-demo') };
    persist();
    message = { tone: 'muted', text: '' };
    if (mounted) mounted.render();
    emit({ kind: state.active.kind, id: state.active.id, apiConfig: state.active.kind === 'api' ? publicConfig(activeApi()) : null });
    return publicList();
}

function setWasmMeta(meta) {
    state.wasm = Object.assign({}, state.wasm, meta);
}

/** Accept only http(s) or same-origin relative URLs for user-initiated model downloads. */
function resolveModelUrl(raw) {
    const value = String(raw == null ? '' : raw).trim();
    if (!value) throw new Error('Enter a model file URL.');
    let url;
    try {
        url = new URL(value, document.baseURI);
    } catch (_error) {
        throw new Error('That model URL is not valid.');
    }
    if (url.protocol !== 'http:' && url.protocol !== 'https:') {
        throw new Error('Model URLs must use http(s).');
    }
    return url;
}

function parseUorLutHeader(arrayBuffer) {
    const bytes = new Uint8Array(arrayBuffer);
    if (bytes.byteLength < 16) {
        throw new Error('That file is too small to be a UORLUT01 stack artifact.');
    }
    const magic = new TextDecoder('ascii').decode(bytes.subarray(0, 8));
    if (magic !== 'UORLUT01') {
        throw new Error('Unrecognized file header; a stack artifact starts with the ASCII magic UORLUT01.');
    }
    const view = new DataView(arrayBuffer);
    const jsonLenBig = view.getBigUint64(8, true);
    const jsonLen = Number(jsonLenBig);
    if (!Number.isSafeInteger(jsonLen) || jsonLen <= 0 || 16 + jsonLen > bytes.byteLength) {
        throw new Error('The UORLUT01 header length is invalid for this file.');
    }
    let header;
    try {
        header = JSON.parse(new TextDecoder('utf-8').decode(bytes.subarray(16, 16 + jsonLen)));
    } catch (_error) {
        throw new Error('The UORLUT01 JSON header could not be parsed.');
    }
    const schema = header && typeof header.schema === 'string' ? header.schema : '';
    if (!/^uor-r4\.lut-(stack|discrete)(-trunk)?\/\d+$/.test(schema)) {
        throw new Error('Unsupported artifact schema "' + (schema || 'unknown') + '"; expected a uor-r4.lut-stack or uor-r4.lut-discrete container.');
    }
    return {
        schema,
        shape: (header && header.shape) || null,
        group: typeof header.group === 'number' ? header.group : null,
        synthetic: Boolean(header && header.source && header.source.synthetic),
        header,
    };
}

async function readArtifact(bytes, fileName, origin) {
    const parsed = parseUorLutHeader(bytes);
    const meta = {
        kind: origin.kind,
        id: origin.id,
        label: origin.label,
        fileName,
        schema: parsed.schema,
        synthetic: parsed.synthetic,
        byteLength: bytes.byteLength,
        shape: parsed.shape,
    };
    setWasmMeta(meta);
    state.active = { kind: 'wasm', id: meta.id };
    persist();
    if (mounted) mounted.render();
    emit({
        kind: 'wasm',
        id: meta.id,
        bytes,
        fileName,
        header: parsed.header,
        headerMeta: { schema: parsed.schema, shape: parsed.shape, synthetic: parsed.synthetic, group: parsed.group },
        meta: clone(meta),
    });
    return { bytes, fileName, header: parsed.header, meta: clone(meta), source: { kind: 'wasm', id: meta.id } };
}

function getBundledDemo() {
    return { label: catalog.bundled.label || BUNDLED_DEMO_LABEL, url: catalog.bundled.url || BUNDLED_DEMO_URL };
}

function catalogModelEntries() {
    return catalog.models.slice();
}

function sameOriginModelUrl(value) {
    var raw = String(value || '').trim();
    if (!raw) return null;
    try {
        var url = new URL(raw, document.baseURI);
        if (url.protocol !== 'http:' && url.protocol !== 'https:') return null;
        return url;
    } catch (_error) {
        return null;
    }
}

async function loadModelCatalog() {
    try {
        var response = await fetch(MODELS_CATALOG_URL, { cache: 'no-store', credentials: 'omit' });
        if (!response.ok) return;
        var data = await response.json();
        if (!data || data.schema !== MODELS_CATALOG_SCHEMA) return;
        if (data.bundled && typeof data.bundled === 'object') {
            var bundledUrl = sameOriginModelUrl(data.bundled.url || data.bundled.path || '');
            if (bundledUrl) {
                catalog.bundled = {
                    label: typeof data.bundled.label === 'string' && data.bundled.label.trim() ? data.bundled.label.trim() : BUNDLED_DEMO_LABEL,
                    url: bundledUrl.href,
                };
            }
        }
        // The publisher's stage helper writes models[] entries:
        //   kind "bundled" -> the bundled demo (label/url override the hardcoded default);
        //   any other kind with a URL -> an optional extra model card loaded user-initiated.
        var entries = Array.isArray(data.models) ? data.models : [];
        var published = [];
        entries.forEach(function (entry, index) {
            if (!entry || typeof entry !== 'object') return;
            var url = sameOriginModelUrl(entry.url || entry.path || '');
            if (!url) return;
            var label = typeof entry.label === 'string' && entry.label.trim()
                ? entry.label.trim()
                : (typeof entry.name === 'string' && entry.name.trim() ? entry.name.trim() : 'Model ' + (index + 1));
            var kind = typeof entry.kind === 'string' ? entry.kind.toLowerCase() : '';
            if (kind === 'bundled' || kind === 'demo') {
                catalog.bundled = { label: label, url: url.href };
                return;
            }
            published.push({
                id: typeof entry.id === 'string' && entry.id ? 'published-' + entry.id : 'published-' + index,
                label: label,
                url: url.href,
                note: typeof entry.note === 'string' ? entry.note : '',
                schema: typeof entry.schema === 'string' ? entry.schema : '',
                sha256: typeof entry.sha256 === 'string' ? entry.sha256 : '',
            });
        });
        catalog.models = published;
        if (mounted) mounted.render();
    } catch (_error) {
        // A publish catalog is optional; a missing or malformed one is ignored.
    }
}

function getCatalog() {
    return { bundled: getBundledDemo(), models: catalogModelEntries() };
}

const api = {
    STORAGE_KEY,
    EVENT_NAME,
    API_PRESETS,

    list() {
        return publicList();
    },

    active() {
        return clone(state.active);
    },

    apiConfig(id) {
        const entry = id ? findApi(id) : activeApi();
        return publicConfig(entry);
    },

    save() {
        persist();
        return publicList();
    },

    catalog() {
        return getCatalog();
    },

    bundledDemo() {
        return getBundledDemo();
    },

    loadModelCatalog() {
        return loadModelCatalog();
    },

    setActive(next) {
        return setActive(next);
    },

    addApi(input) {
        const entry = apiEntry(input || {});
        if (!entry.baseUrl) throw new Error('An API base URL is required.');
        state.api.push(entry);
        persist();
        if (mounted) mounted.render();
        emit({ kind: 'state', id: entry.id, apiConfig: publicConfig(entry) });
        return clone(entry);
    },

    updateApi(id, patch) {
        const entry = findApi(id);
        if (!entry) throw new Error('No API endpoint with that id.');
        Object.assign(entry, apiEntry(Object.assign({}, entry, patch, { id: entry.id, createdAt: entry.createdAt })));
        persist();
        if (mounted) mounted.render();
        emit({ kind: 'state', id: entry.id, apiConfig: publicConfig(entry) });
        return clone(entry);
    },

    removeApi(id) {
        const index = state.api.findIndex((item) => item.id === id);
        if (index < 0) return publicList();
        state.api.splice(index, 1);
        if (state.active.kind === 'api' && state.active.id === id) {
            state.active = { kind: 'wasm', id: state.wasm.id || 'bundled-demo' };
        }
        persist();
        if (mounted) mounted.render();
        emit({ kind: 'state', id: null });
        return publicList();
    },

    async testApi(id) {
        if (!globalThis.UORStudioApi || typeof globalThis.UORStudioApi.testConnection !== 'function') {
            throw new Error('The API client module is not loaded.');
        }
        const config = publicConfig(id ? findApi(id) : activeApi());
        if (!config) throw new Error('No API endpoint selected.');
        return globalThis.UORStudioApi.testConnection(config);
    },

    async loadLocalFile(file) {
        if (!file) throw new Error('Choose a .lut/.bin stack artifact first.');
        if (file.size > MAX_WASM_BYTES) {
            throw new Error('That model file is larger than the ' + Math.round(MAX_WASM_BYTES / (1024 * 1024)) + ' MB browser limit for this Studio.');
        }
        if (file.size === 0) throw new Error('The selected model file is empty.');
        setMessage('info', 'Reading ' + file.name + '…');
        const bytes = await file.arrayBuffer();
        const result = await readArtifact(bytes, file.name, { kind: 'local-file', id: 'local-file', label: file.name });
        setMessage('success', 'Loaded ' + file.name + '.');
        return result;
    },

    async loadFromUrl(rawUrl) {
        const url = resolveModelUrl(rawUrl);
        setMessage('info', 'Downloading ' + url.pathname.split('/').pop() + ' from ' + url.origin + '…');
        const response = await fetch(url.toString(), { cache: 'no-store', credentials: 'omit' });
        if (!response.ok) {
            throw new Error('The model download failed with HTTP ' + response.status + '.');
        }
        const declared = Number(response.headers.get('content-length') || 0);
        if (declared > MAX_WASM_BYTES) {
            throw new Error('That model file is larger than the ' + Math.round(MAX_WASM_BYTES / (1024 * 1024)) + ' MB browser limit for this Studio.');
        }
        const bytes = await response.arrayBuffer();
        if (bytes.byteLength > MAX_WASM_BYTES) {
            throw new Error('The downloaded model is larger than the browser limit for this Studio.');
        }
        const fileName = url.pathname.split('/').pop() || 'downloaded-stack.lut';
        const result = await readArtifact(bytes, fileName, { kind: 'url', id: 'url', label: fileName });
        setMessage('success', 'Loaded ' + fileName + '.');
        return result;
    },

    mount(container) {
        if (!container) throw new Error('A container element is required.');
        mounted = { container, render: () => renderAll(container) };
        container.classList.add('model-sources');
        renderAll(container);
        loadModelCatalog();
        container.addEventListener('click', onClick);
        container.addEventListener('submit', onSubmit);
        container.addEventListener('change', onChange);
        return {
            render: () => renderAll(container),
            unmount: () => {
                container.removeEventListener('click', onClick);
                container.removeEventListener('submit', onSubmit);
                container.removeEventListener('change', onChange);
                container.textContent = '';
                mounted = null;
            },
        };
    },
};

/* ------------------------------------------------------------------ rendering */

function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
}

function button(label, action, extra) {
    const node = el('button', 'btn ' + (extra && extra.className ? extra.className : 'btn-secondary'), label);
    node.type = 'button';
    node.dataset.action = action;
    if (extra && extra.id) node.id = extra.id;
    if (extra && extra.title) node.title = extra.title;
    if (extra && extra.ariaLabel) node.setAttribute('aria-label', extra.ariaLabel);
    return node;
}

function cardHeader(kind, label, detail, active) {
    const main = el('div', 'source-card-main');
    const title = el('div', 'source-card-title');
    title.appendChild(el('span', 'source-kind', kind));
    title.appendChild(el('span', 'source-label', label));
    if (active) title.appendChild(el('span', 'badge badge-active', 'Active'));
    main.appendChild(title);
    if (detail) main.appendChild(el('div', 'source-detail', detail));
    return main;
}

function renderAll(container) {
    container.textContent = '';
    const local = el('section', 'source-group');
    local.appendChild(el('h3', null, 'Local WASM engine'));
    local.appendChild(el('p', 'hint', 'Runs in a Web Worker inside this tab. Prompts and replies stay on this device. The bundled demo is a synthetic stack, not a chat model.'));

    local.appendChild(bundledCard());
    for (var i = 0; i < catalog.models.length; i += 1) {
        local.appendChild(publishedModelCard(catalog.models[i]));
    }
    local.appendChild(localFileCard());
    local.appendChild(urlCard());
    container.appendChild(local);

    const remote = el('section', 'source-group');
    remote.appendChild(el('h3', null, 'OpenAI-compatible API'));
    remote.appendChild(el('p', 'hint', 'Optional. When an API source is active, prompts are sent to that endpoint and leave this device. Keys are stored only in this browser and are never committed.'));
    for (const entry of state.api) remote.appendChild(apiCard(entry));
    if (state.api.length === 0) remote.appendChild(el('p', 'empty-note', 'No API endpoints configured.'));
    remote.appendChild(button('Add API endpoint', 'add-api', { className: 'btn-primary btn-small' }));
    if (editing) remote.appendChild(apiForm());
    container.appendChild(remote);

    const status = el('p', 'source-status', message.text || '');
    status.dataset.role = 'status';
    status.setAttribute('role', 'status');
    status.setAttribute('aria-live', 'polite');
    status.className = 'source-status source-status-' + (message.tone || 'muted');
    container.appendChild(status);

    const note = el('p', 'source-footnote', 'Model files are UORLUT01 stack containers (.lut / .bin). Local mode is offline; only a model URL, an API endpoint or a page reload will use the network.');
    container.appendChild(note);
}

function bundledCard() {
    const demo = getBundledDemo();
    const card = el('div', 'source-card');
    const activeBundled = state.active.kind === 'wasm' && state.wasm.kind === 'bundled' && state.active.id === 'bundled-demo';
    card.appendChild(cardHeader('Demo', demo.label, 'Bundled ' + demo.url + ' · loads only after this build produces it', activeBundled));
    const actions = el('div', 'source-card-actions');
    actions.appendChild(button('Load bundled demo', 'load-bundled', { className: 'btn-secondary btn-small' }));
    card.appendChild(actions);
    return card;
}

function publishedModelCard(entry) {
    const card = el('div', 'source-card');
    const active = state.active.kind === 'wasm' && state.wasm.kind === 'url';
    card.appendChild(cardHeader('Published', entry.label, (entry.note || entry.url) + (entry.sha256 ? ' · sha256 ' + entry.sha256.slice(0, 12) + '…' : ''), active));
    const actions = el('div', 'source-card-actions');
    const load = button('Load published model', 'load-published', { className: 'btn-secondary btn-small' });
    load.dataset.modelUrl = entry.url;
    actions.appendChild(load);
    card.appendChild(actions);
    return card;
}

function localFileCard() {
    const card = el('div', 'source-card');
    const activeLocal = state.active.kind === 'wasm' && state.wasm.kind === 'local-file';
    card.appendChild(cardHeader('Local file', activeLocal ? (state.wasm.fileName || 'Local stack file') : 'Local .lut / .bin stack file', 'Opens a UORLUT01 artifact from this device; nothing is uploaded.', activeLocal));
    const actions = el('div', 'source-card-actions');
    const input = el('input');
    input.type = 'file';
    input.accept = '.lut,.bin,application/octet-stream';
    input.id = 'modelFileInput';
    input.className = 'visually-hidden';
    input.dataset.role = 'model-file';
    const label = el('label', 'btn btn-secondary btn-small', 'Choose model file');
    label.htmlFor = 'modelFileInput';
    actions.appendChild(input);
    actions.appendChild(label);
    card.appendChild(actions);
    return card;
}

function urlCard() {
    const card = el('div', 'source-card');
    const activeUrl = state.active.kind === 'wasm' && state.wasm.kind === 'url';
    card.appendChild(cardHeader('From URL', activeUrl ? (state.wasm.fileName || 'Downloaded stack') : 'Load a stack artifact from a URL', 'A user-initiated download; the page makes no external requests on load.', activeUrl));
    const row = el('div', 'source-url-row');
    const input = el('input', 'input');
    input.type = 'url';
    input.id = 'modelUrlInput';
    input.placeholder = 'https://example.com/model.lut';
    input.autocomplete = 'off';
    input.spellcheck = false;
    input.setAttribute('aria-label', 'Model file URL');
    row.appendChild(input);
    row.appendChild(button('Load URL', 'load-url', { className: 'btn-secondary btn-small' }));
    card.appendChild(row);
    return card;
}

function apiCard(entry) {
    const card = el('div', 'source-card');
    const active = state.active.kind === 'api' && state.active.id === entry.id;
    const detail = (entry.baseUrl || 'No base URL yet.') + ' · model: ' + (entry.model || '(not set)') + (entry.apiKey ? ' · key set' : ' · no key');
    card.appendChild(cardHeader('API', entry.label, detail, active));
    const actions = el('div', 'source-card-actions');
    if (!active) actions.appendChild(button('Use', 'api-use', { className: 'btn-primary btn-small' }));
    actions.appendChild(button('Test', 'api-test', { className: 'btn-secondary btn-small' }));
    actions.appendChild(button('Edit', 'api-edit', { className: 'btn-ghost btn-small' }));
    actions.appendChild(button('Remove', 'api-remove', { className: 'btn-ghost btn-small btn-danger', title: 'Remove this API endpoint' }));
    card.appendChild(actions);
    card.dataset.apiId = entry.id;
    return card;
}

function field(labelText, input) {
    const wrap = el('label', 'field');
    wrap.appendChild(el('span', 'field-label', labelText));
    wrap.appendChild(input);
    return wrap;
}

function apiForm() {
    const form = el('form', 'api-form');
    form.dataset.role = 'api-form';
    const values = editing.values;
    const preset = el('select', 'input');
    preset.name = 'preset';
    for (const option of API_PRESETS) {
        const node = el('option', null, option.label);
        node.value = option.preset;
        if (values.preset === option.preset) node.selected = true;
        preset.appendChild(node);
    }
    form.appendChild(field('Preset', preset));

    const label = el('input', 'input');
    label.name = 'label';
    label.required = true;
    label.value = values.label || '';
    label.placeholder = 'My endpoint';
    form.appendChild(field('Name', label));

    const baseUrl = el('input', 'input');
    baseUrl.name = 'baseUrl';
    baseUrl.required = true;
    baseUrl.value = values.baseUrl || '';
    baseUrl.placeholder = 'http://127.0.0.1:8000/v1';
    baseUrl.spellcheck = false;
    baseUrl.autocomplete = 'off';
    form.appendChild(field('Base URL', baseUrl));

    const model = el('input', 'input');
    model.name = 'model';
    model.required = true;
    model.value = values.model || '';
    model.placeholder = 'model name';
    model.autocomplete = 'off';
    form.appendChild(field('Model', model));

    const apiKey = el('input', 'input');
    apiKey.name = 'apiKey';
    apiKey.type = 'password';
    apiKey.value = values.apiKey || '';
    apiKey.placeholder = 'optional; stored only in this browser';
    apiKey.autocomplete = 'off';
    form.appendChild(field('API key', apiKey));

    const actions = el('div', 'form-actions');
    const save = el('button', 'btn btn-primary btn-small', 'Save endpoint');
    save.type = 'submit';
    actions.appendChild(save);
    actions.appendChild(button('Cancel', 'api-cancel', { className: 'btn-ghost btn-small' }));
    form.appendChild(actions);
    return form;
}

function readForm(form) {
    const data = new FormData(form);
    return {
        preset: String(data.get('preset') || 'custom'),
        label: String(data.get('label') || '').trim(),
        baseUrl: String(data.get('baseUrl') || '').trim(),
        model: String(data.get('model') || '').trim(),
        apiKey: String(data.get('apiKey') || ''),
    };
}

function applyPresetToForm(form) {
    const presetName = form.elements.preset.value;
    const preset = API_PRESETS.find((item) => item.preset === presetName);
    if (!preset) return;
    if (preset.preset !== 'custom') {
        form.elements.label.value = preset.label;
        form.elements.baseUrl.value = preset.baseUrl;
        form.elements.model.value = preset.model;
    }
}

async function onClick(event) {
    const target = event.target.closest('[data-action]');
    if (!target || !mounted.container.contains(target)) return;
    const action = target.dataset.action;
    try {
        if (action === 'load-bundled') {
            await loadBundled();
        } else if (action === 'load-published') {
            await api.loadFromUrl(target.dataset.modelUrl);
        } else if (action === 'load-url') {
            const input = mounted.container.querySelector('#modelUrlInput');
            await api.loadFromUrl(input.value);
        } else if (action === 'add-api') {
            editing = { id: null, values: { preset: 'local-uor', label: API_PRESETS[0].label, baseUrl: API_PRESETS[0].baseUrl, model: API_PRESETS[0].model, apiKey: '' } };
            mounted.render();
        } else if (action === 'api-cancel') {
            editing = null;
            mounted.render();
        } else if (action === 'api-use') {
            const card = target.closest('[data-api-id]');
            setActive({ kind: 'api', id: card.dataset.apiId });
            setMessage('success', 'API mode active: prompts will be sent to the configured endpoint.');
        } else if (action === 'api-test') {
            const card = target.closest('[data-api-id]');
            setMessage('info', 'Testing endpoint…');
            const result = await api.testApi(card.dataset.apiId);
            setMessage(result.ok ? 'success' : 'error', result.detail || (result.ok ? 'Reachable.' : 'Unreachable.'));
        } else if (action === 'api-edit') {
            const card = target.closest('[data-api-id]');
            const entry = findApi(card.dataset.apiId);
            if (entry) {
                editing = { id: entry.id, values: { preset: entry.preset, label: entry.label, baseUrl: entry.baseUrl, model: entry.model, apiKey: entry.apiKey } };
                mounted.render();
            }
        } else if (action === 'api-remove') {
            const card = target.closest('[data-api-id]');
            api.removeApi(card.dataset.apiId);
        }
    } catch (error) {
        setMessage('error', String(error && error.message ? error.message : error));
    }
}

async function loadBundled() {
    const demo = getBundledDemo();
    setMessage('info', 'Loading the bundled synthetic demo…');
    const url = new URL(demo.url, document.baseURI);
    const response = await fetch(url.toString(), { cache: 'no-store' });
    if (!response.ok) {
        throw new Error('The bundled demo model is not present in this build (HTTP ' + response.status + '). A wasm build must produce pkg/synthetic-stack.lut first.');
    }
    const bytes = await response.arrayBuffer();
    const fileName = url.pathname.split('/').pop() || 'synthetic-stack.lut';
    const result = await readArtifact(bytes, fileName, { kind: 'bundled', id: 'bundled-demo', label: demo.label });
    setMessage('success', 'Bundled synthetic demo loaded (' + Math.round(bytes.byteLength / 1024) + ' KB).');
    return result;
}

function onSubmit(event) {
    const form = event.target.closest('[data-role="api-form"]');
    if (!form) return;
    event.preventDefault();
    try {
        const values = readForm(form);
        if (!values.baseUrl) throw new Error('An API base URL is required.');
        if (editing && editing.id) {
            api.updateApi(editing.id, values);
            setMessage('success', 'Endpoint updated.');
        } else {
            api.addApi(values);
            setMessage('success', 'Endpoint added.');
        }
        editing = null;
        mounted.render();
    } catch (error) {
        setMessage('error', String(error && error.message ? error.message : error));
    }
}

async function onChange(event) {
    const input = event.target.closest('[data-role="model-file"]');
    if (input && input.files && input.files[0]) {
        try {
            await api.loadLocalFile(input.files[0]);
        } catch (error) {
            setMessage('error', String(error && error.message ? error.message : error));
        }
    }
    const presetSelect = event.target.closest('select[name="preset"]');
    if (presetSelect) {
        const form = presetSelect.closest('[data-role="api-form"]');
        if (form) applyPresetToForm(form);
    }
}

/* Attach the manager to the page global for other Studio modules / tests. */
if (typeof globalThis !== 'undefined') {
    globalThis.UORModelSources = api;
}

export const ModelSources = api;

/** Mount the Model sources panel into a container element. */
export function mountModelSources(container) {
    return api.mount(container);
}

export default api;
