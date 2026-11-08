import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { writeStudioCatalog } from '../../../../scripts/studio/catalog.mjs';

function fixtureRoot() {
    const root = mkdtempSync(join(tmpdir(), 'uor-studio-catalog-'));
    const repo = join(import.meta.dirname, '..', '..', '..', '..');
    execFileSync('bash', ['scripts/studio/runtime-files.sh', 'copy', join(repo, 'web/studio'), root, '--allow-generated-missing'], { cwd: repo });
    mkdirSync(join(root, 'pkg'), { recursive: true });
    for (const name of ['uor_r4_integer.js', 'uor_r4_integer_bg.wasm', 'synthetic-stack.lut', 'tokenizer.json']) {
        writeFileSync(join(root, 'pkg', name), name);
    }
    return root;
}

test('shared Studio catalog matches the existing publisher byte-for-byte', () => {
    const root = fixtureRoot();
    const dist = join(root, 'dist');
    mkdirSync(dist, { recursive: true });
    cpSync(join(root, 'pkg'), join(dist, 'pkg'), { recursive: true });
    writeStudioCatalog({ dest: dist, publicUrl: '', publicLabel: 'Public model', trainedManifestPath: '', receiptGate: '' });

    const published = join(root, 'published');
    execFileSync('bash', ['scripts/stage-studio-site.sh', root, published], { cwd: join(import.meta.dirname, '..', '..', '..', '..'), stdio: 'pipe' });
    assert.equal(readFileSync(join(dist, 'models.json'), 'utf8'), readFileSync(join(published, 'models.json'), 'utf8'));
});
