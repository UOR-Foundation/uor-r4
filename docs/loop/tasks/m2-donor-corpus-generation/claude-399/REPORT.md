# REPORT: m2-donor-corpus-generation, claude-399 (Casey's production-donor answers recorded as D87)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **No corpus was generated.** This unit records a decision only. Nothing has been measured on DeepSeek V4.1 Flash for this workload. The rates and costs quoted in the chat come from published benchmarks, not from runs of this project.
- **The run caps are an owner term with no tool enforcement of "this run only".** The pod tool reads `UOR_POD_MAX_RATE`, so the production unit sets it to 35 for that run alone. Nothing stops a later session from setting it; D87's text is the only limit.

## What changed

- **D87** in `docs/integration/DECISIONS.md`:
  - the donor is DeepSeek V4.1 Flash (MIT), with gpt-oss-120b as the fallback;
  - the reasoning text is kept with the answer, both as hard text targets;
  - the first corpus is 1B kept tokens;
  - the caps are 4 pods and $35/h for that run only.
- **The 16 Nov entry** in `docs/rules/changes.md`, quoting Casey's answers.
- **`PLAN.md`:** a new owner-direction row, and one sentence on D87 in the M2 route.

## Measurements

None. This is a record.

## Checks run at the head

- `loop pr-check --local`, at the head of this PR.
- `git diff --check` is clean.

## Skills used

- `scope-guard`: three owner-path edits named in the ORDER, nothing else.

## Still open

1. **The production unit:**
   - pin the checkpoint and check its licence;
   - build a prompt bank large enough for 1B kept tokens with reasoning;
   - point the generation client at vLLM;
   - measure one 8×H200 pod for one hour, then fan out inside the run caps;
   - seal the receipt.
2. **Wick's role after the first corpus**, and whether a public pretraining corpus is used. Both need an owner decision.

## Plain summary

Casey chose DeepSeek V4.1 Flash as the teacher, kept its reasoning, asked for 1B tokens first, and allowed up to $35/h for that run. This change writes those words into the decision log, the rules change log and the plan, so the production run can cite them.
