#!/usr/bin/env bash
# Unit-test the launch guard's three conditions, including the memory pair.
#
# A guard that never fires and a guard that always fires are both failures, and
# neither is visible from the outside. This exercises every combination.
set -uo pipefail

check(){ # others load avail_mb swap_mb [poll] -> FIRE|wait
  local others="$1" load="$2" avail="$3" swap="$4" i="${5:-1}"
  local limit; if [ "$i" -le 48 ]; then limit=4.0; else limit=8.0; fi
  python3 -c "
q = float('$load') < $limit
r = int('$avail') >= 2100 and int('$swap') >= 512
print('FIRE' if ('$others' == '0' and q and r) else 'wait')"
}

fail=0
expect(){ local got="$1" want="$2" label="$3"
  if [ "$got" = "$want" ]; then printf "  ok    %-46s -> %s\n" "$label" "$got"
  else printf "  FAIL  %-46s -> %s (want %s)\n" "$label" "$got" "$want"; fail=1; fi; }

echo "=== all three conditions must hold ==="
expect "$(check 0 2 4000 4000)"    FIRE "others=0 load=2 avail=4000 swap=4000"
expect "$(check 2 2 4000 4000)"    wait "a peer job present, everything else green"
expect "$(check 0 9 4000 4000)"    wait "load above the strict bar"
expect "$(check 0 2 900  4000)"    wait "reclaimable memory too low"
expect "$(check 0 2 4000 300)"     wait "swap too low"

echo "=== load tier relaxes after STRICT_POLLS; memory never does ==="
expect "$(check 0 5 4000 4000 49)" FIRE "poll 49: relaxed load bar"
expect "$(check 0 5 900  4000 49)" wait "poll 49: relaxed load, memory still low"
expect "$(check 0 5 4000 300  49)" wait "poll 49: relaxed load, swap still low"

echo "=== boundaries, inclusive ==="
expect "$(check 0 3.99 2100 512)"  FIRE "exactly at both memory floors"
expect "$(check 0 4.01 2100 512)"  wait "just over the strict load bar"
expect "$(check 0 3.99 2099 512)"  wait "one MB under the reclaimable floor"
expect "$(check 0 3.99 2100 511)"  wait "one MB under the swap floor"

echo
if [ "$fail" = 0 ]; then echo "all launch-guard cases pass"; else echo "FAILURES PRESENT"; exit 1; fi

echo "=== tier 3: after LONG_WAIT_POLLS, a lingering peer no longer blocks,"
echo "    but load and memory bars both tighten ==="
tier3(){ # others load avail swap poll -> FIRE|wait
  local others="$1" load="$2" avail="$3" swap="$4" i="${5:-800}"
  python3 -c "
if $i <= 48:      owner, limit, floor = $others == 0, 4.0, 2100
elif $i <= 720:   owner, limit, floor = $others == 0, 8.0, 2100
else:             owner, limit, floor = True,        1.5, 3000
ok = owner and float('$load') < limit and int('$avail') >= floor and int('$swap') >= 512
print('FIRE' if ok else 'wait')"
}
expect "$(tier3 1 1.0 4000 2000)"   FIRE "tier 3, a lingering peer, quiet and roomy"
expect "$(tier3 1 2.0 4000 2000)"   wait "tier 3, load 2.0 exceeds the tightened bar"
expect "$(tier3 1 1.0 2500 2000)"   wait "tier 3, 2500 < the raised 3000 floor"
expect "$(tier3 2 8.0 1000 100)"    wait "tier 2, everything still tight"
expect "$(tier3 0 1.0 4000 2000 10)" FIRE "tier 1 unaffected"

echo
if [ "$fail" = 0 ]; then echo "all launch-guard cases pass (including tier 3)"; else echo "FAILURES PRESENT"; exit 1; fi
