import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
RUNTIME = REPO / 'scripts/studio/runtime-files.sh'
PREPARE = REPO / 'scripts/studio/prepare-tauri-runtime.sh'
CONF = REPO / 'src-tauri/tauri.conf.json'


class TauriRuntimeContractTests(unittest.TestCase):
    def test_allowlist_has_no_development_files(self):
        text = RUNTIME.read_text(encoding='utf-8')
        result = subprocess.run(['bash', str(RUNTIME), '--list'], text=True, capture_output=True, check=True)
        for required in ['index.html', 'index.css', 'models.json', 'assets/js/studio_ui.js', 'pkg/uor_r4_integer_bg.wasm']:
            self.assertIn(required, result.stdout)
        for forbidden in ['src/main.ts', 'package.json', 'tsconfig.json', 'vite.config.ts', 'model_sources.test.mjs', 'pkg/README.md', 'node_modules']:
            self.assertNotIn(forbidden, result.stdout)
        self.assertNotIn('https://', result.stdout)

    def test_copy_is_runtime_only(self):
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / 'studio'
            subprocess.run(['bash', str(RUNTIME), 'copy', str(REPO / 'web/studio'), str(dest), '--allow-generated-missing'], check=True)
            for required in ['index.html', 'index.css', 'assets/js/studio_ui.js', 'assets/js/studio_api.js']:
                self.assertTrue((dest / required).is_file(), required)
            for forbidden in ['src/main.ts', 'package.json', 'tsconfig.json', 'vite.config.ts', 'assets/js/model_sources.test.mjs', 'pkg/README.md', 'node_modules']:
                self.assertFalse((dest / forbidden).exists(), forbidden)

    def test_packaging_contract(self):
        conf = json.loads(CONF.read_text(encoding='utf-8'))
        self.assertEqual(conf['build']['frontendDist'], './runtime/studio')
        self.assertEqual(conf['bundle']['externalBin'], ['binaries/uor-chat'])
        self.assertTrue(conf['bundle']['active'])
        csp = conf['app']['security']['csp']
        self.assertIn("'wasm-unsafe-eval'", csp)
        self.assertIn("worker-src 'self'", csp)
        self.assertNotIn('https://', csp)
        self.assertIn('runtime/studio/pkg/', conf['bundle']['resources'])
        self.assertTrue((REPO / 'src-tauri/assets/uor-icon.svg').is_file())

    def test_prepare_verifies_pinned_runtime(self):
        with tempfile.TemporaryDirectory() as tmp:
            env = os.environ.copy()
            env['UOR_TAURI_RUNTIME_HOME'] = tmp
            result = subprocess.run(['bash', str(PREPARE)], env=env, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((Path(tmp) / 'binaries/uor-chat-aarch64-apple-darwin').is_file())
            self.assertTrue((Path(tmp) / 'runtime/studio/pkg/synthetic-stack.lut').is_file())
            self.assertTrue((Path(tmp) / 'runtime/studio/pkg/tokenizer.json').is_file())
            self.assertTrue((Path(tmp) / 'runtime/studio/models.json').is_file())
            self.assertTrue((Path(tmp) / 'icons/icon.png').is_file())
            self.assertTrue((Path(tmp) / 'icons/icon.icns').is_file())

    def test_prepare_check_is_side_effect_free(self):
        with tempfile.TemporaryDirectory() as tmp:
            env = os.environ.copy()
            env['UOR_TAURI_RUNTIME_HOME'] = str(Path(tmp) / 'not-created')
            result = subprocess.run(['bash', str(PREPARE), '--check'], env=env, text=True, capture_output=True, check=True)
            self.assertIn('uor-chat-aarch64-apple-darwin', result.stdout)
            self.assertIn('externalBin: ["binaries/uor-chat"]', result.stdout)
            self.assertIn('f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e', result.stdout)
            self.assertFalse(Path(env['UOR_TAURI_RUNTIME_HOME']).exists())


if __name__ == '__main__':
    unittest.main()
