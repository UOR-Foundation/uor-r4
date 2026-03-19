# Local Agent Wrapper Examples

These examples show how an external local-process agent plugs into MUDBench through the current `run --agent-command` hook.

## Current contract

The current local-process bridge is single-shot per request:

1. MUDBench starts the configured local command
2. MUDBench writes one observation JSON object to the process `stdin`
3. The process writes one action JSON object to `stdout`
4. The process exits

The wrapper must not invent extra protocol fields. The action payload must be:

```json
{"action":"move east"}
```

The observation payload includes the current protocol fields used by MUDBench, including:

- `run_id`
- `step`
- `location`
- `description`
- `exits`
- `entities`
- `inventory`
- `health`
- `messages`
- `action_space`
- `remaining_steps`
- `protocol_version`

## Example wrappers

- `strict_wrapper_example.py`
  - validates the expected observation shape
  - returns a deterministic valid action
- `deterministic_rule_agent.py`
  - simple rule-based policy for local testing
  - prefers `take` then `move` then `attack` then `look` then `wait`
- `mock_llm_wrapper.py`
  - shows prompt construction and JSON output parsing
  - uses a deterministic in-process mock instead of a real API

## Example commands

Run a tiny canonical scenario with the deterministic rule wrapper:

```bash
python -m src.cli.main run \
  --scenario tiny-fetch-quest \
  --actor-id agent-a \
  --agent-command "python examples/agents/deterministic_rule_agent.py"
```

Run the mock LLM-style wrapper:

```bash
python -m src.cli.main run \
  --scenario tiny-hidden-key \
  --actor-id agent-a \
  --agent-command "python examples/agents/mock_llm_wrapper.py"
```

Use the strict example when you want a small reference implementation of the observation-in / action-out contract:

```bash
python examples/agents/strict_wrapper_example.py
```
