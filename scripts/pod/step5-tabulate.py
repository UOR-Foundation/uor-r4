#!/usr/bin/env python3
"""Tabulate the Step 5 knowledge A/B (#820) and apply its frozen decision rule.

Tabulation only (reads sealed JSON reports; no model runs):

  step5-tabulate.py RUN_ROOT [--clean DIR] [--json OUT.json]

RUN_ROOT is the `R` of scripts/pod/step5-knowledge.sh. Arms: A (current mix) and
K (current mix with 25% knowledge corpus), seeds 1 and 2. It reads
  base-{A,K}-s{1,2}/report.json                 base train report (final ts-valid NLL)
  eval-knowledge-{base,ft}-{A,K}-s{1,2}/evaluation.json   knowledge held-out NLL
  session-{sieve,off}-{A,K}-s{1,2}/report.json  D19 grounded sessions
  grade-{A,K}-s{1,2}/report.json                open 232-request panel grades
and the clean subsets in docs/research/step5/panel-clean (clean-ids.txt and
wellposed-ids.txt).

Decision rule (frozen in docs/research/step5/knowledge-ab.md before any run):
  Manipulation check: K's knowledge held-out NLL (base) is at least 0.10 nats
    below A's on both seeds; otherwise the corpus did not land and the panel is
    not read (instrument/data-path failure).
  Panel delta: d_s = acceptable(K, s) - acceptable(A, s) on all 232 rows;
    mean_d = (d_1 + d_2) / 2; pooled exact McNemar over the 464 (request, seed)
    pairs.
  Clean guard: d_s on the 143-row clean subset is > 0 on both seeds.
  Instruction guard: pooled D19 Instruction passes (sieve on, 2 x 98 turns) of K
    >= pooled A - max(3, |A_s1 - A_s2|).
  PROMOTE  mean_d >= 10, p < 0.05, clean guard and instruction guard hold.
  BLOCKED  panel criterion (mean_d >= 10, p < 0.05, clean guard) holds but the
           instruction guard fails.
  NULL     mean_d < 5, or d_1 and d_2 have opposite signs.
  WEAK     anything else.
"""
import argparse
import json
import math
import os
import sys

ARMS = ('A', 'K')
SEEDS = (1, 2)
HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_CLEAN = os.path.join(HERE, '..', '..', 'docs', 'research', 'step5', 'panel-clean')


def load(path):
    if not os.path.exists(path):
        return None
    with open(path, encoding='utf-8') as handle:
        return json.load(handle)


def ids(path):
    with open(path, encoding='utf-8') as handle:
        return [line.strip() for line in handle if line.strip()]


def mcnemar_exact(b, c):
    """Two-sided exact McNemar p (the chat-grade definition)."""
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    tail = sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n
    return min(1.0, 2 * tail)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('root')
    parser.add_argument('--clean', default=DEFAULT_CLEAN)
    parser.add_argument('--json')
    args = parser.parse_args()
    R = args.root
    clean = ids(os.path.join(args.clean, 'clean-ids.txt'))
    wellposed = ids(os.path.join(args.clean, 'wellposed-ids.txt'))
    out = {'root': os.path.abspath(R), 'arms': {}}

    ok = {}
    for arm in ARMS:
        for seed in SEEDS:
            key = f'{arm}-s{seed}'
            row = {}
            base = load(os.path.join(R, f'base-{key}', 'report.json'))
            row['base_ts_valid_nll'] = (base or {}).get('final', {}).get('nll')
            for stage in ('base', 'ft'):
                ev = load(os.path.join(R, f'eval-knowledge-{stage}-{key}', 'evaluation.json'))
                row[f'knowledge_nll_{stage}'] = ev['full']['nll'] if ev else None
            for recall in ('sieve', 'off'):
                session = load(os.path.join(R, f'session-{recall}-{key}', 'report.json'))
                if session:
                    cats = session['arms']['default']['by_category']
                    row[f'session_{recall}'] = {c: v['pass'] for c, v in cats.items()}
                    row[f'session_{recall}_total'] = sum(v['pass'] for v in cats.values())
                    row[f'session_{recall}_of'] = sum(v['of'] for v in cats.values())
            grade = load(os.path.join(R, f'grade-{key}', 'report.json'))
            if grade:
                ok[key] = {r['id']: bool(r['grades']['fluent'] and r['grades']['relevant'])
                           for r in grade['rows']}
                row['panel_all'] = sum(ok[key].values())
                row['panel_wellposed'] = sum(ok[key][i] for i in wellposed)
                row['panel_clean'] = sum(ok[key][i] for i in clean)
            out['arms'][key] = row

    print(f"{'arm':6} {'tsNLL':>7} {'kNLLb':>7} {'kNLLft':>7} {'panel':>6} {'well':>5} {'clean':>5} "
          f"{'D19 sieve':>10} {'D19 off':>8} {'Instr':>5}")
    for key, row in out['arms'].items():
        def f(v, fmt='{:.4f}'):
            return '-' if v is None else fmt.format(v)
        print(f"{key:6} {f(row.get('base_ts_valid_nll')):>7} {f(row.get('knowledge_nll_base')):>7} "
              f"{f(row.get('knowledge_nll_ft')):>7} {f(row.get('panel_all'), '{}'):>6} "
              f"{f(row.get('panel_wellposed'), '{}'):>5} {f(row.get('panel_clean'), '{}'):>5} "
              f"{f(row.get('session_sieve_total'), '{}'):>10} {f(row.get('session_off_total'), '{}'):>8} "
              f"{f((row.get('session_sieve') or {}).get('Instruction'), '{}'):>5}")

    need = [f'{a}-s{s}' for a in ARMS for s in SEEDS]
    if not all(k in ok for k in need):
        print('incomplete: grades missing for', [k for k in need if k not in ok])
        decision = 'INCOMPLETE'
    else:
        a = out['arms']
        manip = all(
            a[f'K-s{s}'].get('knowledge_nll_base') is not None
            and a[f'A-s{s}'].get('knowledge_nll_base') is not None
            and a[f'A-s{s}']['knowledge_nll_base'] - a[f'K-s{s}']['knowledge_nll_base'] >= 0.10
            for s in SEEDS)
        d = {s: a[f'K-s{s}']['panel_all'] - a[f'A-s{s}']['panel_all'] for s in SEEDS}
        dc = {s: a[f'K-s{s}']['panel_clean'] - a[f'A-s{s}']['panel_clean'] for s in SEEDS}
        b = sum(ok[f'K-s{s}'][i] and not ok[f'A-s{s}'][i] for s in SEEDS for i in ok['A-s1'])
        c = sum(ok[f'A-s{s}'][i] and not ok[f'K-s{s}'][i] for s in SEEDS for i in ok['A-s1'])
        p = mcnemar_exact(b, c)
        mean_d = (d[1] + d[2]) / 2
        instr = {arm: [(a[f'{arm}-s{s}'].get('session_sieve') or {}).get('Instruction') for s in SEEDS]
                 for arm in ARMS}
        if None in instr['A'] or None in instr['K']:
            instr_ok = None
        else:
            tol = max(3, abs(instr['A'][0] - instr['A'][1]))
            instr_ok = sum(instr['K']) >= sum(instr['A']) - tol
        clean_ok = dc[1] > 0 and dc[2] > 0
        panel_ok = mean_d >= 10 and p < 0.05 and clean_ok
        if not manip:
            decision = 'INSTRUMENT: knowledge corpus did not land (manipulation check failed); panel not read'
        elif panel_ok and instr_ok:
            decision = 'PROMOTE'
        elif panel_ok and instr_ok is False:
            decision = 'BLOCKED (instruction guard failed)'
        elif instr_ok is None and panel_ok:
            decision = 'INCOMPLETE (session reports missing)'
        elif mean_d < 5 or d[1] * d[2] < 0:
            decision = 'NULL'
        else:
            decision = 'WEAK'
        out['comparison'] = {
            'manipulation_check': manip, 'delta_all': d, 'delta_clean': dc, 'mean_delta_all': mean_d,
            'mcnemar_pooled': {'K_only': b, 'A_only': c, 'p': p},
            'instruction_sieve': instr, 'instruction_guard': instr_ok, 'clean_guard': clean_ok,
        }
        print(f"manipulation check (knowledge NLL A-K >= 0.10 on both seeds): {manip}")
        print(f"panel delta K-A: s1 {d[1]:+d}, s2 {d[2]:+d}, mean {mean_d:+.1f}; clean s1 {dc[1]:+d}, s2 {dc[2]:+d}")
        print(f"pooled McNemar over 464 pairs: K-only {b}, A-only {c}, p = {p:.4g}")
        print(f"D19 Instruction (sieve) A {instr['A']} K {instr['K']}: guard {instr_ok}")
    out['decision'] = decision
    print(f'DECISION: {decision}')
    if args.json:
        with open(args.json, 'x', encoding='utf-8') as handle:
            json.dump(out, handle, indent=1)
    return 0


if __name__ == '__main__':
    sys.exit(main())
