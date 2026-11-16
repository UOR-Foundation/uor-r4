# Prep sheet: the donor throughput probe on wick (m2-donor-corpus-generation, claude-398, 15 Nov 2026)

Lab: claude. Item: [m2-donor-corpus-generation](../../roadmap/items/m2-donor-corpus-generation.md) (#2848). Order: [claude-398](../../loop/tasks/m2-donor-corpus-generation/claude-398/ORDER.md). Code: branch `claude/claude-398-donor-throughput`, PR #4025. Release cited at dispatch: D86 (§3.10).

TEST FITNESS: PENDING

## The question

How many kept-answer tokens per second can `openai/gpt-oss-20b` give on wick, and with which settings? The pilot (claude-390) measured 220.7 kept and 503.6 generated tokens/s at one setting, so 100M kept tokens would take about 126 hours. The production pass `distill-d1` is sized from this probe's best measured rate. Each lever is a hypothesis, measured on its own against the pilot's setting.

The pilot's server log suggests a cause. With 32 slots busy, a decode step took about 55 ms and the GPU averaged 47%. Its requests send `top_k 0`, so the engine's CPU sampler handles the full 201k-token vocabulary for every slot at every step. Arms A8a and A8b test this. `b12000` runs on the GPU only the leading samplers of the chain that have a GPU version, and the default chain starts `penalties;dry;…`, where `dry` has none. So `--backend-sampling` alone changes nothing; the first smoke confirmed this, giving shards byte-identical to the CPU run. At the request's values (`top_k 0`, `top_p 1`, `min_p 0`, no penalties, DRY or XTC), every sampler but `temperature` is a no-op. So `--samplers temperature` samples the same distribution, and with it `--backend-sampling` can run the whole chain on the GPU (greedy at temperature 0, `dist` at 1).

## Steps

1. **Smoke** (instrument check, not corpus): `wick-throughput.sh throughput/arms-smoke.txt smoke <commit>` runs the head `generate` on the set's first 4 prompts at `--parallel 1`, once per mode (`seq`, `n4`, `g-n3`), once with `--tokenize 0` then `count`, once with the short system message, and once with `--backend-sampling`. Each smoke must keep ≥ 3 records, with 0 markup answers, 0 errors and 16 answers returned. This is the first time the head client's integer sampling request reaches `llama-server`. **Done at `9d7c62536`: all six passed** (16 answers each, 0 markup, 0 errors, 0 tokenize errors; `count` gave 514 kept tokens for the `--tokenize 0` smoke). Two more smokes run inside the probe job, before A8a and A8b, with the trimmed chain on the CPU and on the GPU.
2. **Probe** (`throughput/arms-probe.txt`): one job, ten timed arms and A0's repeat. Each is a 300 s dispatch window from the start of the pilot's prompt set, and the prompts in flight then finish. A0 runs first and again last, so its spread is measured. Every arm uses `--max-tokens 1024`, seed 20261115 and the pilot's sampling values.

   | Arm | Server (`llama-server -ngl 99 -fa on --jinja` plus) | Client | Lever |
   |---|---|---|---|
   | A0, A0-repeat | `-np 32 -c 131072` | `seq`, 32 workers | the pilot's settings, re-measured with the head client |
   | A1 | `-np 64 -c 131072` | `seq`, 64 | more slots |
   | A2 | `-np 64 -c 131072 -b 4096 -ub 1024` | `seq`, 64 | more slots, larger batches |
   | A3 | `-np 128 -c 262144 -b 8192 -ub 2048` | `seq`, 128 | most slots, largest batches |
   | A4 | A0's | `seq`, 32, `--system-file short-system.txt` | a shorter system message |
   | A5 | A0's | `seq`, 32, `--tokenize 0`, then `count` | no per-answer `/tokenize` |
   | A6 | A0's | `n4`, 8 | four answers in one request (all sampled) |
   | A7 | A0's | `g-n3`, 11 | one greedy request, then `n` = 3 (the pilot's seed rule) |
   | A8a | A0's plus `--samplers temperature` | `seq`, 32 | the same distribution through a shorter CPU chain (diagnostic) |
   | A8b | A0's plus `--samplers temperature --backend-sampling` | `seq`, 32 | the same chain on the GPU (diagnostic) |

3. **Combined arm C**, a second job: every lever that wins (see Bar), on the fastest winning server setting, in one 300 s window, run twice.
4. **Optional V (vLLM):** the same client against vLLM's OpenAI server on the donor's released safetensors, if it installs in Claude's container within 30 minutes. vLLM is Python and is named in the receipt with its version. If it does not install or serve gpt-oss on `sm_89`, V is reported as not run.
5. **Storage:** each arm's `metrics.json`, `run.json`, `count.json`, server and GPU logs, and `summary.tsv` go to the HF store under `results/claude/d41-throughput-2026-11-15/`, fetched back and checked by sha256. Answer texts go there too, marked as probe output: not corpus, never training data.

## Bar

The probe is a measurement. Its bar decides which numbers may size production.
- **B1 smoke:** all six smokes pass, as in step 1.
- **B2 arm quality:** an arm counts only if its truncated plus markup answers are ≤ 5% of answers returned, its request errors are ≤ 1% of prompts dispatched, and its tokenize errors (in `metrics.json` or `count.json`) are 0.
- **B3 baseline:** A0's kept-answer rate lies within ±15% of the pilot's 220.7 tok/s, and A0 and A0-repeat lie within 10% of each other. This shows the head client and a 300 s window measure what the pilot measured.

**A lever wins** when its arm passes B2 and its kept-answer rate is at least 1.10 × A0's and above the larger of A0 and A0-repeat by more than their difference.

**Recorded, with no bar:** generated and kept tokens/s, kept tokens per 1,000 generated, request p50/p95, prompts dispatched, mean GPU utilization and peak memory.

**Reading map** (fixed before the run):
- **P:** B1–B3 hold. The REPORT sizes production as hours = 100,000,000 / (kept tok/s) / 3,600, from the best arm that passes B2. It gives two numbers: the best arm that keeps the corpus plan (the pilot's system message, one greedy and three sampled answers), and the best arm overall. A6 has no greedy answer and A4 changes the prompt, so either one changes the plan, and production's own sheet would have to adopt that change.
- **I (instrument):** B1 or B3 fails. The client or the window is repaired. No number sizes production until a new sheet's run passes.
- **Q:** an arm fails B2. That arm is reported and not used for sizing.
- **H-sampler (diagnostic):** if A8a or A8b wins by ≥ 1.5× with higher GPU utilization, the CPU sampler was the pilot's limit. The server log must show the backend sampler set for A8b's slots, and A8b's sampled answers must differ from A8a's on the smoke, or A8b is read as "not active" and not as a negative. Backend sampling is experimental in `b12000`, so production's sheet must check it before adopting it: on the same prompts at `--parallel 1`, its greedy answers must equal the CPU sampler's.

## Base artifact

- **Donor:** `openai/gpt-oss-20b` revision `6cee5e81ee83917806bbde320786a8fb61efebee`, the pilot's GGUF (`gpt-oss-20b.gguf`, its sha256 in the pilot's receipt).
- **Engine:** llama.cpp `b12000` (`68d20b884c3878217de79eade0a249efde96ab38`), the pilot's build, outside the repository (§3.4).
- **Instrument:** `docs/labs/d41-driver-math-pilot/instrument` at the commit each job's command line names; every `run.json` records it as `instrument_commit`. It is a client with no model code.
- **Machine:** wick, NVIDIA GeForce RTX 4090, UUID `GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`, in Claude's container (`local/claude/wick/wick.sh`).

## Data split

- Every prompt comes from the pilot's seeded, panel-guarded prompt set (sha256 `6a6630b2eb0078c3c795b04e005da47c810113cf41d15888c7632bb96d37cca0`), checked by the script before any arm. None comes from TRAIN, DEV or a panel.
- The probe's answers are probe output only. They are not part of `distill-d1`, not sealed and never used to train.

## Minimum test power

- No mechanism claim, so no seed requirement. One seed, 20261115.
- At the pilot's rate, a 300 s window holds about 3,000 requests, and each rate is reported with its request p50 and p95.
- Run-to-run spread is measured by A0 and A0-repeat. A lever must beat that spread to win.
- Every arm starts at the same prompt, so the arms see the same prompts in the same order, as far as each one gets.

## Cost

Wick is the owner's own GPU, so there is no rental cost (D86). The probe job takes about 11 × 5.5 minutes plus 7 server loads, about 70 minutes; the combined job about 15 minutes. No pod, and nothing near the §5.4 caps. `wick.sh gpu` is read before each job, and the job runs only if the GPU is free of other users' work.

## Stop rule

- Each arm stops dispatching after 300 s, and the prompts in flight finish.
- A server that does not start (for example, out of memory at `-np 128`) is recorded as `server-failed`, its arm is reported as not measured, and the job goes on to the next arm.
- Hard stop: if a job passes 2 hours, the lab stops it by its own PID and reports the arms finished.
- One pass per arm name. A failed arm is reported and is not rerun under this sheet.
