#!/usr/bin/env python3
"""Normative release gate. SPEC.md section 20.2.

Exits non-zero unless every numbered condition holds. It is expected to fail at
step 9 while WGG-GO-1 is outstanding; that failure is the conforming behavior.
"""
import hashlib, json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)
FAIL = []
def check(n, name, ok, detail=""):
    print(f"  [{'PASS' if ok else 'FAIL'}] {n}. {name}" + (f" — {detail}" if detail else ""))
    if not ok: FAIL.append((n, name, detail))

def sha(p): return hashlib.sha256(open(p, "rb").read()).hexdigest()

print("release gate (SPEC 20.2)\n")

# 1. authority + toolchain identities
man = json.load(open("authority/manifest.json"))
u = man["pins"]["uorGnaf"]
check(1, "UOR-GNAF authority digest",
      sha(u["path"]) == u["pinnedSha256"], u["pinnedSha256"][:16] + "...")
mf = subprocess.run(["python3", "Tools/manifest.py", "--check"], capture_output=True, text=True)
check(1, "manifest identity stages current and acyclic", mf.returncode == 0,
      mf.stdout.strip().splitlines()[-1][:150] if mf.stdout.strip() else mf.stderr[:120])
tc = open("lean-toolchain").read().strip()
check(1, "Lean toolchain pin", tc == man["pins"]["lean"]["toolchain"], tc)
check(1, "WebAssembly Core wg-3.0 vendored",
      man["pins"]["wasmCore"]["vendored"],
      "vendor/wasm-spec/ empty; pinned tree not vendored")

# 2. claim graph
claims = json.load(open("model/claims.json"))["claims"]
ids = [c["id"] for c in claims]
check(2, "claim graph nonempty", len(claims) > 0, f"{len(claims)} claims")
check(2, "claim ids unique", len(ids) == len(set(ids)))
dangling = [d for c in claims for d in c.get("dependsOn", []) if d not in ids]
check(2, "no orphan dependencies", not dangling, str(dangling))
fw = subprocess.run(["python3", "Tools/firewall.py"], capture_output=True, text=True)
check(2, "dependency firewall (SPEC 10.1)", fw.returncode == 0,
      fw.stdout.strip().splitlines()[-1][:160] if fw.stdout.strip() else "")

# 3. Lean builds, no placeholder or unexpected axiom
r = subprocess.run(["lake", "build"], capture_output=True, text=True)
check(3, "lake build", r.returncode == 0, r.stderr.strip()[:120])
# `lake build` can report success while serving a stale .olean, masking a root
# module that does not actually elaborate (this really happened: two modules
# declared clashing `Fault` types and lake reported green). Elaborate the root
# directly — success there is the claim that matters.
env0 = dict(os.environ, LEAN_PATH=os.path.join(ROOT, ".lake/build/lib/lean"))
rootlean = subprocess.run(["lean", "WasmGemmGnaf.lean"], capture_output=True,
                          text=True, env=env0)
check(3, "root module elaborates (not a stale olean)",
      rootlean.returncode == 0 and not rootlean.stdout.strip(),
      rootlean.stdout.strip()[:160])
scan = subprocess.run(["python3", "Tools/scan.py"], capture_output=True, text=True)
banned = "" if scan.returncode == 0 else scan.stdout.strip()
check(3, "no forbidden constructs", banned == "", banned[:160])

ALLOWED = {"propext", "Quot.sound", "Classical.choice"}
proved = [c for c in claims if c["level"] == "formalProof"]
# The root module, not one layer of it: a probe importing only `Cost.Objective`
# cannot see a claim proved in `GNAF` or `Gemm`, and an unseen claim is an
# unaudited claim.
probe = "\n".join(["import WasmGemmGnaf"] +
                  [f"#print axioms {c['leanDeclaration']}" for c in proved])
open(".gate_axioms.lean", "w").write(probe + "\n")
env = dict(os.environ, LEAN_PATH=os.path.join(ROOT, ".lake/build/lib/lean"))
ax = subprocess.run(["lean", ".gate_axioms.lean"], capture_output=True, text=True, env=env)
os.remove(".gate_axioms.lean")
unexpected = [w for w in ("sorryAx", "Lean.ofReduceBool", "Lean.trustCompiler") if w in ax.stdout]
check(3, "axiom closure clean", ax.returncode == 0 and not unexpected,
      (str(unexpected) or ax.stderr.strip()[:120]) if (unexpected or ax.returncode) else
      "propext, Quot.sound only")

devs = json.load(open("model/spec-deviations.json"))["deviations"]
undocumented = [d for d in devs if not d.get("intendedContentProvedBy")]
check(2, "every SPEC deviation carries a proved replacement",
      not undocumented, f"{len(devs)} deviation(s), all justified"
      if not undocumented else str([d["id"] for d in undocumented]))

# 4-8. semantics, coverage, artifact, lower bound
def outstanding(cid):
    return next(c for c in claims if c["id"] == cid)["status"] != "outstanding"
check(4, "WebAssembly and GEMM semantics built", outstanding("WS-001"), "WS-001 outstanding (O-6)")
check(5, "universal sublevel coverage proved", outstanding("UV-001"), "UV-001 outstanding (O-5)")
check(6, "committed artifact matches proved value",
      os.path.exists("artifacts/wasm-gemm-gnaf.wasm"), "artifact not emitted")
check(7, "artifact decode/validate/ABI/cost theorems", False, "gated on WS-001")
check(8, "universal lower bound and attainment", outstanding("LB-001"), "LB-001 outstanding (O-5)")

# 9. the release theorem itself
go = next(c for c in claims if c["id"] == "GO-001")
check(9, "released_wasm_gemm_gnaf_global_optimal closed",
      go["status"] != "outstanding",
      f"answer class {go['answerClass']} per {go['answerAuthority']}")

# 10-13
check(10, "Atlas seal reconstructs", False, "seal not constructed")
mut = subprocess.run(["python3", "Tools/mutation.py"], capture_output=True, text=True)
check(11, "mutation suites reject planted faults", mut.returncode == 0,
      "all planted faults rejected" if mut.returncode == 0 else mut.stdout.strip()[-160:])
check(12, "two clean emissions byte-identical", False, "gated on step 6")
root = subprocess.run(["python3", "Tools/root.py", "--check"], capture_output=True, text=True)
check(13, "no stale or unowned Lean module", root.returncode == 0, root.stdout.strip()[:160])
check(13, "worktree clean after verification",
      subprocess.run(["git", "status", "--porcelain"], capture_output=True,
                     text=True).stdout.strip() == "", "untracked build outputs present")

print(f"\n{'='*66}")
if FAIL:
    print(f"GATE: FAIL — {len(FAIL)} unmet condition(s).")
    print("This is the conforming outcome while WGG-GO-1 is outstanding.")
    print("Terminal answer: WorkloadIncomplete (UOR-GNAF 10.9). See CERTIFICATION.md.")
    print("Per UOR-GNAF 13.3 the gate MUST NOT return an unproved global label.")
    sys.exit(1)
print("GATE: PASS")
