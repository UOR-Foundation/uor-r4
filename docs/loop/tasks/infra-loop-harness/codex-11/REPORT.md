# REPORT for three permanent branches

[codex] Infrastructure and documentation delivery. [ORDER](ORDER.md). References #2904 and the owner's November 17 branch decision.

## What did not work

The former publishing workflow recreated a branch the owner asked to retire. Three independent consumers also encoded the old list: cleanup, delivery diagnostics and the Studio publisher. Changing only GitHub’s branch names would have made the next documentation change recreate the preview.

## What changed

The owner designated main, gh-pages and archive. I applied that decision to the charter clause and recorded the change in the charter log. Main carries research, gh-pages carries the public book and Studio, and archive preserves original commits. Short-lived delivery branches follow the existing pull-request lifecycle.

A push to main builds and checks the book with a read-only token. A manual dispatch from main selecting gh-pages passes the checked book to a separate publisher. Its push targets only gh-pages. The publisher verifies that the Studio tree is intact. Branch cleanup protects all three permanent names, and the Studio helper accepts only the live destination.

## Checks run at the head

The local workflow and Studio destination tests exercised a real bare Git remote, verified the live book updates, protected the Studio tree and refused conflicting book paths. The complete Python workflow/Studio regression set passed 13 checks. The pinned book build checked 106 pages with zero broken links. Rust formatting and shell syntax passed.

## Skills used

Git object inspection, the repository harness and independent exact-head review.

## Still open

After protected merge, dispatch the production workflow and verify the live site. Preserve the preview’s final commit in the remote archive before retiring its branch name. Check the remote branches again after the prior publisher has finished.

## Plain summary

Researchers can publish the book beside Studio without keeping a separate preview branch. Original experiment histories remain available through the archive branch.
