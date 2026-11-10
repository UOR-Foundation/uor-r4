interface LegacyStudioUi {
    closePalette?(restoreFocus?: boolean): void;
}

interface LegacyStudioWindow extends Window {
    __studioUi?: LegacyStudioUi;
}

const paletteHashes = new Set(['#command-palette', '#palette']);

function paletteNode(doc: Document): HTMLElement | null {
    return doc.getElementById('commandPalette');
}

function closeLegacyPalette(view: Window): void {
    const ui = (view as LegacyStudioWindow).__studioUi;
    if (ui && typeof ui.closePalette === 'function') ui.closePalette(false);
    const palette = paletteNode(view.document);
    if (palette && !palette.hidden) palette.hidden = true;
}

function clearPaletteHash(view: Window): boolean {
    if (!paletteHashes.has(view.location.hash)) return false;
    view.history.replaceState(view.history.state, '', `${view.location.pathname}${view.location.search}`);
    return true;
}

export interface LegacyShortcutGuard {
    dispose(): void;
}

export function installLegacyShortcutGuard(root: HTMLElement): LegacyShortcutGuard {
    const doc = root.ownerDocument;
    const view = doc.defaultView;
    if (!view) return { dispose() {} };

    closeLegacyPalette(view);
    clearPaletteHash(view);

    const palette = paletteNode(doc);
    const observer = palette
        ? new MutationObserver(() => { if (!palette.hidden) closeLegacyPalette(view); })
        : null;
    if (palette && observer) observer.observe(palette, { attributes: true, attributeFilter: ['hidden'] });

    const blockPaletteShortcut = (event: KeyboardEvent) => {
        if (!(event.metaKey || event.ctrlKey) || event.altKey) return;
        if (event.key !== 'k' && event.key !== 'K') return;
        event.preventDefault();
        event.stopImmediatePropagation();
        closeLegacyPalette(view);
    };
    const closeOnPaletteHash = () => { if (clearPaletteHash(view)) closeLegacyPalette(view); };
    const closeOnLegacyReady = () => closeLegacyPalette(view);

    view.addEventListener('keydown', blockPaletteShortcut, true);
    view.addEventListener('hashchange', closeOnPaletteHash);
    doc.addEventListener('uor-studio-ready', closeOnLegacyReady);

    return {
        dispose() {
            observer?.disconnect();
            view.removeEventListener('keydown', blockPaletteShortcut, true);
            view.removeEventListener('hashchange', closeOnPaletteHash);
            doc.removeEventListener('uor-studio-ready', closeOnLegacyReady);
        },
    };
}
