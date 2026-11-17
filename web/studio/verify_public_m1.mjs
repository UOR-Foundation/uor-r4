// Usage: node web/studio/verify_public_m1.mjs <staged or live Studio URL>
import assert from 'node:assert/strict';
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
    context.on('request', request => {
        const target = new URL(request.url());
        if (/^https?:$/.test(target.protocol) && target.origin !== url.origin) external.push([request.method(), target.hostname]);
    });
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(url.href);
    await page.getByRole('button', { name: 'Models', exact: true }).click();
    const load = page.getByRole('button', { name: 'Load M1 next-token predictor (seed 1)', exact: true });
    await load.waitFor();
    assert.deepEqual(external, []);
    await load.click();
    const predict = page.getByRole('button', { name: 'Predict next tokens', exact: true });
    await predict.waitFor({ timeout: 180000 });
    const model = await page.locator('html').evaluate(node => ({ ...node.dataset }));
    assert.equal(model.modelSha256, sha);
    assert.equal(model.modelSize, '86536132');
    assert.equal(model.modelSchema, 'uor-r4.lut-discrete-trunk/1');
    assert.equal(model.modelShaVerified, 'true');
    assert.match(model.modelTokenizer, /stack-bound byte-bpe/);
    assert.ok(external.some(([method, host]) => method === 'GET' && host === 'huggingface.co'));
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
