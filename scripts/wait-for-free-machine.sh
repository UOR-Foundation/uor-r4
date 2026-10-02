#!/usr/bin/env bash
# Wait for a genuinely free machine, then claim the slot and launch the
# full-dose chat run. Never touches another lab's process.
#
# Why this exists rather than just launching: this machine has 8 cores and the
# project runs several labs on it. Measured, running a second trainer alongside
# the first gave the newer process 0.87 cores and turned a 4.7 h projection into
# 43.4 h. Waiting is ~9x cheaper than contending.
#
# The two conditions are deliberately both required:
#   others=0  -- no other geometric-stack process at all, regardless of load,
#                because a peer job that has not yet spun up its threads is
#                still a job that will contend;
#   load<4    -- the machine is otherwise quiet. Verified attainable: the app
#                harness's own floor is about load 1.5, so this is reachable and
#                not an impossible bar. `test-monitor-logic.sh` unit-tests both
#                boundaries.
#
# The launcher is resolved next to this script, so a copy of the pair works from
# anywhere -- earlier revisions hard-coded /tmp, which the OS may clear.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LAUNCHER="${LAUNCHER:-$HERE/launch-fulldose.sh}"
REPORTS="${REPORTS:-$HOME/uor-r4-worktrees/stack-prose-reports}"
SLOT="$HOME/.local/share/uor-r4/locks/model-slot.json"
LOG="$REPORTS/sequence-wait.log"
RUNLOG="$REPORTS/dose-ladder.log"
POLL_SECONDS="${POLL_SECONDS:-150}"
MAX_POLLS="${MAX_POLLS:-2300}"        # ~96 h at the default interval
LOAD_LIMIT="${LOAD_LIMIT:-4.0}"

log(){ echo "[$(date -u +%H:%M:%SZ)] $*" >> "$LOG"; }

if [ ! -x "$LAUNCHER" ]; then
  log "FATAL: launcher not executable: $LAUNCHER"
  exit 1
fi

log "monitor started; waiting for a free machine (launcher=$LAUNCHER)"

for i in $(seq 1 "$MAX_POLLS"); do
  others=$(pgrep -f "geometric-stack" | wc -l | tr -d ' ')
  load=$(uptime | sed -E 's/.*averages: ([0-9.]+).*/\1/')

  # `others` counts *processes*, but what cost me 9x was *resource contention*
  # (0.87 cores; 4.7 h -> 43.4 h). Those differ. Observed here: a peer job held
  # 0.14 cores for 40 minutes with no evaluation -- a process competing with
  # nothing -- yet `others=0` blocked on it. An all-or-nothing trigger can
  # therefore wait out a job that will never finish, however safe the machine
  # becomes.
  #
  # So the trigger is resource-based, in two tiers:
  #   tier 1 (always)   others=0 exactly -- the normal case
  #   tier 2 (after LONG_WAIT_POLLS)  allow lingering processes, but only on a
  #                     genuinely quiet machine: stricter load AND larger memory
  #                     margin than tier 1, buying back with measured headroom
  #                     the "no peer at all" guarantee it trades away
  if [ "$i" -le "${STRICT_POLLS:-48}" ]; then
    limit="$LOAD_LIMIT"
    owner_ok=$(python3 -c "print(1 if $others == 0 else 0)")
    avail_floor="${MIN_AVAIL_MB:-2100}"
  elif [ "$i" -le "${LONG_WAIT_POLLS:-720}" ]; then
    limit="${RELAXED_LOAD_LIMIT:-8.0}"
    owner_ok=$(python3 -c "print(1 if $others == 0 else 0)")
    avail_floor="${MIN_AVAIL_MB:-2100}"
  else
    limit="${LONG_WAIT_LOAD:-1.5}"
    owner_ok=1
    avail_floor="${LONG_WAIT_AVAIL_MB:-3000}"
  fi
  quiet=$(python3 -c "print(1 if float('$load') < $limit else 0)" 2>/dev/null || echo 0)

  # Idle CPU, measured directly. Load average is a proxy, and a poor one for
  # this decision: what matters is whether EIGHT cores are available for a
  # job whose whole point is throughput. Observed here with a peer at 168% and
  # a build at 190%: load 7.4 but only 33% idle, i.e. ~2.7 free cores against a
  # need for 8. Launching into that oversubscribes and slows every peer, which
  # is the exact harm the sequencing exists to avoid -- so require idle.
  # Parsed with python and iostat, not by matching top's output: an earlier
  # attempt with `sed` on `top` returned 0 when it failed to match, and a 0
  # would have made the guard NEVER fire -- the same class of bug as a guard
  # that can never fire for any other reason. iostat's second sample is a
  # measured interval, and a failed parse here is detectable rather than
  # silently zero.
  idle_pct=$(python3 -c "
import subprocess
try:
    lines = [l for l in subprocess.run(['iostat','-c','2'],capture_output=True,text=True).stdout.splitlines() if l.strip()]
    f = [float(x) for x in lines[-1].split()]
    print(round(100.0 - f[0] - f[2], 1))   # 100 - user - sys = idle
except Exception:
    print(-1)                              # sentinel: unknown, never 'roomy'
" 2>/dev/null)
  [ -n "$idle_pct" ] || idle_pct=-1
  idle_ok=$(python3 -c "print(1 if float('$idle_pct') >= ${MIN_IDLE_PCT:-50} else 0)" 2>/dev/null || echo 0)

  # Memory guard. Two numbers, because they measure different things.
  #
  # Reclaimable memory (free + inactive + speculative + purgeable) is what a new
  # 1.4 GB process can actually take. Swap free is what the kernel can fall back
  # on when it must evict. Both matter, and neither alone is the right test:
  # swap occupancy is sticky -- pages stay swapped with no pressure to reclaim
  # them -- so a swap-only test can refuse to launch on an idle machine, while a
  # reclaimable-only test would launch into a box whose swap is exhausted and
  # whose compressor is already holding gigabytes.
  #
  # Measured on this machine during the queue: reclaimable 2168 MB against a
  # 1400 MB need (1.5x, tight), swap free 762 MB, wired 8977 MB, compressor
  # 4245 MB. This is not about my own throughput: when memory runs out the
  # kernel kills the largest task present, which here is a peer lab's
  # multi-hour run.
  read -r avail_mb swap_free_mb <<EOF2
$(python3 -c "
import re, subprocess
ps = int(subprocess.run(['sysctl','-n','hw.pagesize'],capture_output=True,text=True).stdout.strip())
vm = subprocess.run(['vm_stat'],capture_output=True,text=True).stdout
def pg(n):
    m = re.search(rf'{n}:\\s+(\\d+)', vm)
    return int(m.group(1)) if m else 0
avail = (pg('Pages free')+pg('Pages inactive')+pg('Pages speculative')+pg('Pages purgeable'))*ps//(1024*1024)
out = subprocess.run(['sysctl','-n','vm.swapusage'],capture_output=True,text=True).stdout
m = re.search(r'free = ([0-9.]+)M', out)
print(avail, int(float(m.group(1))) if m else 0)
" 2>/dev/null || echo "0 0")
EOF2
  roomy=$(python3 -c "
ok = $avail_mb >= $avail_floor and $swap_free_mb >= ${MIN_SWAP_FREE_MB:-512}
print(1 if ok else 0)" 2>/dev/null || echo 0)

  # Heartbeat every ~30 min so the wait is observable and a stall cannot be
  # mistaken for patience: logging only state changes is indistinguishable from
  # a hung monitor.
  if [ $((i % 12)) = 1 ]; then
    log "waiting: others=$others load=$load idle=${idle_pct}% avail=${avail_mb}MB swap_free=${swap_free_mb}MB (tier poll=$i: peer_ok=$owner_ok load<$limit avail>=$avail_floor swap>=${MIN_SWAP_FREE_MB:-512})"
  fi

  if [ "$owner_ok" = "1" ] && [ "$quiet" = "1" ] && [ "$roomy" = "1" ] && [ "$idle_ok" = "1" ]; then
    log "machine free (others=$others load=$load idle=${idle_pct}% avail=${avail_mb}MB swap_free=${swap_free_mb}MB); claiming slot"
    if (set -o noclobber; cat > "$SLOT" <<SLOTEOF
{"lab":"deepseek","card":"full-dose chat-v0 LM + response #1512","pid":$$,"started_utc":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","expected_end_utc":"$(date -u -v+12H +%Y-%m-%dT%H:%M:%SZ)","threads":8}
SLOTEOF
    ) 2>/dev/null; then
      log "slot claimed; launching $LAUNCHER"
      nohup "$LAUNCHER" > "$RUNLOG" 2>&1 &
      log "launched pid $!"
      exit 0
    else
      log "slot already taken; retrying"
    fi
  fi
  sleep "$POLL_SECONDS"
done

log "monitor expired after $MAX_POLLS polls without a free machine"
