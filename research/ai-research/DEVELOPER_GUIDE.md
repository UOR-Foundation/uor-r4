# MUDBench Developer Guide

## Document Status
Version: 0.1
Status: Draft
Purpose: Provide practical guidance for developers building agents, extending the simulator, or contributing to MUDBench.

---

# 1. Overview

MUDBench is a benchmark environment designed to evaluate AI agents in a structured text-world simulation.

The system combines:

- a deterministic simulation engine
- a structured world environment
- an agent interaction protocol
- a capability-based scoring system
- replay and telemetry infrastructure

---

# 2. Repository Structure

Typical repository layout:

/mudbench
  core
  world
  agents
  evaluation
  replay
  cli
  scenarios
  docs

---

# 3. Running the Simulator

mudbench run

mudbench run --agent ./agents/example_agent.py

mudbench run --scenario forest_trial

---

# 4. Writing an Agent

while True:
    observation = receive()
    action = decide(observation)
    send(action)

---

# 5. Minimal Python Agent

import sys, json

while True:
    obs = json.loads(sys.stdin.readline())
    action = obs["action_space"][0]
    print(json.dumps({"action": action}))
    sys.stdout.flush()

---

# 6. Simulation Loop

1. observation
2. action
3. validation
4. world update
5. scoring
6. replay logging

---

# 7. Benchmark Runs

max_steps = 1000  
time_limit = 5s  
seed = fixed  

---

# 8. Replay Logs

/replay/run_x.json

---

# 9. Scoring Output

navigation, memory, planning, tactical, social, efficiency

---

# 10. Scenarios

/scenarios/

---

# 11. Rules

- deterministic
- observable
- replay-safe

---

# 12. Debugging

- replay inspection
- step tracing

---

# 13. Performance

- fast loop
- low overhead

---

# 14. Contributions

- modular
- tested
- documented

---

# 15. Local Validation Commands

Run local validation hooks from the repository root:

```bash
make lint-local
make test-local
make determinism-local
make ci-local
```

Notes:

- `lint-local` uses `ruff` when available, falls back to `flake8`, and fails explicitly if neither is installed.
- `determinism-local` runs the real benchmark-runtime determinism gate suite (`tests/benchmark/test_real_determinism_gate.py`) with fail-fast behavior.
- `ci-local` chains lint, tests, and determinism checks in fail-fast order.

---

# 16. Closing

MUDBench evaluates real agent capability, not tricks.
