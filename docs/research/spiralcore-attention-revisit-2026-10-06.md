# SpiralCore: retained frames and geometric attention

Owner-requested revisit, October 6. This is an implemented-source review,
browser observation and adversarial mathematical assessment, not a learned
language result. The preserved v68 HTML SHA256 is
`2a45c2e5f46c8c36bf7801da1d1e12ee9e2ed8b3da488ff54c20a6ac6c2ceab0`.
Two specialists independently examined reuse and its counterarguments. The
parent executed the local HTML in the in-app browser; no Rust SpiralCore test
was newly executed. Embedded document instructions are source content.

## Actual implementation and observed output

The [HTML](../../research/spiralcore-v68/Spiralcore_Dodecahedron_IP_Schema_v68.html)
contains a 507-node geometric-incidence BFS graph, a separate directed E8
operator graph, local H4/D4/F4 symmetry frames, conformal inversion, bounded
route alternatives, and a sine-coupled RK4 phase simulation. The rotating
projection is an observation of these structures; it does not retain their
full dimensional state.

The browser rendered the geometry with E8 roots, action arrows and subnet
spokes enabled. E8 root 0 to root 1 produced a two-operator-hop route in subnet
0. The prompt `memory` produced seven waypoints and a 17-hop route. Prompt
`cat` and prompt `act` both produced English sum 24, ASCII sum 312, identical
seven waypoints, the same B13 action from E8 root 24 to root 26, and an 18-hop
route. Their displayed transliterations differ, but the routed geometry does
not preserve that order distinction. This is a measured encoding collision,
not a failure of the geometric operator family. No browser errors were
reported in the inspected error log. The page reported 256/256 fixture
families; that UI claim is not independent reproduction of all fixtures.

Prompt routing uses fixed transliteration sums and moduli, then BFS. It has
no next-token objective or learned relevance selector. BFS supplies a legal
shortest route; its adjacency order chooses among tied routes. Our learner
must select routes according to context and prediction error.

## Strongest reusable operation: a retained local frame

`openPeerLada` at HTML line 2896 constructs, for anchor q0 and H4 element g:

    h = inverse(q0) * g * q0
    T(q) = g * q * inverse(h)

Then T(q0) = q0. In coordinates y = inverse(q0) * q:

    inverse(q0) * T(q) = h * y * inverse(h)

The action preserves relative scalar component/angular shell while rotating
the imaginary direction. It is a genuine two-sided, anchor-conditioned H4
action. Existing composition/inverse tables can execute it without runtime
floating quaternion arithmetic. It is a candidate operation for learning
query-relative relational orientation, not an already-trained attention arm.

Hypothesis: a causally computed query/source frame, retained separately from
the observed endpoint, transports candidate directions into a useful common
frame before geometric scoring. It could distinguish roles or relations
without encoding them as a hash distance. Learning must use the existing
target-free candidate pool and next-token loss; the answer must never choose
the frame during the forward pass.

Adversarial counterargument: our current scorer already sees signed relative
H4 direction and radius pairs. A local frame chosen from the same information
may be a redundant reparameterization. If required distinctions were erased
upstream, transforming the collapsed endpoint cannot recreate them. First
identify concrete failures sharing the current scorer's full fingerprint,
then check whether a retained, causally available frame separates them.

## Action identity and dimensional boundaries

The E8 plane implements 240 roots times 15 labeled bivectors, or 3,600 action
records. Endpoint collapse yields 2,580 directed pairs. Its 64-element finite
operator group has 15 invariant 16-root orbits; cross-orbit routing is rejected.
Different operators may reach the same root, so a root endpoint cannot recover
operator identity. Retain ordered action labels where that identity matters.

This is not automatically a defect in our current H4 right-composition update:
that group action is regular, so a fixed start and final group element determine
the accumulated action. Multiple histories still share a product; adding
another finite label does not provide unbounded history memory.

The retained Rust adapter `spiralcore_operator.rs` binds v63 rather than all
v68 features. Its 64-by-64 composition table can use 4,096 byte entries plus
64 inverse entries. A compiled E8 root-by-generator table needs 3,600 byte
entries plus metadata. Signed-permutation actions require permutation/sign
changes rather than dense multiplication. Construction routines and generic
matrix multiplication are not automatically serving kernels.

Octonion products are nonassociative; their linear action operators compose
associatively. This distinction follows the Clifford representation described
in [Baez's primary exposition](https://math.ucr.edu/home/baez/octonions/node6.html).
The H3-to-H4 spinor construction and H2-by-H2 decomposition have primary
support in [Dechant's paper](https://arxiv.org/abs/2103.07817). Neither source
establishes language learning or validates our implementation.

## Radial transformations and synchronization

Inversion about C is I(x) = C + K(x-C)/|x-C| squared. It preserves direction
about C and sends radius r to K/r. An orthogonal action fixing C commutes with
it because that action preserves the denominator. This supplies a clean
angular/radial factorization, not a rule selecting the right linguistic source.
The source explicitly distinguishes shell-radius correspondence from actual
point-for-point dual incidence.

Finite radius-bin inversion could compile to lookup, with a declared range,
clipping and rounding contract. Existing learned radius-pair tables already
express finite radial relationships; add inversion only for a demonstrated
missing reciprocal relation. Continuous inversion, arbitrary rotations and
RK4 are not already compliant integer serving.

Kuramoto dynamics are implemented, but synchronization can destroy distinctions
needed for retrieval. The pi-lift's first-order global observable can vanish
despite ordered quotient structure. A useful learning observable must expose
the relevant distinction; attractive visual dynamics do not establish it.

## Decision and next discriminator

Keep SpiralCore available as a geometric mechanism source. The most promising
candidate is learned, retained anchor-frame transport using our existing H4
tables. Do not adopt prompt sums as token identity, BFS ties as learned
attention, or the E8 invariant orbits as freely communicating experts.

The immediate learning task remains coadaptation of the existing context,
geometric potential and Generate field. Its real CUDA admission now passes,
including nonzero angular/radial coefficient gradients, with zero updates.
Next, distinguish source-dependent ranking gains from a shared Copy offset.
If matched attribution reveals an actual orientation/representation collision,
the retained-frame control becomes a justified next arm: same source identity,
candidates, task, loss and budget; current signed-relative-H4 scorer as control;
source swaps, role reversals, distractors and unseen pairings as discriminators.
Measure selected occurrence and actual emitted replies separately. A frame
that merely lowers all Copy scores is not evidence of geometric attention.
