"""Local docs workflow checks. Run with Python, PyYAML, bash and git."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / '.github/workflows'


def workflow(name):
    # BaseLoader preserves GitHub's "on" key and literal shell text.
    return yaml.load((WORKFLOWS / name).read_text(), Loader=yaml.BaseLoader)


def step(job, name):
    return next(entry for entry in job['steps'] if entry.get('name') == name)


class BranchPolicy(unittest.TestCase):
    def test_only_manual_main_publication_gets_a_write_token(self):
        wf = workflow('docs-site.yml')
        build, publish = wf['jobs']['site'], wf['jobs']['publish']
        inputs = wf['on']['workflow_dispatch']['inputs']['target']
        self.assertEqual(wf['on']['push']['branches'], ['main'])
        self.assertEqual(inputs['default'], 'build-only')
        self.assertEqual(inputs['options'], ['build-only', 'gh-pages'])
        self.assertEqual(wf['permissions'], {'contents': 'read'})
        self.assertNotIn('permissions', build)
        self.assertEqual(publish['permissions'], {'contents': 'write'})
        gate = "github.event_name == 'workflow_dispatch' && github.ref == 'refs/heads/main' && inputs.target == 'gh-pages'"
        self.assertEqual(publish['if'], gate)
        self.assertEqual(publish['needs'], 'site')
        upload = step(build, 'Pass the checked book to the manual publisher')
        self.assertEqual(upload['if'], gate)
        self.assertEqual(upload['with']['path'], 'book/')
        self.assertEqual(publish['steps'][0]['with']['ref'], 'refs/heads/gh-pages')

    def test_cleanup_guards_and_registry_agree(self):
        registry = json.loads((ROOT / 'docs/harness/branches.json').read_text())
        protected = [entry['name'] for entry in registry['protected']]
        self.assertEqual(set(protected), {'main', 'gh-pages', 'archive'})
        cleanup = workflow('branch-cleanup.yml')['jobs']
        for job, var in [('merged-head', 'BRANCH'), ('sweep', 'name')]:
            body = cleanup[job]['steps'][0]['run']
            guard = next(line for line in body.splitlines() if f'case "${var}"' in line)
            names = guard.split(' in ', 1)[1].split(')', 1)[0].split('|')
            self.assertEqual(set(names), set(protected))
            self.assertLess(body.index(guard), body.index('--force-with-lease='))

    def test_book_artifact_rejects_links_before_upload(self):
        body = step(workflow('docs-site.yml')['jobs']['site'], 'Check the book artifact boundary')['run']
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            book = root / 'book'
            book.mkdir()
            (book / 'index.html').write_text('book')
            subprocess.run(['bash', '-c', body], cwd=root, check=True)
            (book / 'link').symlink_to(root)
            refused = subprocess.run(['bash', '-c', body], cwd=root, capture_output=True)
            self.assertNotEqual(refused.returncode, 0)

    def test_publish_preserves_studio_and_refuses_collisions(self):
        publish = step(workflow('docs-site.yml')['jobs']['publish'], 'Publish')
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            env = {**os.environ, **publish['env'], 'GITHUB_SHA': 'fixture'}
            def git(*args, cwd=root):
                return subprocess.check_output(['git', *args], cwd=cwd, env=env, text=True).strip()
            remote, site, book = [root / name for name in ('origin.git', 'site', 'book')]
            git('init', '-q', '--bare', str(remote))
            git('init', '-q', '-b', 'gh-pages', str(site))
            (site / 'studio').mkdir()
            (site / 'studio/index.html').write_text('original Studio')
            (site / 'old.html').write_text('old book')
            git('add', '.', cwd=site)
            git('commit', '-qm', 'original Pages tree', cwd=site)
            original = git('rev-parse', 'HEAD', cwd=site)
            git('remote', 'add', 'origin', str(remote), cwd=site)
            for name in ('gh-pages', 'main', 'archive'):
                git('push', '-q', 'origin', f'HEAD:refs/heads/{name}', cwd=site)
            book.mkdir()
            (book / 'index.html').write_text('new documentation')
            def run_publish():
                return subprocess.run(['bash', '-c', publish['run']], cwd=root,
                                      env=env, capture_output=True, text=True)
            for name in ('studio', '.git'):
                with self.subTest(reserved=name):
                    (book / name).mkdir()
                    self.assertNotEqual(run_publish().returncode, 0)
                    self.assertEqual(git('rev-parse', 'gh-pages', cwd=remote), original)
                    (book / name).rmdir()
            published = run_publish()
            self.assertEqual(published.returncode, 0, published.stdout + published.stderr)
            self.assertEqual(git('show', 'gh-pages:index.html', cwd=remote), 'new documentation')
            self.assertEqual(git('rev-parse', 'gh-pages:studio', cwd=remote),
                             git('rev-parse', f'{original}:studio', cwd=remote))
            self.assertNotIn('old.html', git('ls-tree', '--name-only', 'gh-pages', cwd=remote))
            for name in ('main', 'archive'):
                self.assertEqual(git('rev-parse', name, cwd=remote), original)
            self.assertEqual(set(git('for-each-ref', '--format=%(refname:short)', 'refs/heads', cwd=remote).splitlines()),
                             {'main', 'gh-pages', 'archive'})
