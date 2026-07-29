#!/usr/bin/env python3
import sys
import json

# Pinned previous record (Gate C Rule 1+2 on the small corpus test fixture)
# Determined from trend_output/score_report.json on main.
#
# Era note (issue #281, 2026-07-29): re-pinned for the single-key store +
# read-time query-beam semantics (the #244 decision). Fixture-scale price
# vs the pre-#281 pins (0.317 / 9.86): top-1 -1.7pp, +0.3757 bits/token.
# D3-scale price (the decision scale, #244 matrix): top-1 +-0.0pp,
# +0.087 bits/token, at 2.56x fewer store keys. The fixture's 150-story
# miniature dilutes sparse beam evidence harder than D3; both deltas are
# disclosed on PR #283. The regression thresholds are unchanged.
PINNED_TOP1_AGREEMENT = 0.300  # ~30.0% (post-#281)
PINNED_BITS_PER_TOKEN = 10.24  # bits/token (post-#281)

# Regression Thresholds
MAX_TOP1_DROP = 0.02           # fail if top-1 drops > 2 points (0.02)
MAX_BPT_WORSEN = 0.1           # fail if bits/token worsens (increases) > 0.1

def main():
    if len(sys.argv) < 2:
        print("Usage: check_gate_c_regression.py <path/to/score_report.json>")
        sys.exit(1)

    report_path = sys.argv[1]
    with open(report_path, 'r') as f:
        report = json.load(f)
    
    rule12 = report.get('gate_c', {}).get('rule12_precedence')
    if not rule12:
        print("Error: 'rule12_precedence' metrics not found in the report.")
        sys.exit(1)
        
    top1 = rule12.get('top1_agreement', 0.0)
    bpt = rule12.get('bits_per_token', 0.0)
    
    print(f"Gate C (Rule 1+2) Current: top-1={top1:.4f} ({top1*100:.1f}%), bits/token={bpt:.4f}")
    print(f"Gate C (Rule 1+2) Pinned : top-1={PINNED_TOP1_AGREEMENT:.4f} ({PINNED_TOP1_AGREEMENT*100:.1f}%), bits/token={PINNED_BITS_PER_TOKEN:.4f}")
    
    failed = False
    
    if top1 < (PINNED_TOP1_AGREEMENT - MAX_TOP1_DROP):
        print(f"🚨 REGRESSION ALARM: top-1 agreement dropped by more than {MAX_TOP1_DROP*100:.1f} points!")
        print(f"   Delta: {(top1 - PINNED_TOP1_AGREEMENT)*100:.2f} points")
        failed = True
        
    if bpt > (PINNED_BITS_PER_TOKEN + MAX_BPT_WORSEN):
        print(f"🚨 REGRESSION ALARM: bits/token worsened by more than {MAX_BPT_WORSEN:.2f}!")
        print(f"   Delta: {bpt - PINNED_BITS_PER_TOKEN:+.4f} bits/token")
        failed = True
        
    if failed:
        print("Gate C regression check FAILED.")
        sys.exit(1)
    else:
        print("Gate C regression check PASSED.")
        sys.exit(0)

if __name__ == '__main__':
    main()
