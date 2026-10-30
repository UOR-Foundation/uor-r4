# A memory key a reworded question can reach (claude-218, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md). Order: [claude-218](../../../loop/tasks/one-model-end-to-end/claude-218/ORDER.md). The redirect from the rollout hinge was made by the lab's decision review, logged on #3407 (6526603691).

## Step 0: what each exact key could name, on the laptop

- **The method:** `train-in-space --opening-bound 1 --panel p-s=DIR`, at `8ea674541`; re-run with the panel's anchor check at the head after FIT round 1, its bounds byte-identical. It counts over the corpus: for each key, TRAIN's commonest reply opening, scored on the openings of DEV and of the panel p-s. There is no model compute.
  - p-s is the held-out-phrasing panel of claude-217 (seed 2091; tokens sha256 `62099a48…10f1`, mask `9053e117…f11a`).
  - It is now a development panel: it chooses keys and decides nothing.
- **The output:** [opening-bound.jsonl](opening-bound.jsonl), sha256 `f22316c7db876e8eba565d064de997b40367f100828395611177f26eefa77d05`.
- **Fires** means TRAIN holds the key. **Precision** is the share right among those.

| Key | DEV fires | DEV precision | DEV right | p-s fires | p-s precision | p-s right |
|---|---|---|---|---|---|---|
| none (TRAIN's commonest opening) | 100% | 13.1% | 13.1% | 100% | 15.3% | 15.3% |
| `first-4` | 100% | 18.5% | 18.5% | 99.6% | 15.1% | 15.2% |
| `first-6` | 100% | 37.5% | 37.5% | 62.4% | 20.6% | 17.9% |
| **`first-8`, the memory's key** | **97.4%** | **50.3%** | **49.5%** | **17.0%** | **19.3%** | **15.3%** |
| `set`, the prompt's distinct ids | 63.7% | 61.8% | 47.1% | 0.4% | 80% (4 of 5) | 15.6% |
| `set-first-8` | 97.4% | 50.3% | 49.5% | 17.8% | 20.9% | 15.2% |
| **`reply-template`, the oracle U** | 76.6% | 100% | 80.8% | **69.1%** | 100% | **76.1%** |

- **Totals:** DEV has 1,226 openings and p-s 1,400.
- **The memory's key:** the turn anchor keeps `first-8`'s tokens at every opening (1,226 of 1,226 on DEV, 1,400 of 1,400 on p-s, 212,968 of 212,968 on TRAIN), so the laptop's exact key is the memory's served one.
- **The oracle U** is keyed by the reference reply itself, with its values masked (K = 1,024). So its precision is 100% by construction. It is the ceiling for a key that knew which question was asked, not a key that can be built.

## What step 0 shows

- **The memory's key rarely fires on new wording, and is mostly wrong when it does.**
  - On DEV, `first-8` fires on 97.4% of openings and names 49.5% right. That matches the model's own teacher-forced openings with the memory on (49.1–49.3%, claude-217).
  - On p-s it fires on 17.0% and names only 19.3% of those right.
  - The commonest TRAIN keys are the question's opening words ("User: Note these down:", "User: Remember these:", "User: Tell me what").
- **Exact word keys cannot reach a reworded question.** The whole prompt as an order-free key fires on 0.4% of p-s, so the reworded questions use different words, not just a different order. The first 8 as an order-free key does no better than `first-8`.
- **The headroom is large.** 69.1% of p-s openings belong to a reply template TRAIN holds, and the oracle would name 76.1% right. The best prompt key names about 15–18%, and the model about 14–19%. So most new phrasings ask questions TRAIN knows the answer to, and no key over the words finds them.
- **By the ORDER's decision table:** there is headroom ("no headroom" does not hold), and no id key reaches it. So the GPU key probe (step 2) runs. It tests whether the trunk's own opening point puts a reworded question beside its known one: by its Hamming sign cell, or by the nearest TRAIN opening point under L1. A forced-opening arm sits beside it.
