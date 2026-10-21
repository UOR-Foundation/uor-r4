# ORDER: infra-loop-pr09-client-adapters, antigravity-2 (repair: guard tokenization, subagent binding, install staging)

Lab: antigravity. Item: [infra-loop-pr09-client-adapters](../../../../roadmap/items/infra-loop-pr09-client-adapters.md) (#2914). Setup row: 9. Base: `e7b768e04`. Repair of antigravity-1 (PR #2932), after the counted `CHANGES` record at `e7b768e043349bc7456ebfb4ab1082949d85a963` (`claude`, `uor-review v1`, fresh context).

## Done in antigravity-1 (history)

- Claimed `#2914` and committed `docs/loop/tasks/infra-loop-harness/antigravity-1/ORDER.md` first (`ed10c86f7`).
- Built `crates/uor-r4-harness/src/loopguard.rs` (`loop guard`, `loop install`, `loop onboard`, `loop copyright`), `tests/loop_guard.rs`, and `docs/harness/onboarding/{antigravity,claude,codex,deepseek,kimi}.json` (`774db9003`).
- Opened PR `#2932` and requested the D59 review (`e7b768e04`).

## Goal

Address every HIGH, MEDIUM and LOW finding in the `e7b768e04` `CHANGES` review record:

1. **HIGH 1 — Tokenize shell commands in `loop guard`:**
   - Split shell commands into segments (across `;`, `|`, `||`, `&&`, `&`, newlines, `(...)`, `$(...)`, backticks), unwrap leading `KEY=VALUE` assignments and command wrappers (`env`, `command`, `nohup`, `time`, `nice`, `sudo`, `sh -c`, `bash -c`, `zsh -c`, `xargs`), and strip any directory prefix from `argv[0]` so `/usr/bin/pkill foo` and `/bin/killall` are refused.
   - Skip all `git` global options (`-C <dir>`, `-c <key=val>`, `--git-dir`, `--work-tree`, `--namespace`, `--bare`, `--no-pager`, `-p`, `--config-env`, etc.) before matching the `git` verb and its arguments, while treating any `-c core.hooksPath=...` or `--config-env=core.hooksPath=...` (case-insensitive) as a forbidden hook bypass.
   - On `git commit`, refuse `-n` in short flag clusters (such as `-nm x`, `-an`, `-anm`) as well as `--no-verify`.
   - On `git push`, skip options (such as `-u`, `--set-upstream`, `--repo`) and treat any refspec whose destination is `main` or `refs/heads/main` (including `git push origin main`, `git push -u origin main`, `git push origin feat:main`, `git push origin HEAD:refs/heads/main`, `git push origin :main`) as a forbidden push to `main`.
   - Add negative tests for every case (`git -c core.hooksPath=/dev/null commit -m x`, `git push origin feat:main`, `/usr/bin/pkill foo`, `git -c a=b push --force`, `git push -u origin main`, `git commit -nm x`).
2. **HIGH 2 — `loop install` staging in `local/` and explicit documentation of every user-level path:**
   - By default (when `--home` is not passed), `loop install (--client <C> | --all)` writes **only** inside gitignored `local/loop/install/<client>/` (and `local/loop/install-backups/` and `local/deepseek/headless-overlay.yml`), staging the normalized skills and adapter files without touching `~/`.
   - When `--home <dir>` is passed, refuse with exit `2` if `<dir>` resolves to the user's live `$HOME` while `UOR_LAB` is set (since modifying global client configs in `$HOME` is an owner-only action).
   - Document in `docs/harness/README.md`, `docs/loop/README.md`, this ORDER and the REPORT every file path that `loop install` stages under `local/loop/install/<client>/` or writes under `--home <dir>`:
     - `claude`: `~/.claude/settings.json`, `~/.claude/CLAUDE.md`, `~/.claude/skills/<6 skills>/SKILL.md`
     - `codex`: `~/.codex/hooks.json`, `~/.codex/rules/uor-r4.rules`, `~/.codex/skills/<6 skills>/SKILL.md`
     - `kimi`: `~/.kimi-code/config.toml`, `~/.kimi-code/UOR-R4-START.md`, `~/.kimi-code/UOR-R4-HANDOFF.md`, `~/.agents/skills/<6 skills>/SKILL.md`
     - `antigravity`: `~/.gemini/config/plugins/uor-r4-loop/plugin.json`, `~/.gemini/config/plugins/uor-r4-loop/hooks.json`, `~/.gemini/config/skills/<6 skills>/SKILL.md`
     - `deepseek`: `~/.dsh/hooks.json`, `~/.dsh/profiles/headless/cordis.patch.yml`, `~/.dsh/skills/<6 skills>/SKILL.md`, and `local/deepseek/headless-overlay.yml`
3. **MEDIUM 1 — Tokenized `gh` merge checks and comprehensive credential-read / env-dump refusal:**
   - Match tokenized `gh` commands so `gh  pr merge` (extra whitespace or global flags) without `--match-head-commit <40-hex>` and `gh api .../merge` are refused.
   - Refuse any command referencing credential paths (`~/.netrc`, `~/.git-credentials`, `.env*`, `auth.json`, `credentials.json`, `~/.aws/credentials`, `~/.runpod/config.toml`, `~/.vast_api_key`, `~/.config/gh/hosts.yml`, `~/.npmrc`, `~/.pypirc`, `~/.cargo/credentials*`, `~/.ssh/id_*`) regardless of the reader (`cat`, `head`, `tail`, `less`, `more`, `grep`, `rg`, `awk`, `sed`, etc.), and refuse environment-variable dumps (`printenv`, bare `env`, `export -p`, `declare -p`, secret `$...KEY...`/`$...TOKEN...` expansions).
4. **MEDIUM 2 — Active ORDER from claimed branch or active-order marker only:**
   - `find_active_order` reads the active ORDER only from `local/<lab>/guard/active-order` or from the ORDER added on the worktree's claimed `<lab>/<topic>` branch relative to `origin/main` (never from closed historical task folders such as `claude-20`). With neither present, top-level writes inside the lab's worktree are not held to a closed task's scope.
5. **MEDIUM 3 — Bind Claude Code `agent_id` to numbered suborder `<n>.md`:**
   - Bind `agent_id` to the next unclaimed `local/<lab>/<topic>/orders/<n>.md` at `SubagentStart` (stored atomically in `local/<lab>/guard/subagent-bindings.json`, released at `SubagentStop`), and refuse subagent writes when `agent_id` has no binding.
6. **MEDIUM 4 — Antigravity `SessionStart` / fail-closed state and atomic Kimi depth counter:**
   - In Antigravity, replace stale `root_conversation_id` on `SessionStart` unless the root session has a pending `invoke_subagent` spawn (or `isSubagent` is set), and fail closed (treat as a subagent) when `PreToolUse` fires with no prior `SessionStart`.
   - Protect all guard state updates (`antigravity`, `kimi` depth, `claude` bindings) with an advisory lockfile and atomic temp-file rename.
7. **MEDIUM 5 — Shell file-modifying commands checked against write rules:**
   - Check target paths of `cp`, `mv`, `rm`, `unlink`, `rmdir`, `install`, `ln`, `touch`, `truncate`, `chmod`, `chown`, `sed -i`, `perl -i`, `ruby -i`, `dd of=...`, `git rm`, `git mv` and `git restore` (in addition to `>`, `>>` and `tee`) through `check_write_path`.
8. **LOW 1, LOW 2, LOW 3:**
   - Allow `git push --force-with-lease` while refusing `--force`, `--force-if-includes`, `-f` and `+refspec`.
   - Check repository membership (`find_uor_roots`) before validating `--client` or failing on malformed JSON in `loop guard`, so hooks outside `~/uor-r4` always exit `0`.
   - Record `N/A` in `docs/harness/onboarding/*.json` for hooks and skills that have not yet been installed via `loop install`, and state explicitly in the receipts and `loop onboard` output that the canary is evaluated in-process on `loop guard` rather than through the client's live hook runner.

## Cancel probe (run first)

Is the counted `CHANGES` record at `e7b768e043349bc7456ebfb4ab1082949d85a963` still the last counted record on PR `#2932`, and do `git -c core.hooksPath=/dev/null commit -m x`, `git push origin feat:main` and `/usr/bin/pkill foo` still exit `0` on the `e7b768e04` binary? Result: yes — `uor-harness loop review --check --pr 2932` exits `1` (`fail: the last counted record is CHANGES`), and all three bypass probes reproduce at `e7b768e04`. The repair proceeds. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs`
- `crates/uor-r4-harness/tests/loop_guard.rs`
- `docs/harness/onboarding/antigravity.json`
- `docs/harness/onboarding/claude.json`
- `docs/harness/onboarding/codex.json`
- `docs/harness/onboarding/deepseek.json`
- `docs/harness/onboarding/kimi.json`
- `docs/harness/README.md`
- `docs/loop/README.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-1/REPORT.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-2/**`

## Done when

- Every HIGH, MEDIUM and LOW point from the `e7b768e04` `CHANGES` record is fixed and covered by unit/integration tests in `crates/uor-r4-harness/tests/loop_guard.rs`.
- Probes on the built binary refuse `--no-verify`, `pkill`, `git -c core.hooksPath=/dev/null commit -m x`, `git push origin feat:main`, `/usr/bin/pkill foo`, `git -c a=b push --force`, `git push -u origin main`, `git commit -nm x`, `gh  pr merge`, `gh api .../merge`, `head ~/.netrc`, `less ~/.netrc`, `grep x ~/.netrc`, `printenv`, `~/.aws/credentials`, `~/.runpod/config.toml`, `cp x CHARTER.md`, `sed -i 's/a/b/' CHARTER.md`, `mv x CHARTER.md` and `rm CHARTER.md` with exit `2`, while allowing `git push --force-with-lease origin feat` and exiting `0` outside the repo even with unknown `--client` or malformed JSON.
- `uor-harness loop gates --full`, `uor-harness loop onboard --check-receipts`, `uor-harness loop copyright` and `uor-harness loop report --check` pass.

## Stop and ask if

- Casey's decision on running `loop install` against live `$HOME` changes the command's required flags.
- Any file outside the scope above must change.
