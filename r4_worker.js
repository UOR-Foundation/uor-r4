// r4_worker.js — Web Worker for WASM event-loop isolation
// Offloads uor-r4 candidate scoring and generation off the main DOM thread.

let router = null;
let wasmInitialized = false;
let r4g1ProductionInstalled = false;
let r4g1InstallError = "schema-2 production bundle has not been installed";

const R4G1_PRODUCTION_COMPONENTS = [
    ['./graph/score.r4g1', 'graph'],
    ['./graph/score_sections_absent.r4g1', 'sectionsAbsentGraph'],
    ['./graph/score_label_shuffled.r4g1', 'labelShuffledGraph'],
    ['./tless_artifacts.bin', 'signatureArtifact'],
    ['./tless_store.bin', 'tlaComparatorStore'],
    ['./tokenizer.bin', 'tokenizer'],
    ['./graph/score_report.json', 'scoreReport'],
    ['./graph-cover/cover_report.json', 'compileReport'],
    ['./graph/deployed_quality_report.json', 'deployedQualityReport'],
    ['./graph/cross_surface_parity.json', 'crossSurfaceParity'],
    ['./graph/witness_replay.json', 'witnessReplay'],
    ['./corpus.meta', 'corpusMeta'],
    ['./corpus.records', 'corpusRecords'],
    ['./tokenizer_adapter.json', 'tokenizerAdapter'],
    ['./release-bundle.json', 'releaseManifest'],
];

// Fetch one immutable schema-2 envelope. Mixed/stale generations are refused
// by the Rust CID verifier, so no partial fetch can become active.
async function tryInstallStaticR4g1Bundle(scope, wasmModule) {
    if (typeof wasmModule.set_r4g1_production_bundle !== 'function') {
        throw new Error('WASM module has no schema-2 production-envelope installer');
    }
    const loaded = await Promise.all(R4G1_PRODUCTION_COMPONENTS.map(async ([path, name]) => {
        const response = await fetch(path, { cache: 'no-store' });
        if (!response.ok) {
            throw new Error(`required R4G1 component ${path} returned HTTP ${response.status}`);
        }
        return [name, new Uint8Array(await response.arrayBuffer())];
    }));
    const bytes = Object.fromEntries(loaded);
    wasmModule.set_r4g1_production_bundle(
        bytes.graph,
        bytes.sectionsAbsentGraph,
        bytes.labelShuffledGraph,
        bytes.signatureArtifact,
        bytes.tlaComparatorStore,
        bytes.tokenizer,
        bytes.scoreReport,
        bytes.compileReport,
        bytes.deployedQualityReport,
        bytes.crossSurfaceParity,
        bytes.witnessReplay,
        bytes.corpusMeta,
        bytes.corpusRecords,
        bytes.tokenizerAdapter,
        bytes.releaseManifest,
    );
    r4g1ProductionInstalled = true;
    r4g1InstallError = "";
    console.log(`[${scope}] schema-2 R4G1 production bundle installed`);
}

self.onmessage = async function (e) {
    const { type, id, payload } = e.data || {};
    
    switch (type) {
        case 'INIT_ENGINE': {
            try {
                const wasmModule = await import('./pkg/uor_r4_wasm_router.js');
                const { default: init, UorR4Router } = wasmModule;
                await init();
                // #790 item 5: this global was never assigned before, so
                // the r4g1/transformerless branches below could never run.
                self.wasm_module = wasmModule;
                try {
                    await tryInstallStaticR4g1Bundle('r4_worker', wasmModule);
                } catch (err) {
                    r4g1ProductionInstalled = false;
                    r4g1InstallError = String(err);
                    console.error('[r4_worker] strict R4G1 installation refused:', err);
                }
                // Initialize native geometric language model runtime
                if (typeof wasmModule.native_geometric_init === 'function') {
                    try {
                        const capsJson = wasmModule.native_geometric_init(new Uint8Array());
                        self.native_capabilities = JSON.parse(capsJson);
                        console.log('[r4_worker] Native geometric language model runtime initialized');
                    } catch (nErr) {
                        console.warn('[r4_worker] Native geometric runtime init warning:', nErr);
                    }
                }
                router = new UorR4Router(1.2);
                wasmInitialized = true;
                if (router.get_vocab_size() === 0) {
                    router.index_default_corpus();
                }
                self.postMessage({
                    type: 'ENGINE_READY',
                    id,
                    success: true,
                    indexedSentences: router.get_total_indexed_sentences()
                });
            } catch (err) {
                console.error("[r4_worker] Failed to init local WASM:", err);
                self.postMessage({
                    type: 'ENGINE_ERROR',
                    id,
                    error: String(err)
                });
            }
            break;
        }

        case 'GENERATE_RESPONSE': {
            if (!wasmInitialized || !router) {
                self.postMessage({
                    type: 'ENGINE_ERROR',
                    id,
                    error: "WASM Engine not initialized in worker"
                });
                return;
            }

            try {
                const { text, identity, max_tokens, temperature, gamma, selectedEngine, userId, projectId } = payload;
                let responseText = "";
                let generationMode = "geometric-local-worker";
                let nativeMeta = null;

                if (selectedEngine === "native-geometric") {
                    if (typeof self.wasm_module?.native_geometric_create_session !== 'function') {
                        throw new Error('WASM module has no native geometric runtime exports');
                    }
                    const sessId = `session-${Date.now()}`;
                    const uId = userId || identity || "default-user";
                    const pId = projectId || "studio-project";
                    const handle = self.wasm_module.native_geometric_create_session(sessId, uId, pId);
                    self.active_native_session = handle;

                    self.wasm_module.native_geometric_ingest(handle, text);
                    const stepJson = self.wasm_module.native_geometric_generate_step(handle, max_tokens || 32);
                    const stepResult = JSON.parse(stepJson);

                    responseText = stepResult.text || "Native geometric completion finished.";
                    generationMode = "native-geometric-wasm";
                    nativeMeta = {
                        tokenCount: stepResult.token_count,
                        stoppedBy: stepResult.stopped_by,
                        elapsedUs: stepResult.elapsed_us,
                        memoryFactsRead: stepResult.memory_facts_read,
                        handle,
                    };
                } else if (selectedEngine === "transformerless" || selectedEngine === "r4g1") {
                    if (!r4g1ProductionInstalled) {
                        throw new Error(`strict R4G1 production bundle unavailable: ${r4g1InstallError}`);
                    }
                    if (typeof self.wasm_module?.typed_r4g1_response !== 'function') {
                        throw new Error('WASM module has no typed R4G1 production response export');
                    }
                    const typed = JSON.parse(self.wasm_module.typed_r4g1_response(text, max_tokens));
                    if (typed.status !== 'supported-answer') {
                        throw new Error(`R4G1 ${typed.status}: ${typed.reason || typed.cause || 'request not served'}`);
                    }
                    responseText = typed.text;
                    generationMode = selectedEngine === "r4g1" ? "r4g1-zero-multiply-wasm" : "transformerless-r4g1-wasm";
                } else {
                    const geomResult = router.generate_geometric_response(
                        text,
                        identity || "null_dev_00",
                        max_tokens || 25,
                        temperature || 0.7,
                        10.0,
                        4.0,
                        gamma || 0.5
                    );
                    responseText = geomResult.text || "Manifold resonance too sparse for synthesis.";
                    generationMode = "geometric-local-worker";
                }

                router.index_sentence(responseText, identity || "null_dev_00");

                self.postMessage({
                    type: 'GENERATION_COMPLETE',
                    id,
                    responseText,
                    generationMode,
                    nativeMeta,
                    indexedSentences: router.get_total_indexed_sentences()
                });
            } catch (err) {
                console.error("[r4_worker] Generation error:", err);
                self.postMessage({
                    type: 'ENGINE_ERROR',
                    id,
                    error: String(err)
                });
            }
            break;
        }

        case 'CANCEL_GENERATION': {
            if (self.active_native_session && typeof self.wasm_module?.native_geometric_cancel === 'function') {
                try {
                    self.wasm_module.native_geometric_cancel(self.active_native_session);
                } catch (cErr) {
                    console.warn('[r4_worker] Cancel error:', cErr);
                }
            }
            self.postMessage({ type: 'CANCEL_ACK', id });
            break;
        }

        case 'EXPORT_SESSION': {
            try {
                const handle = payload?.handle || self.active_native_session;
                if (!handle || typeof self.wasm_module?.native_geometric_export_session !== 'function') {
                    throw new Error("No active session to export");
                }
                const bytes = self.wasm_module.native_geometric_export_session(handle);
                self.postMessage({ type: 'SESSION_EXPORTED', id, bytes });
            } catch (err) {
                self.postMessage({ type: 'ENGINE_ERROR', id, error: String(err) });
            }
            break;
        }

        case 'IMPORT_SESSION': {
            try {
                const handle = payload?.handle || self.active_native_session;
                const bytes = payload?.bytes;
                if (!handle || !bytes || typeof self.wasm_module?.native_geometric_import_session !== 'function') {
                    throw new Error("Invalid session import parameters");
                }
                self.wasm_module.native_geometric_import_session(handle, bytes);
                self.postMessage({ type: 'SESSION_IMPORTED', id });
            } catch (err) {
                self.postMessage({ type: 'ENGINE_ERROR', id, error: String(err) });
            }
            break;
        }

        case 'GET_CAPABILITIES': {
            try {
                if (typeof self.wasm_module?.native_geometric_capabilities !== 'function') {
                    throw new Error("Capabilities API not exposed by WASM");
                }
                const caps = JSON.parse(self.wasm_module.native_geometric_capabilities());
                self.postMessage({ type: 'CAPABILITIES_RESULT', id, capabilities: caps });
            } catch (err) {
                self.postMessage({ type: 'ENGINE_ERROR', id, error: String(err) });
            }
            break;
        }

        case 'INDEX_SENTENCE': {
            if (router && payload?.text) {
                router.index_sentence(payload.text, payload.identity || "null_dev_00");
                self.postMessage({
                    type: 'INDEX_COMPLETE',
                    id,
                    indexedSentences: router.get_total_indexed_sentences()
                });
            }
            break;
        }

        default:
            console.warn("[r4_worker] Unknown message type:", type);
            break;
    }
};
