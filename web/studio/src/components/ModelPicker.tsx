import { useEffect, useRef, useState } from 'preact/hooks';
import type { ModelSourceAdapter } from '../adapters/modelSourceAdapter.js';
import type { ApiProvider, ModelSourcesApi } from '../../assets/js/model_sources.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { classifyEndpoint } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { Modal } from './Modal.js';

interface ModelPickerProps {
    store: LifecycleStore;
    adapter: ModelSourceAdapter;
    sources: ModelSourcesApi;
    onError(message: string): void;
    onSuccess?(): void;
}

interface ProviderDraft {
    label: string;
    preset: string;
    baseUrl: string;
    model: string;
    apiKey: string;
}

const emptyDraft: ProviderDraft = { label: '', preset: 'custom', baseUrl: '', model: '', apiKey: '' };

export function ModelPicker({ store, adapter, sources, onError, onSuccess }: ModelPickerProps) {
    const state = useLifecycle(store);
    const [url, setUrl] = useState('');
    const [providerDraft, setProviderDraft] = useState<ProviderDraft>(emptyDraft);
    const [editingId, setEditingId] = useState<string | null>(null);
    const [providerOpen, setProviderOpen] = useState(false);
    const [providerError, setProviderError] = useState<string | null>(null);
    const [removing, setRemoving] = useState<{ id: string; label: string } | null>(null);
    const [testResults, setTestResults] = useState<Record<string, 'testing' | 'passed' | 'failed'>>({});
    const testVersions = useRef<Record<string, number>>({});
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);
    const report = (error: unknown) => onError(error instanceof Error ? error.message : String(error));
    const run = (action: Promise<unknown>) => { action.catch(report); };

    useEffect(() => {
        let active = true;
        const changed = () => { if (active) refresh(); };
        document.addEventListener('uor-model-catalog-changed', changed);
        sources.loadModelCatalog().then(changed).catch(report);
        return () => {
            active = false;
            document.removeEventListener('uor-model-catalog-changed', changed);
        };
    }, [sources]);

    const progress = state.progress;
    const total = progress?.total ?? null;
    const resident = state.residentArtifact;

    function testProvider(provider: ApiProvider) {
        const version = (testVersions.current[provider.id] || 0) + 1;
        testVersions.current[provider.id] = version;
        setTestResults((current) => ({ ...current, [provider.id]: 'testing' }));
        run(sources.testApi(provider.id).then((result) => {
            if (testVersions.current[provider.id] === version) {
                setTestResults((current) => ({ ...current, [provider.id]: result.ok ? 'passed' : 'failed' }));
                if (result.ok) onSuccess?.();
            }
        }).catch((error) => {
            if (testVersions.current[provider.id] === version) {
                setTestResults((current) => ({ ...current, [provider.id]: 'failed' }));
            }
            report(error);
        }));
    }

    function editProvider(provider?: ApiProvider) {
        setEditingId(provider?.id ?? null);
        setProviderError(null);
        setProviderDraft(provider ? { label: provider.label, preset: provider.preset, baseUrl: provider.baseUrl, model: provider.model, apiKey: '' } : emptyDraft);
        setProviderOpen(true);
    }

    function saveProvider(event: Event) {
        event.preventDefault();
        setProviderError(null);
        try {
            classifyEndpoint(providerDraft.baseUrl);
            if (editingId) {
                const patch: Partial<ApiProvider> = {
                    label: providerDraft.label,
                    preset: providerDraft.preset,
                    baseUrl: providerDraft.baseUrl,
                    model: providerDraft.model,
                };
                if (providerDraft.apiKey) patch.apiKey = providerDraft.apiKey;
                sources.updateApi(editingId, patch);
                testVersions.current[editingId] = (testVersions.current[editingId] || 0) + 1;
                setTestResults((current) => {
                    const next = { ...current };
                    delete next[editingId];
                    return next;
                });
                if (state.selectedSource?.kind === 'api' && state.selectedSource.id === editingId) {
                    store.dispatch({ type: 'api/selected', sourceId: editingId, endpoint: providerDraft.baseUrl });
                    document.dispatchEvent(new CustomEvent('uor-api-config-changed', { detail: { id: editingId } }));
                }
            } else {
                sources.addApi(providerDraft);
            }
            setProviderOpen(false);
            setEditingId(null);
            setProviderDraft(emptyDraft);
            refresh();
            onSuccess?.();
        } catch (error) {
            setProviderError(error instanceof Error ? error.message : String(error));
            report(error);
        }
    }

    function selectProvider(provider: ApiProvider) {
        try {
            setProviderError(null);
            adapter.selectApi(provider.id, provider.baseUrl);
            onSuccess?.();
        } catch (error) {
            setProviderError(error instanceof Error ? error.message : String(error));
            report(error);
        }
    }

    function removeProvider() {
        if (!removing) return;
        const id = removing.id;
        try {
            setProviderError(null);
            sources.removeApi(id);
            testVersions.current[id] = (testVersions.current[id] || 0) + 1;
            setTestResults((current) => {
                const next = { ...current };
                delete next[id];
                return next;
            });
            if (state.selectedSource?.kind === 'api' && state.selectedSource.id === id) {
                const active = sources.active();
                store.dispatch({ type: 'artifact/selected', sourceId: active.id || 'bundled-demo' });
            }
            setRemoving(null);
            refresh();
            onSuccess?.();
        } catch (error) {
            setProviderError(error instanceof Error ? error.message : String(error));
            report(error);
        }
    }

    return (
        <section aria-label="Model sources">
            <h1>Models</h1>
            <p>{state.selectedSource ? `${state.selectedSource.kind === 'api' ? 'API' : 'Artifact'}: ${state.selectedSource.id}` : 'No model source selected'}</p>
            <p>{state.selectedSource?.kind === 'artifact' && (state.artifact.phase !== 'ready' || state.residentArtifact?.sourceId !== state.selectedSource.id) ? 'Selected model; not ready' : `Artifact: ${state.artifact.phase}`}</p>
            {state.artifact.error && <p role="alert">{state.artifact.error}</p>}

            <section aria-label="Local engine">
                <h2>Local engine</h2>
                <button type="button" onClick={() => run(adapter.loadBundled())}>Load bundled demo</button>
                <label>Model file<input aria-label="Model file" type="file" accept=".lut,.bin,.d11" onChange={(event) => {
                    const file = event.currentTarget.files?.[0];
                    if (file) run(adapter.loadLocalFile(file));
                }} /></label>
                <form data-action="load-url" onSubmit={(event) => {
                    event.preventDefault();
                    if (url.trim()) run(adapter.loadFromUrl(url.trim()));
                }}>
                    <label>Model URL<input aria-label="Model URL" type="url" value={url} onInput={(event) => setUrl(event.currentTarget.value)} /></label>
                    <button type="submit">Load URL</button>
                </form>
            </section>

            <section aria-label="Published models">
                <h2>Published models</h2>
                {sources.catalog().models.map((entry) => (
                    <button type="button" onClick={() => run(adapter.loadPublished(entry.id || entry.url))}>{`Load ${entry.label || entry.id || entry.url}`}</button>
                ))}
            </section>

            {(state.artifact.phase === 'downloading' || state.artifact.phase === 'finalizing') && (
                <div>
                    <progress
                        aria-label="Model download progress"
                        max={total ?? undefined}
                        value={state.artifact.phase === 'downloading' && total !== null ? progress?.loaded : undefined}
                    />
                    {state.artifact.phase === 'finalizing' ? (
                        <p>Finalizing model; progress is indeterminate</p>
                    ) : total !== null ? (
                        <p>{progress?.loaded ?? 0} / {total} bytes ({progress?.percent ?? 0}%)</p>
                    ) : (
                        <p>{progress?.loaded ?? 0} bytes downloaded; total unknown</p>
                    )}
                </div>
            )}

            {resident && (
                <dl aria-label="Resident artifact">
                    <dt>File</dt><dd>{resident.fileName || 'Unnamed artifact'}</dd>
                    <dt>Schema</dt><dd>{resident.schema || 'Unknown schema'}</dd>
                    {resident.sha256 && <><dt>Hash</dt><dd>SHA-256 {resident.sha256} ({resident.verified === true ? 'verified' : 'not verified'})</dd></>}
                    <dt>Engine</dt><dd>{resident.engineSupported === false ? 'Engine unsupported' : 'Engine supported'}</dd>
                    {resident.canGenerate === false && resident.canPredict === true && <><dt>Capability</dt><dd>Next-token prediction only</dd></>}
                    {resident.canGenerate === false && resident.canPredict !== true && <><dt>Capability</dt><dd>Generation unavailable</dd></>}
                </dl>
            )}

            <section aria-label="API providers">
                <h2>API providers</h2>
                <p>Connection test: {state.selectedSource?.kind === 'api' ? testResults[state.selectedSource.id] || state.api.testStatus : 'untested'}. Selection does not prove reachability.</p>
                {providerError && !providerOpen && !removing && <p role="alert">{providerError}</p>}
                <button type="button" onClick={() => editProvider()}>Add API provider</button>
                <ul>
                    {sources.list().api.map((provider) => (
                        <li>
                            <strong>{provider.label}</strong>
                            <span>{provider.baseUrl} · {provider.model}</span>
                            <span>Test: {testResults[provider.id] || (state.selectedSource?.kind === 'api' && state.selectedSource.id === provider.id ? state.api.testStatus : 'untested')}</span>
                            <button type="button" onClick={() => selectProvider(provider)}>{`Use ${provider.label}`}</button>
                            <button type="button" onClick={() => testProvider(provider)}>{`Test ${provider.label}`}</button>
                            <button type="button" onClick={() => editProvider(provider)}>{`Edit ${provider.label}`}</button>
                            <button type="button" onClick={() => { setProviderError(null); setRemoving({ id: provider.id, label: provider.label }); }}>{`Remove ${provider.label}`}</button>
                        </li>
                    ))}
                </ul>
            </section>

            <details>
                <summary>Research</summary>
                <p>Experimental sources are labeled separately and never presented as qualified conversational models.</p>
            </details>

            {providerOpen && (
                <Modal title={editingId ? 'Edit API provider' : 'Add API provider'} onClose={() => { setProviderOpen(false); setEditingId(null); }}>
                    {providerError && <p role="alert">{providerError}</p>}
                    <form data-action="save-provider" onSubmit={saveProvider}>
                        <label>Provider name<input aria-label="Provider name" required value={providerDraft.label} onInput={(event) => setProviderDraft({ ...providerDraft, label: event.currentTarget.value })} /></label>
                        <label>Provider base URL<input aria-label="Provider base URL" required type="url" value={providerDraft.baseUrl} onInput={(event) => setProviderDraft({ ...providerDraft, baseUrl: event.currentTarget.value })} /></label>
                        <label>Provider model<input aria-label="Provider model" value={providerDraft.model} onInput={(event) => setProviderDraft({ ...providerDraft, model: event.currentTarget.value })} /></label>
                        <label>API key<input aria-label="API key" type="password" autocomplete="off" value={providerDraft.apiKey} placeholder={editingId ? 'Leave blank to keep current key' : ''} onInput={(event) => setProviderDraft({ ...providerDraft, apiKey: event.currentTarget.value })} /></label>
                        <button type="submit">Save provider</button>
                    </form>
                </Modal>
            )}

            {removing && (
                <Modal title="Remove API provider" onClose={() => setRemoving(null)}>
                    {providerError && <p role="alert">{providerError}</p>}
                    <p>Remove {removing.label}? Existing conversations are preserved.</p>
                    <button type="button" autofocus onClick={() => setRemoving(null)}>Keep provider</button>
                    <button type="button" onClick={removeProvider}>Confirm removal</button>
                </Modal>
            )}
        </section>
    );
}
