#!/usr/bin/env node
// Page-load zero-external-request verification for the Studio pages.
// Complements verify_studio_worker.mjs (which traps the worker's network APIs):
// this checks every load-time resource reference in the Studio HTML pages and
// asserts it resolves on-origin. Navigation links (<a href>) are not load
// requests; user-initiated API calls are judged separately by the item record.
//
// Usage: node verify_studio_page.mjs [--strict-router]

import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

const LOAD_ATTRS = ['src', 'srcset', 'poster', 'data'];
const LINK_RELS_LOADING = new Set([
    'stylesheet', 'icon', 'shortcut icon', 'apple-touch-icon', 'manifest',
    'preload', 'prefetch', 'modulepreload', 'apple-touch-startup-image',
]);

export function loadReferences(html) {
    const refs = [];
    const tagRe = /<([a-zA-Z][a-zA-Z0-9-]*)\b([^>]*)>/g;
    let m;
    while ((m = tagRe.exec(html)) !== null) {
        const tag = m[1];
        const attrs = m[2];
        const attrRe = /([a-zA-Z_:][-a-zA-Z0-9_:.]*)\s*=\s*("([^"]*)"|'([^']*)'|([^\s"'>]+))/g;
        const parsed = {};
        let a;
        while ((a = attrRe.exec(attrs)) !== null) {
            parsed[a[1].toLowerCase()] = a[3] ?? a[4] ?? a[5] ?? '';
        }
        if (tag.toLowerCase() === 'link') {
            const rel = String(parsed.rel || '').toLowerCase().trim();
            if (LINK_RELS_LOADING.has(rel) && parsed.href) refs.push({ tag, attr: 'href', value: parsed.href, rel });
            continue;
        }
        for (const attr of LOAD_ATTRS) {
            if (!parsed[attr]) continue;
            if (attr === 'srcset') {
                for (const part of parsed[attr].split(',')) {
                    const url = part.trim().split(/\s+/)[0];
                    if (url) refs.push({ tag, attr, value: url });
                }
            } else {
                refs.push({ tag, attr, value: parsed[attr] });
            }
        }
    }
    const styleRe = /url\(\s*['"]?([^'")]+)['"]?\s*\)/g;
    while ((m = styleRe.exec(html)) !== null) refs.push({ tag: 'css', attr: 'url()', value: m[1] });
    return refs;
}

export function isExternal(value) {
    const v = String(value).trim();
    if (!v) return false;
    if (v.startsWith('#')) return false;
    if (v.startsWith('data:')) return false;
    if (v.startsWith('blob:')) return false;
    if (v.startsWith('mailto:') || v.startsWith('tel:')) return false;
    if (v.startsWith('//')) return true;
    if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(v)) return true;
    return false;
}

function scanPage(name, html) {
    return loadReferences(html).filter((r) => isExternal(r.value))
        .map((r) => ({ page: name, tag: r.tag, attr: r.attr, value: r.value }));
}

function selfTest() {
    const fixture = [
        '<link rel="stylesheet" href="./local.css">',
        '<link rel="icon" href="https://cdn.example.invalid/favicon.ico">',
        '<script src="//cdn.example.invalid/x.js"></script>',
        '<img srcset="./a.png 1x, https://cdn.example.invalid/b.png 2x">',
        '<a href="https://github.com/UOR-Foundation/uor-r4">nav link is fine</a>',
        '<div style="background:url(\'https://cdn.example.invalid/bg.png\')"></div>',
        '<link rel="preload" href="./font.woff2" as="font">',
    ].join('\n');
    const values = scanPage('fixture.html', fixture).map((f) => f.value).sort();
    const expected = ['//cdn.example.invalid/x.js', 'https://cdn.example.invalid/b.png', 'https://cdn.example.invalid/bg.png', 'https://cdn.example.invalid/favicon.ico'].sort();
    if (JSON.stringify(values) !== JSON.stringify(expected)) {
        console.error('FAIL: scanner self-test mismatch:', JSON.stringify({ values, expected }));
        process.exit(1);
    }
    if (values.some((v) => v.includes('github.com'))) {
        console.error('FAIL: scanner treated an <a href> navigation link as a load reference');
        process.exit(1);
    }
}

selfTest();

const strictRouter = process.argv.includes('--strict-router');
const pages = readdirSync(__dirname).filter((f) => f.endsWith('.html')).sort();
const strictPages = new Set(['index.html']);
if (strictRouter) strictPages.add('geometric_prime_router_webapp.html');

let failed = false;
const report = { strictPages: [...strictPages], clean: [], legacyResidual: [], failures: [] };

for (const page of pages) {
    const html = readFileSync(resolve(__dirname, page), 'utf8');
    const offenders = scanPage(page, html);
    if (offenders.length === 0) { report.clean.push(page); continue; }
    if (strictPages.has(page)) { failed = true; report.failures.push({ page, offenders }); }
    else report.legacyResidual.push({ page, note: 'known legacy page: load refs tracked, outside the page-load bar unless the PM rules otherwise', offenders });
}

if (failed) {
    console.error('FAIL: load-time external references in a page under the zero-external-load bar:');
    for (const f of report.failures) for (const o of f.offenders) console.error('  ' + f.page + ' ' + o.tag + '[' + o.attr + '] = ' + o.value);
    process.exit(1);
}
console.log(JSON.stringify({ verdict: 'PAGE-LOAD CHECKS PASSED', ...report }, null, 2));
