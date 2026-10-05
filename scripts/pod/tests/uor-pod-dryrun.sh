#!/usr/bin/env bash
# Dry-run test of scripts/pod/uor-pod's up/down/lease/renew/release/run/reap
# logic against a fake runpodctl, ssh and gh on PATH (no network, no cost):
#
#   scripts/pod/tests/uor-pod-dryrun.sh
#
# The fake account holds pod A (2 x 5090, leased by the test), pod B (1 x 4090,
# idle, unleased, reaper idle for 30 min), pod C (1 x 4090, busy, unleased) and
# pod D (stopped); C sits on a non-canonical volume. Stock: 5090 none in EUR-NO-1
# but Low in EU-RO-1 (which has no uor-shared volume yet); 4090 Low in EUR-NO-1.
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
TOOL=$HERE/../uor-pod
W=$(mktemp -d "${TMPDIR:-/tmp}/uor-pod-test.XXXXXX")
trap 'rm -rf "$W"' EXIT
mkdir -p "$W/bin" "$W/fake"
export UOR_POD_STATE=$W/state UOR_POD_DRY_RUN=1 FAKE=$W/fake PATH="$W/bin:$PATH"
NOW=$(date -u +%s)

cat > "$FAKE/pods.json" <<'J'
[
 {"id":"poda","name":"a","desiredStatus":"RUNNING","gpuCount":2,"costPerHr":1.98,"uptimeSeconds":3600,"env":{"SECRET":"must-not-print"}},
 {"id":"podb","name":"b","desiredStatus":"RUNNING","gpuCount":1,"costPerHr":0.74,"uptimeSeconds":7200},
 {"id":"podc","name":"c","desiredStatus":"RUNNING","gpuCount":1,"costPerHr":0.74,"uptimeSeconds":7200,"networkVolumeId":"volro"},
 {"id":"podd","name":"d","desiredStatus":"EXITED","gpuCount":2,"costPerHr":1.48,"uptimeSeconds":0}
]
J
cat > "$FAKE/gpus.json" <<'J'
[
 {"gpuId":"NVIDIA GeForce RTX 5090","securePricePerHr":0.99,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"none"},{"dataCenterId":"EU-RO-1","stockStatus":"Low"}]},
 {"gpuId":"NVIDIA GeForce RTX 4090","securePricePerHr":0.74,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"Low"}]},
 {"gpuId":"NVIDIA A100 80GB PCIe","securePricePerHr":1.59,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"High"}]}
]
J
# ssh ports: poda 1001, podb 1002, podc 1003
printf 'gpu=0, NVIDIA GeForce RTX 5090, 0, 1\ngpu=1, NVIDIA GeForce RTX 5090, 0, 1\nreaper=1\nprobe_ok=1\n' > "$FAKE/probe-1001"
printf 'gpu=0, NVIDIA GeForce RTX 4090, 0, 1\nidle_since=%s\nreaper=1\nprobe_ok=1\n' $((NOW - 1800)) > "$FAKE/probe-1002"
printf 'gpu=0, NVIDIA GeForce RTX 4090, 97, 20000\napp=4242\nreaper=0\nprobe_ok=1\n' > "$FAKE/probe-1003"

cat > "$W/bin/runpodctl" <<'S'
#!/usr/bin/env bash
echo "runpodctl $*" >> "$FAKE/calls"
case "$1 $2" in
  "pod list") cat "$FAKE/pods.json";;
  "pod get") jq --arg i "$3" '.[] | select(.id == $i) | {networkVolumeId: "lmd1pfah3y"} + .
     + (if .desiredStatus == "RUNNING" then {ssh: {ip: "10.0.0.1", port: ({"poda":1001,"podb":1002,"podc":1003}[$i])}} else {} end)' "$FAKE/pods.json";;
  "gpu list") cat "$FAKE/gpus.json";;
  "network-volume list") echo '[{"id":"lmd1pfah3y","name":"uor-shared-EUR-NO-1","dataCenterId":"EUR-NO-1","size":200}]';;
  "network-volume create") echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9;;
  "pod delete"|"pod create") echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9;;
  *) echo "fake runpodctl: unhandled $*" >&2; exit 1;;
esac
S
cat > "$W/bin/ssh" <<'S'
#!/usr/bin/env bash
port=''; while [ $# -gt 0 ]; do case $1 in -p) port=$2; shift 2;; -i|-o) shift 2;; root@*) shift; break;; *) shift;; esac; done
echo "ssh $port $*" >> "$FAKE/calls"
case "$*" in *probe_ok*) cat "$FAKE/probe-$port" 2>/dev/null;; esac
exit 0
S
cat > "$W/bin/gh" <<'S'
#!/usr/bin/env bash
echo "gh $*" >> "$FAKE/calls"; echo "REAL GH CALLED IN DRY RUN" >&2; exit 9
S
chmod +x "$W/bin/"*

pass=0 fail=0
ok()  { pass=$((pass + 1)); echo "ok   $1"; }
bad() { fail=$((fail + 1)); echo "FAIL $1"; sed 's/^/     | /' "$W/out"; }
expect() {  # expect NAME RC PATTERN -- uor-pod args...
  local name=$1 want=$2 pat=$3 rc=0; shift 4
  "$TOOL" "$@" > "$W/out" 2>&1 || rc=$?
  if [ "$rc" = "$want" ] && grep -qE "$pat" "$W/out"; then ok "$name"; else echo "     rc=$rc want=$want pattern=$pat"; bad "$name"; fi
}

# The lease writes are local state (dry run only skips remote/GitHub/pod mutations).
expect "lease A gpus 0,1 to claude" 0 "Leased poda gpus 0,1 to claude" -- lease poda --lab claude --gpus 0,1 --purpose "test A" --hours 2
expect "lease conflict refused" 1 "under another lab's live lease" -- lease poda --lab codex --gpus 1 --purpose x --hours 1
expect "lease out-of-range refused" 1 "out of range" -- lease podb --lab codex --gpus 3 --purpose x --hours 1
expect "bad lab refused" 1 "lab must match" -- lease poda --lab Claude! --gpus 0 --purpose x --hours 1
expect "renew own lease" 0 "Renewed claude on poda" -- renew poda --lab claude --hours 3
expect "renew without lease refused" 1 "holds no lease" -- renew poda --lab codex
expect "run on leased gpu (dry)" 0 "DRY-RUN: on poda" -- run poda --lab claude --gpu 1 -- echo hello
expect "run on unleased gpu refused" 1 "holds no live lease" -- run podb --lab claude --gpu 0 -- echo hi
expect "forbidden A100 refused" 1 "not used for this trainer" -- up --lab codex --gpu a100 --purpose x --hours 1
# podb (4090, no lease) is a free compatible GPU: up must point at it
expect "up refuses when a free GPU exists" 3 "Free GPUs already exist: pod podb" -- up --lab codex --purpose x --hours 1 --count 1
expect "up over pod cap refused" 1 "cap: 3 pod\\(s\\) already running" -- up --lab codex --purpose x --hours 1 --count 2
export UOR_POD_MAX_PODS=9 UOR_POD_MAX_RATE=99
expect "up: no 5090 in EUR-NO-1 -> EU-RO-1, never 4090" 0 "Creating 2 x 5090 .* in EU-RO-1 volume dryrunvolume \\(non-canonical\\)" -- up --lab codex --purpose x --hours 1 --count 2
expect "up: creates uor-shared-EU-RO-1 lazily" 0 "network-volume create --name uor-shared-EU-RO-1 --size 100 --data-center-id EU-RO-1" -- up --lab codex --purpose x --hours 1 --count 2
expect "up --gpu 4090 explicit -> EUR-NO-1 canonical" 0 "Creating 2 x 4090 .* in EUR-NO-1 for lab" -- up --lab codex --gpu 4090 --purpose x --hours 1 --count 2
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up: no 5090 in volume DCs refuses (no silent fallback)" 1 "no 5090 stock in EUR-NO-1 EUR-IS-1.*--wait" -- up --lab codex --purpose x --hours 1 --count 2
if grep -q "4090 \$0.74: EUR-NO-1\*=Low" "$W/out"; then ok "refusal prints the stock table"; else bad "refusal prints the stock table"; fi
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up --wait retries every 5 min" 0 "would retry every 5 min" -- up --lab codex --purpose x --hours 1 --count 2 --wait --wait-hours 1
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up --allow-off-volume places 5090 in EU-RO-1 without volume" 0 "Creating 2 x 5090 .* in EU-RO-1 OFF-VOLUME" -- up --lab codex --purpose x --hours 1 --count 2 --allow-off-volume
UOR_POD_MAX_RATE=4 expect "up over rate cap refused" 1 "exceeds \\\$4/h" -- up --lab codex --purpose x --hours 1 --count 3
unset UOR_POD_MAX_PODS UOR_POD_MAX_RATE
expect "down of a non-canonical pod needs --confirm-archived" 1 "non-canonical volume volro" -- down podc --lab codex
expect "down non-canonical after archiving (dry)" 0 "DRY-RUN: runpodctl pod delete podc" -- down podc --lab codex --confirm-archived
expect "down refused while another lab leases" 1 "another lab holds a live lease" -- down poda --lab codex
expect "down by the lease holder (dry)" 0 "DRY-RUN: runpodctl pod delete poda" -- down poda --lab claude
expect "release" 0 "Released claude's lease on poda" -- release poda --lab claude
expect "reap: deletes idle unleased B, keeps busy C" 0 "podb: no live lease, idle 30 min — deleting" -- reap
if grep -q "podc: no live lease, GPUs busy — kept" "$W/out"; then ok "reap keeps busy C"; else bad "reap keeps busy C"; fi
if grep -q "poda: no live lease, idle 0 min" "$W/out"; then ok "reap starts A's idle clock"; else bad "reap starts A's idle clock"; fi
expect "status flags unleased pods" 0 "podb .*NO LIVE LEASE" -- status
if grep -q must-not-print "$W/out"; then bad "status hides pod env"; else ok "status hides pod env"; fi
expect "log shows ledger" 0 "lease claude poda" -- log -n 50
if grep -q "REAL" "$W/out" "$FAKE/calls" 2>/dev/null; then bad "no real mutation in dry run"; else ok "no real mutation in dry run"; fi
if ls "$UOR_POD_STATE/leases/poda/"*.json >/dev/null 2>&1; then bad "release removed the lease file"; else ok "release removed the lease file"; fi

echo "passed $pass, failed $fail"
[ "$fail" = 0 ]
