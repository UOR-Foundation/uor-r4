# REPORT: share the mounted Studio runtime across web, native runner, and Tauri, kimi-70

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `000f4b457ee020013654ffbc65f9da5061c2317a`, tree `fed1361391027ca2ef8f9d3bd2041451014eef70`. Implementation/package evidence is committed through `377b76e63`; the final delivery HEAD/tree and merge base belong to the exact-head identity logs rather than this self-referential report.

## What did not work

- The default source UI and production `/studio/` have not been cut over or published. This is an unsigned development-app and local-native-runner unit, not a signed release, production deployment, full Phase A acceptance, useful-chat, M7/M8, accessibility-certification, or Phase B result.
- One packaged Changes run displayed `HEAD (timeout)` and `diff timed out` while returning the complete small two-file diff. The diagnostic is preserved, not hidden or described as a clean Git timing result. No Rust process-control change was made in this unit.
- Native-runner browser Git/terminal actions remain explicitly unavailable. Tauri folder grants and terminal enablement reset per app process; every command still needs separate exact-command approval.
- No real browser profile, credential, external test endpoint, cross-tab/system-quota condition, model training, GPU run, new dependency, or production publisher was used. Synthetic exact arithmetic is a tool response, not useful model generation.

## What changed

- The native runner downloaded and served the pinned legacy UI; Tauri staged that legacy page and loaded `index.html`. There was no shared dist allowlist or local native-runner path.
- `scripts/studio/catalog.mjs` now owns catalog generation for both the existing publisher and `web/studio/src/stage-dist.mjs`. The synthetic dist catalog is byte-identical to the publisher output and pinned hash `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`.
- `runtime-files.sh --typed` stages hashed Vite assets, the early migration runtime, four generated pkg artifacts, and the catalog while rejecting source/test/docs/maps and unexpected outputs. Legacy mode remains available.
- `native-runner.sh --local` builds the existing WASM/pkg artifacts and local `uor-chat`, then serves the same dist at `/?typed=1` without a release download. Existing pinned-release behavior is unchanged. Exact-head acceptance found that running the desktop verifier first left unhashed mixed-workspace artifacts in the shared target, producing duplicate tokenizer/serde types even after package cleaning. A fresh isolated target built successfully; the local runner now uses `${CARGO_TARGET_DIR:-target}/native-runner` and locked resolution, and its runtime contract rejects the old shared-target path. The real desktop-before-native workflow passes after the repair.
- Tauri loads `WebviewUrl::App("index.html?typed=1")`. Preparation extracts only `models.json` from the old archive after an actual package check showed that extracting its `studio/pkg` overwrote the current typed WASM/JS. `src-tauri/src/main.rs` and `tauri.conf.json` are unchanged.
- Package verification compares staged resources with `web/studio/dist`, requires the migration runtime, retains executable/sidecar/resource/hash/icon/development-file boundaries, and passes only with the expected executable SHA supplied by the caller.
- Actual packaged Changes/Terminal acceptance found that `StudioApp` constructed a second desktop adapter without the declared Tauri `invoke`, falsely reporting browser-only capability. A red-first regression reproduced `available: false`; `createDesktopActions()` now defaults to the existing declared host, and the focused test plus complete suite pass. Host-owned approval behavior is unchanged.

## Measurements

- Native-runner CDP: one mounted typed shell; 13 initial requests, zero external/raw-TypeScript/runtime errors; two distinct sessions with actual UOR exact arithmetic `42`, independent transcripts, completed generation, and restored original transcript. Migration success used 21 requests; malformed/conflict/quota used 10 each; all had zero external requests and runtime errors. Old-source reads, marked reload, deleted-destination non-resurrection, and provider isolation passed.
- Packaged window: `tauri://localhost/index.html?typed=1`, real Preact Agent shell, bundled model readiness, and a fresh `17+25` exact `42` response with its geometric trace and completed session state.
- Closing and reopening the same packaged app restored that newly created session and exact transcript. The attachment action retained `pasted-context.txt` with a working remove action.
- The macOS picker granted only the task-owned `acceptance-workspace`. The app created `untitled.md`, transitioned dirty to saved, and the on-disk bytes read back as `Packaged desktop edit/save acceptance: saved by native host.` Older same-named tabs were not edited.
- Changes returned the actual `note.txt` and `untitled.md` diff, with the timeout flags disclosed above. Terminal required and received separate app-session enablement and exact `pwd` approval, then returned the granted workspace path. All app processes started for these checks were stopped by exact task/PID ownership.
- Current package: unsigned `UOR Harness.app`, 21.00 MiB; repaired executable SHA-256 `093ad2957e9b0bdbd4e5358007c27c2ea81edc05e55ef3478de607f0851b8548`. Package contract passed with that expected SHA supplied. Earlier executable evidence is superseded for the final head.

## Checks run at the head

- **435/435** Vitest across 49 files with zero unhandled errors; **113/113** plain-JS Node tests; typecheck and Vite build pass.
- Shared catalog comparison, typed runtime **4/4**, Tauri runtime **7/7**, typed allowlist, preparation, and packaged-resource contract pass.
- Desktop default Rust **7/19/7/13/8** and GUI Rust **9/23/8/13/9** suites pass. One earlier intermittent MCP transport test failure was followed by a focused pass and a complete GUI rerun pass; the final exact-head suites pass without that failure.
- All four legacy checks, native-runner CDP, the desktop-before-native build/workflow, and the rebuilt package contract pass at the reconciled acceptance head.

Evidence is under `local/kimi/studio-shared-shell/`, including the exact-head identity/product/legacy/native/Rust/package logs and `packaged-evidence/` screenshots and disk readback. The harness build passed; report/local-PR/full-gate results are not claimed at this edit. Fresh delivery review, PR/GitHub checks, protected queue/merge, and fresh-main verification remain **NOT_RUN**.

## Skills used

Shared-runtime planning and red-first implementation, systematic packaged-window diagnosis, test-driven adapter repair, macOS accessibility verification, and verification before completion. No implementation subagent or external model compute was used. A fresh exact-head `uor-delivery-reviewer` remains required before protected delivery.

## Still open

Complete acceptance at the final committed head, fresh review, protected merge, and fresh-main verification remain for this unit. Final browser/desktop acceptance must diagnose the Git timeout result and cover the complete Phase A workflows before production publication and packaged-app launch. Production `/studio/`, full Phase A, and Phase B remain incomplete.

## Plain summary

Web preview, the local native runner, and the unsigned macOS app now use the same mounted Vite runtime and catalog. Real native and packaged checks prove exact arithmetic, session restore, attachment, native save, Git output, and approved command execution; the adapter defect found by those checks is repaired. The Git timeout diagnostic, exact-head delivery, final acceptance, and production publication remain open.
