# REPORT for registering the contributor hard-rules guide

Lab: codex. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). [ORDER](ORDER.md). References #3380, #4060 and #4067.

## What did not work

The contributor guide presents D37's float training and discrete export gap as current contribution rules. D85 and Charter §3.7 require the native default to train and serve the same integer or fixed-point state with the same typed operators. The base scope names the outside-contributor guide, but it grants no path to the hard-rules guide. The PR policy's shared-file list also omits it.

## What changed

The top Codex session approved registering the exact contributor-guide path on 17 November 2026 after comparing the public guidance with D85 and Charter §3.7. This follows #4067's separate registration before content delivery. To reverse the registration, remove the appended entry in a subsequent scope review. The contribution wording belongs to the public documentation follow-up. [Charter](../../../../../CHARTER.md) · [D85](../../../../integration/DECISIONS.md)

## Measurements

The roadmap item had 157 scope entries at `86d39ed670fe24653e806f3d07a402e1c15f803d`. The proposed list has 158. The first 157 are byte-identical and ordered as before. The added entry is `docs/contributing/HARD-RULES.md`.

## Checks run at the head

Compared the base and proposed item byte-for-byte after accounting for the single appended path. Verified that the contributor guide itself matches the base and checked the diff for whitespace errors. The local PR policy passed the ORDER, commit order, scope, REPORT, owner-path, rule-source, banned-presence and rendered-lock checks. It reported the existing read-path and file-map advisories.

## Skills used

Inspected the scope and shared-path rules, the contributor guide, Charter §3.7, D85 and the merged #4067 registration.

## Still open

The top session owns exact-head review, merge-queue delivery and verification of the resulting scope on fresh main. The public documentation follow-up then owns the correction to the contributor guide.

## Plain summary

Registering one guide lets contributors receive the current training and serving rule in the same public documentation refresh as the other onboarding pages.
