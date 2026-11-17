import { useEffect, useRef, useState } from 'preact/hooks';
import type { SessionApi } from '../adapters/workflow.js';
import { GENERATION_FINISHED_EVENT } from '../adapters/generation.js';

type Candidate = { id: number; text: string };

export function DiscretePredictor({ sessions, sha256 }: { sessions: SessionApi; sha256: string }) {
    const [prompt, setPrompt] = useState('');
    const [busy, setBusy] = useState(false);
    const [tokens, setTokens] = useState<Candidate[]>([]);
    const [failure, setFailure] = useState('');
    const pending = useRef<number | null>(null);

    useEffect(() => {
        const finish = (event: Event) => {
            const { generationId, outcome, error } = (event as CustomEvent).detail || {};
            if (pending.current === null || generationId !== pending.current) return;
            pending.current = null;
            setBusy(false);
            const result = sessions.predictionResult?.(generationId);
            if (outcome === 'complete' && result?.artifactSha256 === sha256) setTokens(result.tokens);
            else setFailure(error || 'No prediction was returned for this model.');
        };
        document.addEventListener(GENERATION_FINISHED_EVENT, finish);
        return () => document.removeEventListener(GENERATION_FINISHED_EVENT, finish);
    }, [sessions, sha256]);

    function predict(event: Event) {
        event.preventDefault();
        if (pending.current !== null || !prompt.trim()) return;
        setTokens([]);
        setFailure('');
        const id = sessions.predict(prompt);
        if (id === false) {
            setFailure('Prediction could not start. Check that the model has a matching tokenizer and no generation is active.');
            return;
        }
        pending.current = id;
        setBusy(true);
    }

    return <section aria-label="Next-token predictor">
        <h2>Next-token predictor</h2>
        <form onSubmit={predict}>
            <label>Prediction prefix<textarea value={prompt} onInput={event => setPrompt(event.currentTarget.value)} /></label>
            <button type="submit" disabled={busy || !prompt.trim()}>Predict next tokens</button>
        </form>
        {failure && <p role="alert">{failure}</p>}
        {tokens.length > 0 && <section aria-label="Next-token predictions" aria-live="polite">
            <h3>Top next tokens</h3><ol>{tokens.map(({ id, text }) => <li key={id}>ID {id} · <code>{JSON.stringify(text)}</code></li>)}</ol>
        </section>}
    </section>;
}
