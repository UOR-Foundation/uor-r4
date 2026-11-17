// Usage: node web/studio/verify_public_m1.mjs <staged or live Studio URL>
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { chromium } from 'playwright';

const url = new URL(process.argv[2]);
const sha = '04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d';
const prompt = 'The capital of France is';
// IntegerDiscreteModel::evaluate_prefix on bound BPE IDs 413,2743,3695,372,3704,1651,435.
const expected = [223, 952, 2359, 2471, 23, 263, 964, 1520];
const browser = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chrome' });
try {
    const context = await browser.newContext();
    const page = await context.newPage();
    const external = [];
    const errors = [];
    const releaseBase = 'https://huggingface.co/caseyallard/uor-r4-m1-model/resolve/1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55/';
    const permittedRoots = [releaseBase + 't16-frozen-zero-seed1-trunk.d11', releaseBase + 'tokenizer-seed-1.json'];
    context.on('request', request => {
        const target = new URL(request.url());
        if (!/^https?:$/.test(target.protocol) || target.origin === url.origin) return;
        let root = request;
        while (root.redirectedFrom()) root = root.redirectedFrom();
        external.push({ method: request.method(), origin: target.origin, root: root.url() });
    });
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(url.href);
    await page.getByRole('button', { name: 'Models', exact: true }).click();
    const load = page.getByRole('button', { name: 'Load M1 next-token predictor (seed 1)', exact: true });
    await load.waitFor();
    assert.deepEqual(external, []);
    const catalog = await (await context.request.get(new URL('models.json', url).href)).json();
    const card = catalog.models.find(entry => entry.id === 'm1-discrete-trunk-seed-1');
    assert.equal(card.served_forward_audit_schema, 'uor-r4.m1-discrete-served-forward/1');
    const receiptUrl = new URL(card.audit_url, url);
    assert.equal(receiptUrl.origin, url.origin);
    const receipt = await context.request.get(receiptUrl.href);
    assert.equal(receipt.status(), 200);
    const receiptSha = createHash('sha256').update(await receipt.body()).digest('hex');
    assert.equal(receiptSha, 'ff370aefd6ba8062e2316c5478a8d428b8e220f8c5da2d47f7b0eeed24bfe479');
    assert.equal(card.served_forward_audit_sha256, receiptSha);
    await load.click();
    const predict = page.getByRole('button', { name: 'Predict next tokens', exact: true });
    await predict.waitFor({ timeout: 180000 });
    const model = await page.locator('html').evaluate(node => ({ ...node.dataset }));
    assert.equal(model.modelSha256, sha);
    assert.equal(model.modelSize, '86536132');
    assert.equal(model.modelSchema, 'uor-r4.lut-discrete-trunk/1');
    assert.equal(model.modelShaVerified, 'true');
    assert.match(model.modelTokenizer, /stack-bound byte-bpe/);
    assert.deepEqual([...new Set(external.map(request => request.root))].sort(), [...permittedRoots].sort());
    for (const request of external) {
        assert.equal(request.method, 'GET');
        assert.ok(request.origin === 'https://huggingface.co' || /^https:\/\/(?:[a-z0-9-]+\.)+cdn\.hf\.co$/.test(request.origin), 'Unexpected download origin: ' + request.origin);
    }
    const downloads = external.length;
    await context.setOffline(true);
    await page.getByRole('textbox', { name: 'Prompt', exact: true }).fill(prompt);
    await predict.click();
    const result = page.getByRole('region', { name: 'Next-token predictions' });
    await result.waitFor();
    const tokens = await result.getByRole('listitem').allTextContents();
    assert.deepEqual(tokens.map(text => Number(/^ID (\d+)/.exec(text)?.[1])), expected);
    assert.equal(await page.locator('html').getAttribute('data-predictor-prompt-length'), '7');
    assert.equal(external.length, downloads);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ prompt, artifact_sha256: sha, tokenizer: model.modelTokenizer, tokens, external_before_load: 0, download_requests: external, offline_prediction: true }));
} finally {
    await browser.close();
}
