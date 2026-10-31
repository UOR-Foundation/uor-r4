// UOR-R4 Studio — agent-console interaction layer.
// Theme, command palette, collapsible chrome, live status mirrors and view plumbing.
// No frameworks, no network, no external assets. Loaded as a module after the inline app script.

(function () {
    'use strict';

    var UI_KEY = 'uor.studio.ui.v1';
    var THEME_KEY = 'uor.studio.theme';
    var app = document.getElementById('app');
    var ready = false;

    function $(selector, root) { return (root || document).querySelector(selector); }
    function $all(selector, root) { return Array.prototype.slice.call((root || document).querySelectorAll(selector)); }

    function readState() {
        try {
            var raw = window.localStorage.getItem(UI_KEY);
            var parsed = raw ? JSON.parse(raw) : null;
            return parsed && typeof parsed === 'object' ? parsed : {};
        } catch (error) {
            return {};
        }
    }

    function writeState(patch) {
        try {
            var state = readState();
            Object.keys(patch).forEach(function (key) { state[key] = patch[key]; });
            window.localStorage.setItem(UI_KEY, JSON.stringify(state));
        } catch (error) {
            /* storage is optional */
        }
    }

    function uiValue(name, fallback) {
        var stored = readState()[name];
        return stored === 'expanded' || stored === 'collapsed' ? stored : fallback;
    }

    function setChrome(name, value, persist) {
        if (!app) return;
        app.setAttribute('data-' + name, value);
        var expanded = value === 'expanded';
        var map = {
            rail: ['btnToggleRail'],
            sidebar: ['btnToggleSidebar', 'btnMobileMenu'],
            inspector: ['btnToggleInspector', 'btnToggleInspector2', 'btnToggleInspector3']
        };
        (map[name] || []).forEach(function (id) {
            var button = document.getElementById(id);
            if (button) button.setAttribute('aria-expanded', expanded ? 'true' : 'false');
        });
        if (persist) {
            var patch = {};
            patch[name] = value;
            writeState(patch);
        }
    }

    function toggleChrome(name) {
        if (!app) return;
        var current = app.getAttribute('data-' + name) === 'collapsed' ? 'collapsed' : 'expanded';
        setChrome(name, current === 'expanded' ? 'collapsed' : 'expanded', true);
        if (name === 'sidebar' && window.matchMedia('(max-width: 900px)').matches) {
            if (app.getAttribute('data-sidebar') === 'expanded') setChrome('inspector', 'collapsed', false);
        }
        if (name === 'inspector' && window.matchMedia('(max-width: 1180px)').matches) {
            if (app.getAttribute('data-inspector') === 'expanded') setChrome('sidebar', 'collapsed', false);
        }
    }

    /* ------------------------------------------------------------ theme */

    function currentTheme() {
        var value = document.documentElement.getAttribute('data-theme');
        return value === 'light' ? 'light' : 'dark';
    }

    function applyTheme(theme, persist) {
        var next = theme === 'light' ? 'light' : 'dark';
        document.documentElement.setAttribute('data-theme', next);
        document.documentElement.style.colorScheme = next;
        if (persist) {
            try { window.localStorage.setItem(THEME_KEY, next); } catch (error) { /* optional */ }
        }
        $all('[data-theme-label]').forEach(function (node) { node.textContent = next === 'light' ? 'Dark' : 'Light'; });
        var button = document.getElementById('btnTheme');
        if (button) button.setAttribute('title', next === 'light' ? 'Switch to dark theme' : 'Switch to light theme');
        var rail = document.getElementById('railTheme');
        if (rail) rail.setAttribute('title', next === 'light' ? 'Switch to dark theme' : 'Switch to light theme');
    }

    function toggleTheme() {
        applyTheme(currentTheme() === 'light' ? 'dark' : 'light', true);
        if (paletteOpen) buildPalette($('#paletteInput') ? $('#paletteInput').value : '');
    }

    /* ------------------------------------------------------------ toast / help */

    function toast(text, tone) {
        var hooks = window.__studioUi;
        if (hooks && typeof hooks.toast === 'function') {
            hooks.toast(text, tone || 'info');
            return;
        }
        var region = document.getElementById('toastRegion');
        if (!region) return;
        var node = document.createElement('div');
        node.className = 'toast';
        node.setAttribute('data-tone', tone || 'info');
        node.setAttribute('role', 'status');
        node.textContent = text;
        region.appendChild(node);
        window.setTimeout(function () { node.remove(); }, 4200);
    }

    var helpDialog = null;
    function openHelp() {
        if (helpDialog) { helpDialog.hidden = false; return; }
        helpDialog = document.createElement('div');
        helpDialog.className = 'help-backdrop';
        helpDialog.id = 'helpDialog';
        helpDialog.setAttribute('role', 'dialog');
        helpDialog.setAttribute('aria-modal', 'true');
        helpDialog.setAttribute('aria-labelledby', 'helpTitle');
        helpDialog.innerHTML = '';
        var dialog = document.createElement('div');
        dialog.className = 'help-dialog';
        var title = document.createElement('h2');
        title.id = 'helpTitle';
        title.textContent = 'Keyboard & console';
        var list = document.createElement('ul');
        list.className = 'shortcut-list';
        [
            ['⌘K / Ctrl+K', 'Command palette'],
            ['Enter', 'Send chat message'],
            ['Shift+Enter', 'New line in composer'],
            ['Esc', 'Stop generation / close palette'],
            ['⌘B / Ctrl+B', 'Toggle sessions sidebar'],
            ['⌘I / Ctrl+I', 'Toggle context inspector'],
            ['Model view', 'Full model-source manager']
        ].forEach(function (row) {
            var item = document.createElement('li');
            var kbd = document.createElement('kbd');
            kbd.className = 'kbd';
            kbd.textContent = row[0];
            var span = document.createElement('span');
            span.textContent = row[1];
            item.appendChild(kbd);
            item.appendChild(span);
            list.appendChild(item);
        });
        var close = document.createElement('button');
        close.type = 'button';
        close.className = 'primary-btn primary-btn-sm';
        close.textContent = 'Close';
        close.addEventListener('click', function () { helpDialog.hidden = true; });
        dialog.appendChild(title);
        dialog.appendChild(list);
        dialog.appendChild(close);
        helpDialog.appendChild(dialog);
        helpDialog.addEventListener('click', function (event) { if (event.target === helpDialog) helpDialog.hidden = true; });
        document.body.appendChild(helpDialog);
    }

    /* ------------------------------------------------------------ command palette */

    var paletteOpen = false;
    var paletteIndex = 0;
    var paletteCommands = [];
    var lastFocus = null;

    function hook(name) {
        var ui = window.__studioUi;
        return ui && typeof ui[name] === 'function' ? ui[name] : null;
    }

    function clickFirst(selector, fallbackView) {
        var node = $(selector);
        if (node) { node.click(); return true; }
        var setView = hook('setView');
        if (setView && fallbackView) { setView(fallbackView); return true; }
        return false;
    }

    function openPalette() {
        var backdrop = document.getElementById('commandPalette');
        if (!backdrop || paletteOpen) return;
        paletteOpen = true;
        lastFocus = document.activeElement;
        backdrop.hidden = false;
        var input = document.getElementById('paletteInput');
        if (input) { input.value = ''; input.focus(); }
        buildPalette('');
    }

    function closePalette() {
        var backdrop = document.getElementById('commandPalette');
        if (!backdrop || !paletteOpen) return;
        paletteOpen = false;
        backdrop.hidden = true;
        if (lastFocus && typeof lastFocus.focus === 'function') lastFocus.focus();
        lastFocus = null;
    }

    function commands() {
        var list = [
            { group: 'ACTIONS', id: 'new-chat', label: 'New chat', hint: 'N', run: function () { var h = hook('startNewChat'); if (h) h(); else clickFirst('#btnNewChatTop'); } },
            { group: 'ACTIONS', id: 'synthetic-demo', label: 'Load synthetic demo', hint: 'offline', run: function () { if (!clickFirst('[data-action="load-bundled"]')) { setViewHook('model'); toast('The synthetic demo is loading in Model sources.', 'info'); } } },
            { group: 'ACTIONS', id: 'm1-model', label: 'Load M1 model (seed 1)', hint: '86 MB', run: function () { if (!clickFirst('[data-action="load-published"][data-model-id="published-m1-trunk-seed1"]')) { setViewHook('model'); toast('Open Model sources and load the M1 model card (it appears after the catalog loads).', 'info'); } } },
            { group: 'ACTIONS', id: 'local-file', label: 'Load local model file…', hint: 'UORLUT01', run: function () { if (!clickFirst('[data-role="model-file"], #modelFileInput')) { setViewHook('model'); toast('Open Model sources → Local file.', 'info'); } } },
            { group: 'ACTIONS', id: 'load-url', label: 'Load model from URL…', hint: 'http(s)', run: function () { setViewHook('model'); window.setTimeout(function () { var input = document.getElementById('modelUrlInput'); if (input) input.focus(); else toast('Open Model sources → From URL.', 'info'); }, 60); } },
            { group: 'ACTIONS', id: 'add-api', label: 'Add API source…', hint: 'OpenAI', run: function () { setViewHook('model'); window.setTimeout(function () { var button = document.querySelector('[data-action="add-api"]'); if (button) button.click(); else toast('Open Model sources → OpenAI-compatible API.', 'info'); }, 60); } },
            { group: 'ACTIONS', id: 'model-sources', label: 'Open model sources', hint: '', run: function () { setViewHook('model'); } },
            { group: 'VIEW', id: 'view-chat', label: 'Switch: Chat', hint: '1', run: function () { setViewHook('chat'); } },
            { group: 'VIEW', id: 'view-model', label: 'Switch: Model', hint: '2', run: function () { setViewHook('model'); } },
            { group: 'VIEW', id: 'view-editor', label: 'Switch: Editor', hint: '3', run: function () { setViewHook('editor'); } },
            { group: 'INTERFACE', id: 'toggle-theme', label: 'Toggle theme', hint: currentTheme() === 'light' ? 'light' : 'dark', run: toggleTheme },
            { group: 'INTERFACE', id: 'toggle-sidebar', label: 'Toggle sessions sidebar', hint: '⌘B', run: function () { toggleChrome('sidebar'); } },
            { group: 'INTERFACE', id: 'toggle-inspector', label: 'Toggle context inspector', hint: '⌘I', run: function () { toggleChrome('inspector'); } },
            { group: 'INTERFACE', id: 'focus-composer', label: 'Focus composer', hint: 'Enter', run: function () { var h = hook('focusComposer'); if (h) h(); else { var input = document.getElementById('chatInput'); if (input) input.focus(); } } },
            { group: 'INTERFACE', id: 'help', label: 'Keyboard help', hint: '?', run: openHelp },
            { group: 'INTERFACE', id: 'toggle-rail', label: 'Toggle icon rail', hint: '', run: function () { toggleChrome('rail'); } }
        ];
        return list;
    }

    function setViewHook(view) {
        var setView = hook('setView');
        if (setView) { setView(view); return; }
        var button = document.getElementById('viewBtn' + view.charAt(0).toUpperCase() + view.slice(1));
        if (button) button.click();
    }

    function filterCommands(query) {
        var base = commands();
        var q = String(query || '').trim().toLowerCase();
        if (!q) return base;
        return base.filter(function (command) {
            return (command.label + ' ' + command.group + ' ' + command.id).toLowerCase().indexOf(q) >= 0;
        });
    }

    function buildPalette(query) {
        var list = document.getElementById('paletteList');
        if (!list) return;
        paletteCommands = filterCommands(query);
        if (paletteIndex >= paletteCommands.length) paletteIndex = Math.max(0, paletteCommands.length - 1);
        if (paletteCommands.length === 0) {
            list.textContent = '';
            var empty = document.createElement('div');
            empty.className = 'palette-empty';
            empty.textContent = 'No matching commands.';
            list.appendChild(empty);
            return;
        }
        var fragment = document.createDocumentFragment();
        var group = null;
        paletteCommands.forEach(function (command, index) {
            if (command.group !== group) {
                group = command.group;
                var label = document.createElement('div');
                label.className = 'palette-group-label';
                label.textContent = group;
                fragment.appendChild(label);
            }
            var item = document.createElement('button');
            item.type = 'button';
            item.className = 'palette-item';
            item.id = 'palette-item-' + index;
            item.setAttribute('role', 'option');
            item.setAttribute('aria-selected', index === paletteIndex ? 'true' : 'false');
            var labelNode = document.createElement('span');
            labelNode.className = 'palette-item-label';
            labelNode.textContent = command.label;
            var hintNode = document.createElement('span');
            hintNode.className = 'palette-item-hint';
            hintNode.textContent = command.hint || '';
            item.appendChild(labelNode);
            item.appendChild(hintNode);
            item.addEventListener('mouseenter', function () {
                paletteIndex = index;
                syncPaletteSelection();
            });
            item.addEventListener('click', function () { runPalette(index); });
            fragment.appendChild(item);
        });
        list.textContent = '';
        list.appendChild(fragment);
        var input = document.getElementById('paletteInput');
        if (input) input.setAttribute('aria-activedescendant', 'palette-item-' + paletteIndex);
    }

    function syncPaletteSelection() {
        $all('.palette-item').forEach(function (node, index) {
            node.setAttribute('aria-selected', index === paletteIndex ? 'true' : 'false');
        });
        var input = document.getElementById('paletteInput');
        if (input) input.setAttribute('aria-activedescendant', 'palette-item-' + paletteIndex);
        var selected = document.getElementById('palette-item-' + paletteIndex);
        if (selected && selected.scrollIntoView) selected.scrollIntoView({ block: 'nearest' });
    }

    function runPalette(index) {
        var command = paletteCommands[index];
        if (!command) return;
        closePalette();
        window.setTimeout(function () {
            try { command.run(); } catch (error) { toast(String(error && error.message ? error.message : error), 'error'); }
        }, 20);
    }

    function movePalette(delta) {
        if (paletteCommands.length === 0) return;
        paletteIndex = (paletteIndex + delta + paletteCommands.length) % paletteCommands.length;
        syncPaletteSelection();
    }

    /* ------------------------------------------------------------ view sync + model panel reparenting */

    function syncView(view) {
        var railMap = { chat: 'railChat', model: 'railModel', editor: 'railEditor' };
        Object.keys(railMap).forEach(function (key) {
            var button = document.getElementById(railMap[key]);
            if (!button) return;
            var active = key === view;
            button.setAttribute('aria-pressed', active ? 'true' : 'false');
            button.classList.toggle('is-active', active);
        });
        if (window.matchMedia('(max-width: 900px)').matches) setChrome('sidebar', 'collapsed', false);
        if (window.matchMedia('(max-width: 1180px)').matches) setChrome('inspector', 'collapsed', false);
        if (window.matchMedia('(max-width: 780px)').matches) window.scrollTo(0, 0);
    }

    /* ------------------------------------------------------------ live status mirrors */

    function text(selector) {
        var node = $(selector);
        return node ? String(node.textContent || '').replace(/\s+/g, ' ').trim() : '';
    }

    function setText(selector, value, prefix) {
        var node = $(selector);
        if (!node) return;
        var next = (prefix || '') + String(value == null ? '' : value);
        if (node.textContent !== next) node.textContent = next;
    }

    function mirror() {
        var engine = text('#engineStatus').replace(/^Engine:\s*/i, '');
        var model = text('#modelStatus').replace(/^Model:\s*/i, '');
        var tokenizer = text('#footerTokenizer').replace(/^Tokenizer:\s*/i, '');
        var privacy = text('#privacyStatus');
        var storage = text('#footerStorage').replace(/^Storage:\s*/i, '');
        setText('#envEngine', engine || 'starting');
        setText('#envModel', model || 'none');
        setText('#envTokenizer', tokenizer || 'awaiting model');
        setText('#ctxEngine', engine || 'starting');
        setText('#ctxTokenizer', tokenizer || 'awaiting model');
        setText('#ctxPrivacy', /api/i.test(privacy) ? 'api' : 'local only');
        setText('#ctxStorage', storage || 'unknown');
        setText('#statusPrivacy', /api/i.test(privacy) ? 'API' : 'LOCAL');
        var statusPrivacy = document.getElementById('statusPrivacy');
        if (statusPrivacy) statusPrivacy.style.color = /api/i.test(privacy) ? 'var(--err)' : 'var(--accent)';
        var tokenizerFull = text('#footerTokenizer');
        if (/weights\/token/i.test(tokenizerFull)) {
            var parts = tokenizerFull.split('·');
            setText('#ctxWeights', parts[parts.length - 1].replace(/^\s*weights\/token:\s*/i, '').trim() || '—');
        }
        setText('#ctxModelLabel', model || 'No model loaded', '');
        var ctxLabel = document.getElementById('ctxModelLabel');
        if (ctxLabel) ctxLabel.textContent = model ? model.replace(/ · .*$/, '') : 'No model loaded';
        var meta = text('#footerTokenizer');
        setText('#ctxModelMeta', model ? (meta || 'loaded') : 'Load the synthetic demo or a trained stack artifact.');
        var sha = document.documentElement.dataset.modelSha256 || '';
        var shaNode = document.getElementById('ctxModelSha');
        if (shaNode) {
            shaNode.hidden = !sha;
            if (sha) shaNode.textContent = 'sha256 ' + sha.slice(0, 16) + '…';
        }
        var dot = document.getElementById('ctxModelDot');
        if (dot) {
            dot.className = 'dot ' + (model && !/no model/i.test(model) ? 'dot-ok' : 'dot-muted');
        }
        var checkpoint = document.getElementById('ctxCheckpoint');
        if (checkpoint) setText('#ctxCheckpoint', document.documentElement.dataset.modelSchema ? 'ready' : '—');
    }

    function clock() {
        var node = document.getElementById('statusClock');
        if (!node) return;
        var now = new Date();
        function pad(value) { return value < 10 ? '0' + value : String(value); }
        node.textContent = pad(now.getHours()) + ':' + pad(now.getMinutes()) + ':' + pad(now.getSeconds());
    }

    /* ------------------------------------------------------------ wiring */

    function on(id, event, handler) {
        var node = document.getElementById(id);
        if (node) node.addEventListener(event, handler);
    }

    function wire() {
        on('btnToggleRail', 'click', function () { toggleChrome('rail'); });
        on('btnToggleSidebar', 'click', function () { toggleChrome('sidebar'); });
        on('btnSidebarCollapse', 'click', function () { toggleChrome('sidebar'); });
        on('btnMobileMenu', 'click', function () { toggleChrome('sidebar'); });
        ['btnToggleInspector', 'btnToggleInspector2', 'btnToggleInspector3', 'btnInspectorCollapse'].forEach(function (id) {
            on(id, 'click', function () { toggleChrome('inspector'); });
        });
        ['btnTheme', 'railTheme'].forEach(function (id) { on(id, 'click', toggleTheme); });
        on('railHelp', 'click', openHelp);
        ['btnNewChatTop', 'railSessionsBottom'].forEach(function (id) {
            on(id, 'click', function () { var start = hook('startNewChat'); if (start) start(); });
        });
        ['railChat', 'railModel', 'railEditor'].forEach(function (id, index) {
            on(id, 'click', function () { setViewHook(['chat', 'model', 'editor'][index]); });
        });
        on('railPalette', 'click', openPalette);
        on('commandSearch', 'click', openPalette);
        on('ctxLoadModel', 'click', function () { setViewHook('model'); });

        var palette = document.getElementById('commandPalette');
        if (palette) {
            palette.addEventListener('click', function (event) { if (event.target === palette) closePalette(); });
        }
        var paletteInput = document.getElementById('paletteInput');
        if (paletteInput) {
            paletteInput.addEventListener('input', function () {
                paletteIndex = 0;
                buildPalette(paletteInput.value);
            });
            paletteInput.addEventListener('keydown', function (event) {
                if (event.key === 'ArrowDown') { event.preventDefault(); movePalette(1); }
                else if (event.key === 'ArrowUp') { event.preventDefault(); movePalette(-1); }
                else if (event.key === 'Enter') { event.preventDefault(); runPalette(paletteIndex); }
                else if (event.key === 'Escape') { event.preventDefault(); closePalette(); }
            });
        }

        document.addEventListener('keydown', function (event) {
            var meta = event.metaKey || event.ctrlKey;
            if (meta && (event.key === 'k' || event.key === 'K')) {
                event.preventDefault();
                if (paletteOpen) closePalette(); else openPalette();
                return;
            }
            if (meta && (event.key === 'b' || event.key === 'B')) { event.preventDefault(); toggleChrome('sidebar'); return; }
            if (meta && (event.key === 'i' || event.key === 'I')) { event.preventDefault(); toggleChrome('inspector'); return; }
            if (event.key === 'Escape') {
                if (paletteOpen) { event.preventDefault(); event.stopPropagation(); closePalette(); return; }
                if (helpDialog && !helpDialog.hidden) { event.preventDefault(); event.stopPropagation(); helpDialog.hidden = true; }
            }
        }, true);

        var search = document.getElementById('sessionSearch');
        if (search) {
            search.addEventListener('input', function () {
                var query = search.value.trim().toLowerCase();
                $all('#conversationList .conversation-item').forEach(function (item) {
                    item.style.display = !query || item.textContent.toLowerCase().indexOf(query) >= 0 ? '' : 'none';
                });
            });
        }

        document.addEventListener('uor-view-changed', function (event) {
            var view = event && event.detail ? event.detail.view : 'chat';
            syncView(view);
        });
        document.addEventListener('uor-model-source-changed', function () {
            window.setTimeout(mirror, 80);
        });
        window.addEventListener('pagehide', function () {
            if (clockTimer) window.clearInterval(clockTimer);
        });
    }

    var clockTimer = null;

    function initialChrome() {
        var wide = window.matchMedia('(min-width: 1181px)').matches;
        var medium = window.matchMedia('(min-width: 901px)').matches;
        setChrome('rail', uiValue('rail', 'expanded'), false);
        setChrome('sidebar', uiValue('sidebar', medium ? 'expanded' : 'collapsed'), false);
        setChrome('inspector', uiValue('inspector', wide ? 'expanded' : 'collapsed'), false);
    }

    function boot() {
        if (ready) return;
        ready = true;
        initialChrome();
        applyTheme(currentTheme(), false);
        var params = new URLSearchParams(window.location.search);
        if (params.get('theme') === 'light' || params.get('theme') === 'dark') applyTheme(params.get('theme'), false);
        if (window.location.hash === '#theme=light' || window.location.hash === '#theme=dark') applyTheme(window.location.hash.split('=')[1], false);
        wire();
        syncView(document.body.dataset.view || 'chat');
        mirror();
        clock();
        clockTimer = window.setInterval(clock, 1000);
        window.setInterval(mirror, 1200);
        var hash = window.location.hash || '';
        if (hash === '#command-palette' || hash === '#palette') {
            window.setTimeout(openPalette, 120);
        } else if (/^#view=(chat|model|editor)$/.test(hash)) {
            window.setTimeout(function () { setViewHook(hash.split('=')[1]); }, 60);
        }
        window.__studioUi = Object.assign(window.__studioUi || {}, {
            toggleTheme: toggleTheme,
            setTheme: function (theme) { applyTheme(theme, true); },
            getTheme: currentTheme,
            openPalette: openPalette,
            closePalette: closePalette,
            toggleSidebar: function () { toggleChrome('sidebar'); },
            toggleInspector: function () { toggleChrome('inspector'); },
            toggleRail: function () { toggleChrome('rail'); },
            openHelp: openHelp
        });
    }

    if (document.documentElement.dataset.studioReady === '1') {
        window.setTimeout(boot, 0);
    } else {
        document.addEventListener('uor-studio-ready', boot, { once: true });
        window.setTimeout(boot, 2500);
    }
}());
