import { useState } from 'preact/hooks';
import type { StudioSettings, StudioUiApi } from '../adapters/workflow.js';
import { Modal } from './Modal.js';

interface SettingsProps {
    ui: StudioUiApi;
    onClose(): void;
    onChange?(kind: 'sessions' | 'editor'): void;
    onError?(message: string): void;
}

export function Settings({ ui, onClose, onChange, onError }: SettingsProps) {
    const [settings, setSettings] = useState<StudioSettings>(() => ui.getSettings());
    const [clearing, setClearing] = useState<'sessions' | 'editor' | null>(null);

    function update<K extends keyof StudioSettings>(key: K, value: StudioSettings[K]) {
        setSettings({ ...settings, [key]: value });
        ui.updateSettings({ [key]: value });
    }

    function confirmClear() {
        if (clearing === 'sessions') {
            const result = ui.clearSessions();
            if (result.error) onError?.(result.error);
            else if (result.cleared < result.total) onError?.(`Cleared ${result.cleared} of ${result.total} sessions; stop the current reply before deleting the rest.`);
        }
        if (clearing === 'editor') ui.clearEditor();
        setClearing(null);
        if (clearing) onChange?.(clearing);
    }

    return (
        <Modal title="Studio settings" onClose={onClose}>
            <section aria-label="Appearance">
                <h3>Appearance</h3>
                <label>Theme
                    <select aria-label="Theme" value={settings.theme} onChange={(event) => update('theme', event.currentTarget.value as StudioSettings['theme'])}>
                        <option value="dark">Dark</option><option value="light">Light</option><option value="system">System</option>
                    </select>
                </label>
                <label><input aria-label="Reduce motion" type="checkbox" checked={settings.reduceMotion} onChange={(event) => update('reduceMotion', event.currentTarget.checked)} /> Reduce motion</label>
            </section>
            <section aria-label="Editor preferences">
                <h3>Editor</h3>
                <label>Font size<input aria-label="Editor font size" type="number" min="10" max="24" value={settings.fontSize} onInput={(event) => update('fontSize', Number(event.currentTarget.value))} /></label>
                <label>Tab size<input aria-label="Tab size" type="number" min="1" max="8" value={settings.tabSize} onInput={(event) => update('tabSize', Number(event.currentTarget.value))} /></label>
                <label><input aria-label="Word wrap" type="checkbox" checked={settings.wordWrap} onChange={(event) => update('wordWrap', event.currentTarget.checked)} /> Word wrap</label>
            </section>
            <section aria-label="Data controls">
                <h3>Data</h3>
                <p>Clearing uses the existing session and editor stores. Provider settings are not included.</p>
                <button type="button" onClick={() => setClearing('sessions')}>Clear sessions</button>
                <button type="button" onClick={() => setClearing('editor')}>Clear editor tabs</button>
                {clearing && (
                    <section aria-label="Confirm data clearing">
                        <p>{clearing === 'sessions' ? 'Clear all local sessions?' : 'Close all editor tabs?'}</p>
                        <button type="button" autofocus onClick={() => setClearing(null)}>Cancel</button>
                        <button type="button" onClick={confirmClear}>{clearing === 'sessions' ? 'Confirm clear sessions' : 'Confirm clear editor tabs'}</button>
                    </section>
                )}
            </section>
            <section aria-label="About">
                <h3>About</h3>
                <p>UOR-R4 Studio is pre-alpha. M1 supports next-token prediction, not general conversation.</p>
            </section>
        </Modal>
    );
}
