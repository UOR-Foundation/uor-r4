import { useEffect, useRef } from 'preact/hooks';

export interface ScopedShortcutHandlers {
    onPalette(): void;
    onSettings(): void;
}

function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return false;
    const tag = target.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
    const editable = target.closest('[contenteditable]');
    return editable !== null && editable.getAttribute('contenteditable') !== 'false';
}

export function useScopedShortcuts(handlers: ScopedShortcutHandlers): void {
    const handlersRef = useRef(handlers);
    handlersRef.current = handlers;

    useEffect(() => {
        const onKeyDown = (event: KeyboardEvent) => {
            if (!(event.metaKey || event.ctrlKey) || event.altKey) return;
            if (event.isComposing || event.keyCode === 229) return;
            if (isEditableTarget(event.target) || isEditableTarget(document.activeElement)) return;
            const palette = event.shiftKey && (event.key === 'p' || event.key === 'P');
            const settings = !event.shiftKey && event.key === ',';
            if (!palette && !settings) return;
            event.preventDefault();
            event.stopPropagation();
            if (palette) handlersRef.current.onPalette();
            else handlersRef.current.onSettings();
        };
        window.addEventListener('keydown', onKeyDown, true);
        return () => { window.removeEventListener('keydown', onKeyDown, true); };
    }, []);
}
