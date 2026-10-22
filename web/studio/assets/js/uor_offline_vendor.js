// Self-hosted offline vendor shims and Tauri native desktop bridge for UOR-R4 Studio
// (`web/studio/index.html`). Replaces external CDN scripts (marked, KaTeX, pdf.js,
// three.js, Monaco loader) so both the web client and the Tauri desktop client run
// 100% offline with zero external network requests.

(function initUorOfflineVendor(globalScope) {
    'use strict';

    function escapeHtml(raw) {
        return String(raw ?? '')
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    // 1. Offline Markdown Renderer (`marked` compatible API)
    if (typeof globalScope.marked === 'undefined') {
        class OfflineRenderer {
            code(codeText, infostring) {
                const lang = escapeHtml((infostring || 'text').trim().split(/\s+/)[0]);
                return `<pre><code class="language-${lang}">${escapeHtml(codeText)}</code></pre>`;
            }
        }
        let customRenderer = null;
        globalScope.marked = {
            Renderer: OfflineRenderer,
            use(opts) {
                if (opts && opts.renderer) {
                    customRenderer = opts.renderer;
                }
            },
            parse(markdownText) {
                const src = String(markdownText ?? '');
                const blocks = [];
                const withPlaceholders = src.replace(/```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g, (_m, lang, body) => {
                    const idx = blocks.length;
                    if (customRenderer && typeof customRenderer.code === 'function') {
                        try {
                            blocks.push(customRenderer.code(body, lang || 'text'));
                            return `@@UOR_CODE_BLOCK_${idx}@@`;
                        } catch (_e) {
                            // Fall back to standard pre/code below.
                        }
                    }
                    blocks.push(
                        `<pre><code class="language-${escapeHtml(lang || 'text')}">${escapeHtml(body)}</code></pre>`,
                    );
                    return `@@UOR_CODE_BLOCK_${idx}@@`;
                });

                let html = escapeHtml(withPlaceholders);
                html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');
                html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');
                html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');
                html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
                html = html.replace(/`([^`\n]+)`/g, '<code>$1</code>');
                html = html.replace(/\n\n+/g, '</p><p>');
                html = `<p>${html}</p>`;
                html = html.replace(/@@UOR_CODE_BLOCK_(\d+)@@/g, (_m, idx) => blocks[Number(idx)] || '');
                return html;
            },
        };
    }

    // 2. Offline Math Formatter (`katex` compatible API)
    if (typeof globalScope.katex === 'undefined') {
        globalScope.katex = {
            renderToString(expr, opts = {}) {
                const tag = opts.displayMode ? 'div' : 'span';
                const cls = opts.displayMode ? 'katex-display' : 'katex-inline';
                return `<${tag} class="${cls}"><code>${escapeHtml(expr)}</code></${tag}>`;
            },
        };
    }
    if (typeof globalScope.renderMathInElement === 'undefined') {
        globalScope.renderMathInElement = function renderMathInElement() {};
    }

    // 3. Offline Code Editor & Diff Shim (`monaco` + AMD `require` compatible API)
    if (typeof globalScope.monaco === 'undefined') {
        function createOfflineModel(initialValue = '', language = 'plaintext') {
            let currentValue = String(initialValue ?? '');
            const listeners = [];
            return {
                language,
                getValue() {
                    return currentValue;
                },
                setValue(next) {
                    currentValue = String(next ?? '');
                    for (const fn of listeners) {
                        try {
                            fn();
                        } catch (_e) {}
                    }
                },
                onDidChangeContent(fn) {
                    if (typeof fn === 'function') listeners.push(fn);
                },
            };
        }

        globalScope.monaco = {
            editor: {
                createModel(value, language) {
                    return createOfflineModel(value, language);
                },
                create(container, options = {}) {
                    let model = createOfflineModel(options.value || '', options.language || 'plaintext');
                    const changeListeners = [];
                    let textarea = null;
                    if (container && typeof container.appendChild === 'function' && globalScope.document) {
                        container.innerHTML = '';
                        textarea = globalScope.document.createElement('textarea');
                        textarea.className = 'uor-offline-code-editor';
                        textarea.style.cssText =
                            'width:100%;height:100%;background:#050608;color:#e2e8f0;border:none;outline:none;padding:12px;font-family:var(--font-mono,monospace);font-size:12px;line-height:1.5;resize:none;';
                        textarea.value = model.getValue();
                        textarea.addEventListener('input', () => {
                            model.setValue(textarea.value);
                            for (const cb of changeListeners) {
                                try {
                                    cb();
                                } catch (_e) {}
                            }
                        });
                        container.appendChild(textarea);
                    }
                    return {
                        getValue() {
                            return model.getValue();
                        },
                        setValue(val) {
                            model.setValue(val);
                            if (textarea) textarea.value = model.getValue();
                        },
                        getModel() {
                            return model;
                        },
                        setModel(newModel) {
                            if (newModel) {
                                model = newModel;
                                if (textarea) textarea.value = model.getValue();
                            }
                        },
                        onDidChangeModelContent(cb) {
                            if (typeof cb === 'function') changeListeners.push(cb);
                        },
                        layout() {},
                    };
                },
                createDiffEditor(container) {
                    let currentDiff = { original: null, modified: null };
                    return {
                        setModel(diffModels) {
                            currentDiff = diffModels || { original: null, modified: null };
                            if (container && globalScope.document) {
                                const orig = currentDiff.original ? currentDiff.original.getValue() : '';
                                const mod = currentDiff.modified ? currentDiff.modified.getValue() : '';
                                container.innerHTML = `<pre style="margin:0;padding:12px;color:#e2e8f0;font-family:var(--font-mono,monospace);font-size:12px;overflow:auto;height:100%;">--- original (${orig.length} B)\n+++ proposed (${mod.length} B)\n\n${escapeHtml(mod)}</pre>`;
                            }
                        },
                        getModel() {
                            return currentDiff;
                        },
                        layout() {},
                    };
                },
            },
        };
    }

    if (typeof globalScope.require === 'undefined') {
        const amdRequire = function amdRequire(_deps, callback) {
            if (typeof callback === 'function') {
                setTimeout(() => callback(), 0);
            }
        };
        amdRequire.config = function config() {};
        globalScope.require = amdRequire;
    }

    // 4. Tauri Native Desktop Model Worker Bridge
    // Connects the Studio `modelWorker` protocol (`prewarm`, `load_model`, `generate`,
    // `export_session`, `import_session`, `reset_session`, `close_session`,
    // `save_session_durable`, `restore_session_durable`) directly to `window.__TAURI__.core.invoke`
    // so both web and desktop clients share one identical UI interface.
    globalScope.createTauriNativeModelWorker = function createTauriNativeModelWorker(tauriCore) {
        const listeners = [];
        let capabilities = null;
        let activeJob = null;

        function emit(data) {
            const event = { data };
            if (typeof workerShim.onmessage === 'function') {
                workerShim.onmessage(event);
            }
            for (const fn of [...listeners]) {
                try {
                    fn(event);
                } catch (_e) {}
            }
        }

        function encodePromptTokens(payload, vocabSize) {
            if (Array.isArray(payload.tokenIds) && payload.tokenIds.length > 0) {
                return payload.tokenIds.map((t) => Number(t));
            }
            const userMessage = (payload.messages || []).filter((m) => m.role === 'user').pop();
            const text = typeof userMessage?.content === 'string' ? userMessage.content.trim() : '';
            if (!text) {
                throw new Error('A non-empty prompt or tokenIds array is required.');
            }
            const rawBytes = new TextEncoder().encode(text);
            const out = new Array(rawBytes.length);
            for (let i = 0; i < rawBytes.length; i++) {
                out[i] = rawBytes[i] % vocabSize;
            }
            return out;
        }

        const workerShim = {
            isTauriNativeBridge: true,
            onmessage: null,
            addEventListener(type, fn) {
                if (type === 'message' && typeof fn === 'function') {
                    listeners.push(fn);
                }
            },
            removeEventListener(type, fn) {
                if (type === 'message') {
                    const idx = listeners.indexOf(fn);
                    if (idx !== -1) listeners.splice(idx, 1);
                }
            },
            async postMessage(message) {
                const { action, id, payload = {} } = message || {};
                try {
                    if (action === 'cancel' || action === 'stop_generation') {
                        if (activeJob) activeJob.cancelled = true;
                        return;
                    }
                    if (action === 'get_storage_status') {
                        emit({
                            action: 'storage_status_result',
                            storage: { usageMB: '0.0', quotaMB: '64.0' },
                        });
                        return;
                    }
                    if (action === 'prewarm') {
                        emit({
                            action: 'runtime_loaded',
                            engine: 'uor-r4-integer/native-desktop',
                            ready: Boolean(capabilities),
                            capabilities,
                        });
                        return;
                    }
                    if (action === 'load_model' || action === 'load_local_model') {
                        if (activeJob) {
                            throw new Error('Stop generation before loading a stack artifact.');
                        }
                        let caps;
                        const filePath = payload.filePath || payload.path;
                        if (filePath) {
                            caps = await tauriCore.invoke('native_load_model', {
                                filePath,
                                expectedSha256: payload.expectedSha256 || null,
                            });
                        } else if (payload.bytes) {
                            const raw = payload.bytes instanceof Uint8Array
                                ? payload.bytes
                                : new Uint8Array(payload.bytes);
                            caps = await tauriCore.invoke('native_load_model', {
                                bytes: Array.from(raw),
                                fileName: payload.fileName || 'synthetic-stack.lut',
                                expectedSha256: payload.expectedSha256 || null,
                            });
                        } else {
                            caps = await tauriCore.invoke('native_load_synthetic_standin', {});
                        }
                        capabilities = {
                            ...caps,
                            alloc_events: 1,
                            leaked_models: 0,
                        };
                        emit({
                            action: 'model_loaded',
                            capabilities,
                            bytes: capabilities.artifact_bytes,
                        });
                        return;
                    }
                    if (!capabilities) {
                        throw new Error('Load a UORLUT01 stack file (.lut) before running a session command.');
                    }
                    if (action === 'close_session' || action === 'reset_session') {
                        if (activeJob) {
                            throw new Error('Stop generation before closing or resetting a session.');
                        }
                        const convId = payload.conversationId || 'default';
                        await tauriCore.invoke('native_reset_session', { conversationId: convId });
                        emit({
                            action: 'session_reset',
                            id,
                            conversationId: convId,
                        });
                        return;
                    }
                    if (action === 'export_session') {
                        if (activeJob) {
                            throw new Error('Stop generation before exporting a session checkpoint.');
                        }
                        const convId = payload.conversationId || 'default';
                        const res = await tauriCore.invoke('native_export_session', { conversationId: convId });
                        emit({
                            action: 'session_exported',
                            id,
                            conversationId: convId,
                            bytes: new Uint8Array(res.bytes || []),
                            lastToken: res.lastToken ?? null,
                        });
                        return;
                    }
                    if (action === 'import_session') {
                        if (activeJob) {
                            throw new Error('Stop generation before importing a session checkpoint.');
                        }
                        const convId = payload.conversationId;
                        const raw = payload.bytes instanceof Uint8Array
                            ? payload.bytes
                            : new Uint8Array(payload.bytes || []);
                        const res = await tauriCore.invoke('native_import_session', {
                            conversationId: convId,
                            bytes: Array.from(raw),
                            lastToken: payload.lastToken ?? null,
                        });
                        emit({
                            action: 'session_imported',
                            id,
                            conversationId: convId,
                            bytes: res.bytes,
                        });
                        return;
                    }
                    if (action === 'save_session_durable') {
                        const convId = payload.conversationId || 'default';
                        const receipt = await tauriCore.invoke('native_save_session_durable', {
                            userId: payload.userId || 'local-user',
                            projectId: payload.projectId || 'studio',
                            conversationId: convId,
                            storeDir: payload.storeDir || null,
                        });
                        emit({
                            action: 'session_durable_saved',
                            id,
                            conversationId: convId,
                            receipt,
                        });
                        return;
                    }
                    if (action === 'restore_session_durable') {
                        const convId = payload.conversationId || 'default';
                        const receipt = await tauriCore.invoke('native_restore_session_durable', {
                            userId: payload.userId || 'local-user',
                            projectId: payload.projectId || 'studio',
                            conversationId: convId,
                            storeDir: payload.storeDir || null,
                            manifestBytes: payload.manifestBytes ? Array.from(payload.manifestBytes) : null,
                        });
                        emit({
                            action: 'session_durable_restored',
                            id,
                            conversationId: convId,
                            receipt,
                        });
                        return;
                    }
                    if (action !== 'generate') {
                        throw new Error(`Unknown worker action: ${action}`);
                    }
                    if (activeJob) {
                        throw new Error('Another generation is active. Wait for its completion or cancellation.');
                    }
                    const convId = payload.conversationId || 'default';
                    const promptTokens = encodePromptTokens(payload, capabilities.shape.vocab);
                    const maxNewTokens = Math.min(
                        4096,
                        Math.max(1, Number(payload.options?.max_new_tokens) || 32),
                    );
                    const job = { id, conversationId: convId, cancelled: false };
                    activeJob = job;
                    try {
                        const out = await tauriCore.invoke('native_step_tokens', {
                            conversationId: convId,
                            tokenIds: promptTokens,
                            maxNewTokens,
                        });
                        for (const ev of out.events || []) {
                            if (job.cancelled) break;
                            emit({
                                action: 'stream_token',
                                id,
                                chunk: ev.chunk,
                                fullText: ev.full_text,
                                tokenId: ev.token_id,
                                marginQ16: ev.margin_q16,
                                stepCount: ev.position,
                                tokenCount: ev.step_index + 1,
                                tps: null,
                            });
                        }
                        emit({
                            action: 'generate_complete',
                            id,
                            fullText: out.receipt.full_text,
                            tokenIds: out.receipt.token_ids,
                            tokenCount: out.receipt.token_ids.length,
                            durationSec: 0,
                            tps: null,
                            stoppedBy: job.cancelled ? 'cancelled' : 'length',
                            weightsPerToken: out.receipt.weights_per_token,
                            allocDelta: 0,
                            allocEvents: 1,
                            statusLabel: out.receipt.status_label,
                            tokenizerLabel: out.receipt.tokenizer_label,
                        });
                    } finally {
                        if (activeJob === job) activeJob = null;
                    }
                } catch (error) {
                    emit({
                        action: action === 'generate' ? 'generate_error' : 'operation_error',
                        id,
                        error: String(error),
                    });
                }
            },
        };
        return workerShim;
    };
})(typeof globalThis !== 'undefined' ? globalThis : window);
