#!/usr/bin/env bash
tri(){ local others="$1" load="$2" swapmb="$3" i="${4:-1}"
  local limit; if [ "$i" -le 48 ]; then limit=4.0; else limit=8.0; fi
  local q r
  q=$(python3 -c "print(1 if float('$load') < $limit else 0)")
  r=$(python3 -c "print(1 if $swapmb >= 1500 else 0)")
  if [ "$others" = "0" ] && [ "$q" = "1" ] && [ "$r" = "1" ]; then echo FIRE; else echo wait; fi; }
printf "  others=0 load=2 swap=5000  -> %s (expect FIRE)\n" "$(tri 0 2 5000)"
printf "  others=0 load=2 swap=900   -> %s (expect wait: swap tight)\n" "$(tri 0 2 900)"
printf "  others=0 load=9 swap=5000  -> %s (expect wait: load)\n" "$(tri 0 9 5000)"
printf "  others=2 load=2 swap=5000  -> %s (expect wait: peer job)\n" "$(tri 2 2 5000)"
printf "  others=0 load=5 swap=5000 poll49 -> %s (expect FIRE: relaxed)\n" "$(tri 0 5 5000 49)"
printf "  others=0 load=5 swap=900  poll49 -> %s (expect wait: swap tight)\n" "$(tri 0 5 900 49)"
