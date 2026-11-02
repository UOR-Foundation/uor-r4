import { useState } from 'preact/hooks';
import type { ModelSourceAdapter } from '../adapters/modelSourceAdapter.js';
import type { ApiProvider, ModelSourcesApi } from '../../assets/js/model_sources.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { Modal } from './Modal.js';

interface ModelPickerProps {
    store: LifecycleStore;
    adapter: ModelSourceAdapter;
    sources: ModelSourcesApi;
    onError(message: string): void;
}

interface ProviderDraft {
    label: string;
    preset: string;
    baseUrl: string;
    model: string;
    apiKey: string;
}

const emptyDraft: ProviderDraft = { label: '', preset: 'custom', baseUrl: '', model: '', apiKey: '' };

export function ModelPicker({ store, adapter, sources, onError }: ModelPickerProps) {
    const state = useLifecycle(store);
    const [url, setUrl] = useState('');
    const [providerDraft, setProviderDraft] = useState<ProviderDraft>(emptyDraft);
    const [editingId, setEditingId] = useState<string | null>(null);
    const [providerOpen, setProviderOpen] = useState(false);
    const [removing, setRemoving] = useState<ApiProvider | null>(null);
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);
    const report = (error: unknown) => onError(error instanceof Error ? error.message : String(error));
    const run = (action: Promise<unknown>) => { action.catch(report); };
    const progress = state.progress;
    const total = progress?.total ?? null;
    const resident = state.residentArtifact;

    function editProvider(provider?: ApiProvider) {
        setEditingId(provider?.id ?? null);
        setProviderDraft(provider ? { label: provider.label, preset: provider.preset, baseUrl: provider.baseUrl, model: provider.model, apiKey: '' } : emptyDraft);
        setProviderOpen(true);
    }

    function saveProvider(event: Event) {
        event.preventDefault();
        try {
            if (editingId) {
                const patch: Partial<ApiProvider> = {
                    label: providerDraft.label,
                    preset: providerDraft.preset,
                    baseUrl: providerDraft.baseUrl,
                    model: providerDraft.model,
                };
                if (providerDraft.apiKey) patch.apiKey = providerDraft.apiKey;
                sources.updateApi(editingId, patch);
            } else {
                sources.addApi(providerDraft);
            }
            setProviderOpen(false);
            setEditingId(null);
            refresh();
        } catch (error) {
            report(error);
        }
    }

    return (
        <section aria-label="Model sources">
            <h1>Models</h1>
            <p>{state.selectedSource ? `${state.selectedSource.kind === 'api' ? 'API' : 'Artifact'}: ${state.selectedSource.id}` : 'No model source selected'}</p>
            <p>{state.artifact.phase === 'selected' ? 'Selected model; not ready' : `Artifact: ${state.artifact.phase}`}</p>
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
                <p>Connection test: {state.api.testStatus}. Selection does not prove reachability.</p>
                <button type="button" onClick={() => editProvider()}>Add API provider</button>
                <ul>
                    {sources.list().api.map((provider) => (
                        <li>
                            <strong>{provider.label}</strong>
                            <span>{provider.baseUrl} · {provider.model}</span>
                            <button type="button" onClick={() => adapter.selectApi(provider.id, provider.baseUrl)}>{`Use ${provider.label}`}</button>
                            <button type="button" onClick={() => run(adapter.testApi(provider.id))}>{`Test ${provider.label}`}</button>
                            <button type="button" onClick={() => editProvider(provider)}>{`Edit ${provider.label}`}</button>
                            <button type="button" onClick={() => setRemoving(provider)}>{`Remove ${provider.label}`}</button>
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
                    <p>Remove {removing.label}? Existing conversations are preserved.</p>
                    <button type="button" autofocus onClick={() => setRemoving(null)}>Keep provider</button>
                    <button type="button" onClick={() => { sources.removeApi(removing.id); setRemoving(null); refresh(); }}>Confirm removal</button>
                </Modal>
            )}
        </section>
    );
}
