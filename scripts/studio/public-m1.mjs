import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const parityPath = 'docs/labs/one-model-end-to-end/evidence-deepseek-11-trunk-parity.json';
const devPath = 'docs/labs/m1-close-saved-model/evidence/t16-frozen-zero-seed1-trunk-dev-score.json';
const revision = '1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55';
const release = 'https://huggingface.co/caseyallard/uor-r4-m1-model/resolve/' + revision;

// The historical M1 discrete predictor has its own measured CPU/integer and CUDA
// evidence. Only this exact published pair can use this profile.
function readEvidence(path, digest) {
  const bytes = readFileSync(new URL('../../' + path, import.meta.url));
  if (createHash('sha256').update(bytes).digest('hex') !== digest) throw new Error('M1 evidence digest differs: ' + path);
  return JSON.parse(bytes);
}

export function publicM1Card(selector) {
  if (selector.public_release !== 'm1-discrete-trunk-seed-1' || Object.keys(selector).length !== 1) {
    throw new Error('Unknown public release or attempt to override its pins');
  }
  const paritySha = 'd93d61bf0ee25825812522f49f7d82552d95c4f9601f54f0baa4cf49a843c12c';
  const devSha = '685226d637bc541237b230a89b370f24bf02129c0bef57fed33aa77c8fe88add';
  const parity = readEvidence(parityPath, paritySha);
  const dev = readEvidence(devPath, devSha);
  const cpu = parity.cpu_engine_parity;
  if (dev.artifact_sha256 !== parity.artifact.sha256 || dev.reference.agreed !== 14287 ||
      dev.reference.checked !== 14287 || !dev.planted_fault_b4.b4_pass ||
      parity.artifact.tables !== 19 || cpu.rows !== 616 || cpu.query_coordinates_checked !== 19712 ||
      cpu.query_coordinate_mismatches !== 0 || cpu.admitted_cell_mismatches !== 0 ||
      cpu.top8_mismatches !== 0 || cpu.token_mismatches !== 0) throw new Error('M1 release evidence mismatch');
  return {
    id: 'm1-discrete-trunk-seed-1', kind: 'published', primary: true,
    label: 'M1 next-token predictor (seed 1)',
    url: release + '/t16-frozen-zero-seed1-trunk.d11', tokenizer_url: release + '/tokenizer-seed-1.json',
    format: 'UORLUT01', schema: 'uor-r4.lut-discrete-trunk/1', size_bytes: 86536132,
    sha256: parity.artifact.sha256, tokenizer_sha256: parity.tokenizer.sha256,
    discrete_parity_sha256: paritySha, gpu_dev_score_sha256: devSha,
    note: 'Downloads 86.5 MB of public model weights and a matching tokenizer on request. Ranks eight next tokens locally.',
  };
}
