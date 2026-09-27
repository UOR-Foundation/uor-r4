# Exact integer classification of signed H4 roots

This source work prepares the classification step of native geometric attention.
It does not integrate a new reader or qualify a learned artifact. OpenCode owns
the finite signed-group reader and its language comparison. The fourth lab’s
active dialogue code-choice experiment and its fixed endpoints remain separate.
Compilation, focused checks and emitted instruction inspection are pending.

## Chosen mathematical contract

`uor_r4_integer::h4_classifier::signed_h4_code_i32` accepts four signed i32
coordinates with a common positive scale. It selects the root with maximum
exact inner product in the 120-element historical H4 enumeration, retaining
antipodes. Ties between nonzero-query scores choose the lowest historical index.
The all-zero vector explicitly maps to identity index 1, matching the geometric
reader’s zero convention rather than ordinary all-tied argmax index 0.

This is an exact mathematical-root classifier on the declared quantized vector.
It does not promise identical decisions to normalized F32 vectors, rounded F32
root coordinates, or the unrelated three-coordinate S2/Q30 classifier. Those
numerical interfaces may disagree near boundaries. With a common Q8 input scale
and the current reader epsilon of 1e-6, every nonzero quantized lane lies above
the zero threshold. A different scale/epsilon requires an explicit contract.

The immutable `H4_ROOT_COEFFICIENTS` table stores 960 signed bytes. Its order and
coefficients follow `native_geometric::learner::prefix_artifact::historical_roots`,
introduced at `d15360527f7c69ac8b83eef0bbd5839b87c26f02`. A coordinate is
`(a + b*phi)/2`; the ignored factor one-half is shared by every candidate. The
integer crate gains no core dependency, floating constructor or heap allocation.
A separate offline training-crate test compares all coefficients against the
existing exact donor. Sorted closure IDs are a different ordering and cannot
be substituted without the explicit bijection.

## Exact family reduction

The roots partition into eight axis roots, sixteen signed half-roots and twelve
blocks of eight sign choices for the even permutations of
`(0, 1, phi, phi-1)/2`. Each nonzero magnitude is positive, so a family's best
sign matches its assigned query coordinate. A zero coordinate chooses the
negative sign, preserving the smallest historical ID within that family.

The classifier retains one winner from each family: the earliest axis with
maximum widened absolute coordinate, the best signed half-root, and one signed
root for each of the twelve golden permutations. A shared permutation constant
maps output axis to base coordinate. Candidate selection places each query sign
into its corresponding base sign bit, rather than mistakenly treating that
permutation as its inverse. The sign bits are `s1=4, s2=2, s3=1`.

These fourteen IDs occur in ascending historical-family blocks. Comparing only
their scores with strict-greater updates preserves the global lowest-ID tie.
The all-zero input still bypasses ordinary argmax and returns identity 1. The
existing score assembly and exact comparator are unchanged, as is the independent
offline oracle that scans all 120 roots.

This reduces root-score evaluations from **120 to 14** and exact score comparisons
from **119 to 13**, with three additional integer magnitude comparisons to choose
the axis. It uses no candidate approximation or learned selection and retains
the full signed-i32 domain by widening before taking absolute values. These are
structural operation counts, not a measured speed, energy or language result.

## Comparison and bounds

Each dot score is represented as `A + B*phi`, omitting the common half. The
small coefficients 0, ±1 and ±2 require only addition, subtraction and doubling.
For a score difference, let `P = 2*A + B`; twice that difference is
`P + B*sqrt(5)`. Zero and same-sign cases determine the sign immediately. When
signs oppose, comparing `P²` and `5*B²` determines which signed magnitude wins.
The existing checked shift/add product computes the squares; five times a square
is a two-bit shift plus the square. No irrational approximation is needed.

For input magnitude at most M, the donor coefficients give per-root bounds
`|A| <= 4M` and `|B| <= 2M`. Differences therefore have `|A| <= 8M`, `|B| <= 4M`
and conservatively `|P| <= 20M`. At M = 2^31, both required squared quantities
are below 2^71, inside u128. Inputs must be widened before negation or doubling,
including i32::MIN. The equality case of nonzero integer `P² = 5*B²` is
impossible; exact score ties are represented by both difference coefficients
being zero. Strict-greater selection preserves the earlier historical index.

The production source is intended to use only integer shifts, additions,
subtractions, comparisons and table reads. A source property does not establish
the emitted instructions of a particular optimized build. Later inspection must
cover the actual classifier and arithmetic callees, without treating an absent
inline symbol as inspected. No performance or energy measurement is supplied.

## Integration boundary and next evidence

Focused arithmetic checks cover zero, signed identities, ties, extreme i32
inputs and an independent native-i128 reference in offline tests. The separate
donor-order check prevents an internally consistent classifier from silently
using incompatible group IDs. These are mathematical/interface checks, not
language tests or a broader proof campaign.

If the learned reader earns continuation, integration must bind the actual Q/K
quantization, root order, inverse/product tables and legal learned score-table
compilation. It must then observe float-to-integer decision changes and actual
complete replies. This helper alone does not select MLP weight precision,
modify candidate admission, infer semantics from hash addresses, remove dense
vocabulary access, or establish geometric advantage. It also leaves the paired
H4/icosian state and its retained orientation untouched.

Source-only ownership and scope are recorded in the [shared work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5857712244).
References #973 and #820. Cargo, model and instruction checks wait for the active
fit and its declared endpoints; source work/review overlaps are charged once to
the existing cumulative ledger.
