import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
// @ts-expect-error Plain-JavaScript workspace controller.
import { mountMcpConnections } from '../../assets/js/mcp_client.js';

beforeEach(() => {
    vi.resetModules();
    window.localStorage.clear();
    document.body.innerHTML = '<div id="workspaceSelector"></div><div id="workspaceTree"></div><div id="workspaceFiles"></div><div id="openFiles"></div><div id="recentFiles"></div><input id="editorName"><textarea id="editorText"></textarea><div id="editorStats"></div><div id="editorSaveState"></div><div id="toastRegion"></div><div id="mcpMount"></div><button id="ctxOpenFiles"></button><div id="commandPalette" hidden><input id="paletteInput"><div id="paletteList"></div></div><div id="chatMessages"></div><div id="conversationList"></div><div id="statusClock"></div><div id="modelSourceMount"></div>';
});

afterEach(() => {
    delete (window as unknown as { __studioFiles?: unknown }).__studioFiles;
    delete (window as unknown as { __studioUi?: unknown }).__studioUi;
    delete (window as unknown as { UORDesktopActions?: unknown }).UORDesktopActions;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
    document.body.innerHTML = '';
});

describe('exact-head repair regressions', () => {
    it('saves a native tab to its original grant after a second project opens', async () => {
        const writes: Array<{ path: string; content: string }> = [];
        let root = '/project-a';
        (window as unknown as { UORDesktopActions: unknown }).UORDesktopActions = {
            available: true,
            chooseProjectFolder: vi.fn(async () => ({ path: root })),
            listDirectory: vi.fn(async () => ['note.txt']),
            readFile: vi.fn(async (path: string) => path.endsWith('note.txt') ? 'original' : ''),
            writeFile: vi.fn(async (path: string, content: string) => { writes.push({ path, content }); }),
        };
        // @ts-expect-error The plain-JavaScript workspace controller has no declaration file.
        await import('../../assets/js/studio_files.js');
        const files = (window as unknown as { __studioFiles: { openFolder(): Promise<void>; openFile(path: string): Promise<void>; saveActive(): Promise<boolean> } }).__studioFiles;
        await files.openFolder();
        await files.openFile('note.txt');
        (document.getElementById('editorText') as HTMLTextAreaElement).value = 'from project A';
        root = '/project-b';
        await files.openFolder();
        expect(await files.saveActive()).toBe(true);
        expect(writes).toEqual([{ path: '/project-a/note.txt', content: 'from project A' }]);
    });

    it('keeps same-named native tabs bound to their own project roots', async () => {
        const writes: Array<{ path: string; content: string }> = [];
        let root = '/project-a';
        (window as unknown as { UORDesktopActions: unknown }).UORDesktopActions = {
            available: true,
            chooseProjectFolder: vi.fn(async () => ({ path: root })),
            listDirectory: vi.fn(async () => ['note.txt']),
            readFile: vi.fn(async (path: string) => path === '/project-a/note.txt' ? 'from project A' : 'from project B'),
            writeFile: vi.fn(async (path: string, content: string) => { writes.push({ path, content }); }),
        };
        // @ts-expect-error The plain-JavaScript workspace controller has no declaration file.
        await import('../../assets/js/studio_files.js');
        const files = (window as unknown as { __studioFiles: {
            openFolder(): Promise<void>;
            openFile(path: string, rootPath?: string): Promise<void>;
            saveActive(): Promise<boolean>;
            tabs(): Array<{ path: string; rootPath: string }>;
        } }).__studioFiles;
        await files.openFolder();
        await files.openFile('note.txt');
        const editor = document.getElementById('editorText') as HTMLTextAreaElement;
        editor.value = 'project A edited';
        editor.dispatchEvent(new Event('input', { bubbles: true }));
        root = '/project-b';
        await files.openFolder();
        await files.openFile('note.txt');
        expect(editor.value).toBe('from project B');
        expect(files.tabs().map((tab) => tab.rootPath)).toEqual(['/project-a', '/project-b']);
        await files.openFile('note.txt', '/project-a');
        expect(editor.value).toBe('project A edited');
        expect(await files.saveActive()).toBe(true);
        expect(writes).toEqual([{ path: '/project-a/note.txt', content: 'project A edited' }]);
    });

    it('disconnects a native MCP session when tool listing fails', async () => {
        const session = {
            listTools: vi.fn(async () => { throw new Error('list failed'); }),
            disconnect: vi.fn(async () => undefined),
        };
        (window as unknown as { __TAURI__: unknown }).__TAURI__ = { core: { invoke: vi.fn(), Channel: class {} } };
        mountMcpConnections({ connectRemote: async () => session });
        (document.getElementById('mcpConnectButton') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('list failed'));
        expect(session.disconnect).toHaveBeenCalledOnce();
    });

    it('uses real Files, Changes, Terminal and Guide navigation commands', async () => {
        window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
        await import('../../assets/js/studio_ui.js');
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.waitFor(() => expect((window as unknown as { __studioUi?: unknown }).__studioUi).toBeDefined());
        const ui = (window as unknown as { __studioUi: { openPalette(): void } }).__studioUi;
        Element.prototype.scrollIntoView = vi.fn();
        ui.openPalette();
        const commands = document.getElementById('paletteList')!.textContent || '';
        expect(commands).toContain('Go to Changes');
        expect(commands).toContain('Go to Terminal');
        expect(commands).toContain('Go to Guide');
        expect(commands).toContain('⌘6');
        (document.getElementById('ctxOpenFiles') as HTMLButtonElement).click();
        expect(document.body.getAttribute('data-section')).toBe('workspace');
    });
});
