import WasmGemmGnaf.GNAF.Compile
import WasmGemmGnaf.Wasm.Run
import WasmGemmGnaf.Artifact.Emit

set_option autoImplicit false
set_option maxRecDepth 8000

/-!
# GNAF: what the compiler is proved to guarantee about execution (SPEC §11.4)

This file collects the refinement facts that the *current* state of the Wasm
layer actually supports, and states precisely which SPEC §11.4 obligations are
therefore **not** discharged anywhere in this repository.

## What is proved here

`Wasm/Config.lean` fixes initialization: `Wasm.initialConfig` validates the
module, allocates the store, locates the exported `gemm`, and builds the
harness control frame.  Because `GNAF.compile_validates` proves the emitted
module passes release validation, and because the emitted module's shape is
fixed, initialization of a compiled module can be computed exactly:

* `GNAF.compile_initialConfig` — the initial configuration of a compiled module
  is a specific configuration, given in closed form;
* `GNAF.compile_no_instantiation_fault` — no compiled module ever produces an
  instantiation fault;
* `GNAF.compile_gemmBody` — the harness runs exactly the compiled body;
* `GNAF.compile_runInvariant` — the initial configuration satisfies
  `Wasm.RunInvariant`, so `Wasm.returned_has_entry_store` applies to every run
  of a compiled module: a normally returning run of a compiled artifact has
  crossed the harness boundary and carries exactly one entry store.

The last section closes the anti-vacuity gap of SPEC §8.3: `GNAF.gemmWitness` is
a concrete `1×1×1` modular-`u32` GEMM plan proved to lie inside
`Plan.inReleasedSubset`, so `GNAF.code_no_unreachable` and the other in-subset
results have an exhibited inhabitant rather than an unwitnessed domain.  The
witness is not only in the subset: `GNAF.gemmWitness_writes_C` proves that
running it deposits the modular product into the declared `C` region, so the
inhabitant is a plan that genuinely computes a GEMM rather than a plan that
merely type checks.

## What is deliberately omitted, and why

* **`compile_refines`** (SPEC §11.4).  Omitted.  It would relate
  `Wasm.FiniteExecution` of the compiled module to `GNAF.Accepts`.  The GNAF
  machine of `GNAF/Semantics.lean` computes over unbounded `Nat` cells while
  the Wasm machine computes over `i32`, so the statement is *false* as written
  without an explicit boundedness precondition and an explicit representation
  relation between a `GNAF.Machine` and a `Wasm.Config`; and proving even the
  bounded version needs a simulation argument for every plan constructor, which
  is not established here.  Weakening the statement — for instance quantifying
  only over runs that happen to exist, or replacing `Accepts` by an
  observational projection — would produce a theorem that looks like a
  refinement result and is not one, so nothing of the kind is stated.
* **`compile_cost_exact`** (SPEC §11.4).  Omitted.  `Wasm/Costed.lean` does
  supply a costed reduction relation (`Wasm.CostedReduces`), so the statement
  could now be *written*; but its content is that the measured cost of a costed
  run of the compiled module equals `GNAF.certifiedCost` of the plan, and that
  is strictly stronger than the refinement theorem above, which is not proved.
  Stating it with a hypothesis nobody can discharge, or proving a variant whose
  cost function is read off the compiler rather than from the plan, would both
  be worse than saying nothing: neither would be the SPEC obligation.
* **A step bound for the emitted module.**  `GNAF.compile_resources` in
  `GNAF/Compile.lean` bounds the emitted module's *code size* by the plan's
  declared static cost.  A bound on the number of Wasm reduction steps would be
  a consequence of the missing refinement theorem and is not claimed.

Every declaration in this file is proved.  Nothing is assumed.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

variable {s t : Sig}

/-- The store the emitted module allocates: a fresh zeroed memory of the
declared page count and no globals. -/
def compiledStore (e : CompileEnv) : Wasm.Store :=
  { memory := Wasm.Memory.alloc e.pages (some e.pages), globals := [] }

/-- The harness control frame the emitted module installs. -/
def compiledHarness (e : CompileEnv) (body : List Wasm.Instr)
    (raw : Wasm.RawInvocation) : Wasm.Harness :=
  { gemmBody := body
    gemmNumLocals := e.declaredLocals
    rawPtr := raw.ptr
    rawBytes := raw.bytes
    args := [UInt32.ofNat raw.ptr, UInt32.ofNat raw.bytes.length] }

/-- The initial configuration of an emitted module. -/
def compiledInitialConfig (e : CompileEnv) (body : List Wasm.Instr)
    (raw : Wasm.RawInvocation) : Wasm.Config :=
  { store := compiledStore e
    harness := compiledHarness e body raw
    locals := [], stack := [], code := [], ctrl := []
    phase := .beforeEntry, entry? := none, status := .running }

theorem alloc_moduleOf (e : CompileEnv) (b : List Wasm.Instr) :
    Wasm.Store.alloc (moduleOf e b) = some (compiledStore e) := rfl

/-- Initialization of an emitted module succeeds and produces exactly
`compiledInitialConfig`. -/
theorem initialConfig_moduleOf (e : CompileEnv) (b : List Wasm.Instr)
    (hv : Wasm.validate (moduleOf e b) = true) (raw : Wasm.RawInvocation) :
    Wasm.initialConfig (moduleOf e b) raw = .ok (compiledInitialConfig e b raw) := by
  unfold Wasm.initialConfig
  rw [if_neg (by rw [hv]; exact Bool.noConfusion)]
  rw [alloc_moduleOf, gemmIndex_gemmExports]
  simp only [CompiledModule.funcs_eq, List.getElem?_cons_zero, CompiledModule.start,
    Wasm.Func.code, Wasm.Expr.toList_ofList, List.length_replicate]
  rfl

/-- **The compiled module always instantiates.**  Initialization never faults,
and the resulting configuration is given in closed form. -/
theorem compile_initialConfig (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    Wasm.initialConfig (compile c) raw =
      .ok (compiledInitialConfig (envOf s c.plan)
            (bodyCode (envOf s c.plan) s.scratch c.plan) raw) :=
  initialConfig_moduleOf _ _ (compile_validates c) raw

/-- **No compiled module ever produces an instantiation fault.** -/
theorem compile_no_instantiation_fault (c : CheckedPlan s t)
    (raw : Wasm.RawInvocation) (f : Wasm.InstantiationFault) :
    Wasm.initialConfig (compile c) raw ≠ .error f := by
  rw [compile_initialConfig]
  simp

/-- The harness of a compiled module runs exactly the compiled plan body. -/
theorem compile_gemmBody (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw).harness.gemmBody =
      bodyCode (envOf s c.plan) s.scratch c.plan := rfl

/-- The compiled module declares exactly the locals the translation uses. -/
theorem compile_gemmNumLocals (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw).harness.gemmNumLocals =
      (envOf s c.plan).declaredLocals := rfl

/-- The compiled module starts before the harness boundary, with no entry
snapshot and an empty control and operand stack. -/
theorem compile_initial_shape (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    ∀ cfg : Wasm.Config, Wasm.initialConfig (compile c) raw = .ok cfg →
      cfg.status = .running ∧ cfg.phase = .beforeEntry ∧ cfg.entry? = none ∧
        cfg.ctrl = [] ∧ cfg.stack = [] := by
  intro cfg h
  exact Wasm.initialConfig_shape h

/-- The initial configuration of a compiled module satisfies the harness run
invariant of SPEC §7.4. -/
theorem compile_runInvariant (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    Wasm.RunInvariant (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw) :=
  Wasm.runInvariant_of_beforeEntry rfl rfl

/-- **Every normally returning run of a compiled artifact has crossed the
harness boundary**, so its observation carries exactly one entry store.  This is
SPEC §7.4's presence invariant, instantiated at the compiled module. -/
theorem compile_returned_has_entry_store (c : CheckedPlan s t)
    (raw : Wasm.RawInvocation) {final : Wasm.Config} {tr : List Wasm.Event}
    {v : UInt32}
    (hred : Wasm.Reduces (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw) tr final)
    (hfin : final.status = Wasm.Status.returned v) :
    final.entry?.isSome = true :=
  Wasm.returned_has_entry_store hred (compile_runInvariant c raw) hfin

/-- A finite execution of a compiled artifact is a `*BeforeEntry` observation
exactly when it never crossed the harness boundary. -/
theorem compile_beforeEntry_iff (c : CheckedPlan s t) (raw : Wasm.RawInvocation)
    {o : Wasm.ExecutionObservation}
    (h : Wasm.FiniteExecution (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw) o) :
    o.BeforeEntry ↔ Wasm.Event.enterGemm ∉ o.trace :=
  Wasm.beforeEntry_iff_not_mem_enterGemm h rfl

/-! ## The first reduction of a compiled artifact -/

/-- The store after the harness has installed the raw invocation bytes. -/
def installedStore (e : CompileEnv) (raw : Wasm.RawInvocation) : Wasm.Store :=
  { compiledStore e with
    memory := { (compiledStore e).memory with
                data := Wasm.setBytes (compiledStore e).memory.data raw.ptr raw.bytes } }

theorem storeBytes_compiledStore (e : CompileEnv) (raw : Wasm.RawInvocation)
    (hfit : raw.ptr + raw.bytes.length ≤ e.pages * Wasm.pageSize) :
    (compiledStore e).storeBytes raw.ptr raw.bytes = some (installedStore e raw) := by
  have hsize : (compiledStore e).memory.size = e.pages * Wasm.pageSize :=
    Wasm.Memory.alloc_size _ _
  unfold Wasm.Store.storeBytes Wasm.Memory.storeBytes
  rw [if_pos (by rw [hsize]; exact hfit)]
  rfl

/-- **The compiled artifact reaches its compiled body in one reduction.**
Whenever the raw invocation fits in the declared memory, the harness step of
SPEC §7.2 installs the raw bytes, crosses the entry boundary, and hands control
to exactly the instruction sequence the compiler emitted. -/
theorem compile_enters_gemm (e : CompileEnv) (body : List Wasm.Instr)
    (raw : Wasm.RawInvocation)
    (hfit : raw.ptr + raw.bytes.length ≤ e.pages * Wasm.pageSize) :
    Wasm.Step (compiledInitialConfig e body raw) Wasm.Event.enterGemm
      { compiledInitialConfig e body raw with
        store := installedStore e raw
        locals := [UInt32.ofNat raw.ptr, UInt32.ofNat raw.bytes.length] ++
          List.replicate e.declaredLocals 0
        stack := [], code := body, ctrl := []
        phase := .afterEntry
        entry? := some (installedStore e raw).observable } := by
  refine (Wasm.mem_successors_iff_step _ _ _).mp ?_
  show _ ∈ Wasm.successorsAtEnd (compiledInitialConfig e body raw)
  unfold Wasm.successorsAtEnd
  simp only [compiledInitialConfig, compiledHarness]
  rw [storeBytes_compiledStore e raw hfit]
  simp

/-- Consequently the compiled body is the code of a configuration reachable
from the initial configuration of the compiled module. -/
theorem compile_body_reachable (c : CheckedPlan s t) (raw : Wasm.RawInvocation)
    (hfit : raw.ptr + raw.bytes.length ≤
      (envOf s c.plan).pages * Wasm.pageSize) :
    ∃ cfg : Wasm.Config,
      Wasm.Reduces (compiledInitialConfig (envOf s c.plan)
        (bodyCode (envOf s c.plan) s.scratch c.plan) raw) [Wasm.Event.enterGemm] cfg ∧
      cfg.code = bodyCode (envOf s c.plan) s.scratch c.plan :=
  ⟨_, Wasm.Reduces.single (compile_enters_gemm _ _ raw hfit), rfl⟩

/-! ## The artifact round trip -/

/-- The emitted artifact of a compiled plan decodes back to exactly the
compiled module. -/
theorem compile_decode_emit (c : CheckedPlan s t) :
    Wasm.decode (Artifact.emit (compile c)) = .ok (compile c) :=
  Artifact.decode_emit _

/-- Compiled modules with distinct emitted bytes are distinct modules, and
conversely: the artifact identifies the module it came from. -/
theorem compile_emit_injective {c c' : CheckedPlan s t}
    (h : Artifact.emit (compile c) = Artifact.emit (compile c')) :
    compile c = compile c' :=
  Artifact.emit_injective h

/-- The emitted artifact of a compiled plan decodes to a module that passes
release validation. -/
theorem compile_emit_decodes_valid (c : CheckedPlan s t) :
    ∃ m : Wasm.Module,
      Wasm.decode (Artifact.emit (compile c)) = .ok m ∧ Wasm.validate m = true :=
  ⟨compile c, compile_decode_emit c, compile_validates c⟩

/-! ## The anti-vacuity GEMM witness (SPEC §8.3)

`compile_validates` holds for every `CheckedPlan`, and `code_no_unreachable`
holds for every plan inside `Plan.inReleasedSubset`.  Both are universally
quantified, so on their own neither exhibits a single plan that is *both*
inside the subset and a GEMM.  This section supplies one.

`gemmWitness` is a complete released-profile plan: it classifies the raw ABI
header, dispatches on the declared layout class, runs the blocked traversal and
the `i`/`j` loop nest, accumulates the `k` reduction under the modular-`u32`
arithmetic contract of SPEC §8.2, sets the released status word, and constructs
the output from the declared `C` region.  It is checked at a *scalar* interface
(`Sig.lanes = 0`), so `hasType_no_vector` applies to it as well.

The GNAF machine of `GNAF/Semantics.lean` is cell addressed: `Machine.byteAt i`
is memory cell `i`, and `reduce` reads one whole element per cell.  The witness
header therefore lays SPEC §8.3's header table out cell-wise, and the three
matrix elements occupy the three cells after it.  No claim is made here that
this cell image equals the byte image produced by `Gemm/ABI.lean`'s
`encodeHeader`; that is a byte-level representation statement about a different
machine model and it is not proved anywhere, so it is not asserted.

### Where the result goes

The kernel deposits its accumulator into the declared `C` region with
`Plan.storeReg`, the constructor that moves a register into memory; its
load-after-store law is `GNAF.storeReg_reads_back` and its frame law is
`GNAF.storeReg_outside`.  `gemmWitness_eval_acc` proves the product is computed
exactly, `gemmWitness_eval_mem` gives the memory the store leaves — the
four-cell little-endian image of that product at `C` — and
`gemmWitness_writes_C` reads that image back: **after evaluation the `C` region
holds `alpha · A · B + beta · C` modulo `2 ^ 32`**, which is what makes this
plan a GEMM and not merely a well-typed plan.  `gemmWitness_eval_mem_outside`
is the matching frame statement: every cell outside `C`'s four keeps its entry
value.  The store is a *cell-wise little-endian* image, exactly as
`Machine.u16At` reads cells; no claim is made here that it coincides with the
byte image of `Gemm/ABI.lean`, for the same reason the header image is not
claimed to.

Three things `gemmWitness_writes_C` does **not** say, stated precisely because
the theorem is the first link of SPEC §13 Phase B and its strength matters:

* **It is the `alpha = 1`, `beta = 0` instance.**  `gemmWitnessAlpha` and
  `gemmWitnessBeta` are the scalars read out of the witness header's own
  `alpha` and `beta` fields, and that header declares `alpha = 1` and
  `beta = 0` (`gemmWitnessAlpha_eq`, `gemmWitnessBeta_eq`).  The plan contains
  no scaling node at all, so the theorem holds *because* those two literals are
  `1` and `0`; it does not generalize to a descriptor declaring other scalars,
  and no such generalization is claimed anywhere.
* **It is a statement about `Plan.eval`, not about the emitted Wasm.**
  Relating the two is `compile_refines`, which is omitted above for the reasons
  given there.
* **The witness descriptor's cell image still has overlapping regions.**  The
  header declares `A`, `B` and `C` element lengths of four bytes each while the
  cell model gives `A` and `B` one cell apiece, and it declares the
  status-detail record at cell `259`, which `C`'s four cells run into.
  `Machine.classify` checks the header literals, the declared kind/mode triple
  and the invocation extent — it does not check region disjointness — so
  `gemmWitnessMachine_classify` is not evidence that these regions are
  disjoint, and no disjointness is claimed.  The cells `C` overlaps are written
  by no other node of this plan and read by none, so the theorems above are
  unaffected; a descriptor-level `disjoint` obligation would have to be
  discharged against `Gemm/Descriptor.lean`, and it is not discharged here.

What the witness *does* now publish is the whole result: `C` is declared as its
four stored cells, so `buildOutput` emits the complete little-endian image of
the product (`gemmWitness_eval_out`) rather than one byte of it. -/

/-- The released modular arithmetic contract with a `u32` accumulator: SPEC
§8.2 compatibility row 0 at stored kind `u32`, and the one contract shape the
released `i32` profile evaluates exactly. -/
def gemmWitnessContract : ArithmeticContract :=
  { mode := .modular, stored := .u32, accumulator := .u32 }

theorem gemmWitnessContract_compatible : gemmWitnessContract.compatibleB = true := by
  decide

theorem gemmWitnessContract_released : gemmWitnessContract.releasedB = true := by
  decide

/-- The accumulator modulus of the witness contract is exactly `2 ^ 32`. -/
theorem gemmWitnessContract_accModulus : gemmWitnessContract.accModulus = 4294967296 :=
  ArithmeticContract.accModulus_of_releasedB gemmWitnessContract_released

/-- The 256 header cells of the witness descriptor, in exactly SPEC §8.3's
header order: a `1×1×1`, batch `1`, stored-`u32`, accumulator-`u32`, `modular`,
untransposed, `disjoint`, row-major GEMM with `alpha = 1` and `beta = 0`. -/
def gemmWitnessHeader : List Nat :=
  -- `0..3` magic `WGNG`, `4..5` version 1, `6..7` header size 256
  [87, 71, 78, 71, 1, 0, 0, 1] ++
  -- `8..11` A, B, C and accumulator kind tags (`u32` = 5)
  [5, 5, 5, 5] ++
  -- `12` mode `modular`, `13` transpose bits, `14` alias tag `disjoint`, `15` zero
  [0, 0, 0, 0] ++
  -- `16..47` m, n, k, batch
  [1, 0, 0, 0, 0, 0, 0, 0] ++ [1, 0, 0, 0, 0, 0, 0, 0] ++
  [1, 0, 0, 0, 0, 0, 0, 0] ++ [1, 0, 0, 0, 0, 0, 0, 0] ++
  -- `48..87` A view: offset 256, byte length 4, row/column/batch stride 4
  [0, 1, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++
  -- `88..127` B view: offset 257, same packed strides
  [1, 1, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++
  -- `128..167` C view: offset 258, same packed strides
  [2, 1, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++
  -- `168..183` alpha = 1, `184..199` beta = 0
  [1, 0, 0, 0, 0, 0, 0, 0] ++ List.replicate 8 0 ++ List.replicate 16 0 ++
  -- `200..215` scratch offset 0 and scratch length 0
  List.replicate 16 0 ++
  -- `216..231` status-detail offset 259 and status-detail length 32
  [3, 1, 0, 0, 0, 0, 0, 0] ++ [32, 0, 0, 0, 0, 0, 0, 0] ++
  -- `232..255` reserved
  List.replicate 24 0

theorem gemmWitnessHeader_length : gemmWitnessHeader.length = 256 := rfl

/-- The witness machine memory: the 256 header cells, then the `A`, `B` and `C`
elements, then the 32-cell status-detail record the header declares. -/
def gemmWitnessMem (a b c : Nat) : List Nat :=
  gemmWitnessHeader ++ [a, b, c] ++ List.replicate 32 0

theorem gemmWitnessMem_length (a b c : Nat) : (gemmWitnessMem a b c).length = 291 := rfl

/-- The declared `A` region: one element, in the cell after the header. -/
def gemmWitnessA : RegionRef := { base := 256, count := 1 }

/-- The declared `B` region. -/
def gemmWitnessB : RegionRef := { base := 257, count := 1 }

/-- The declared `C` region — the output the plan publishes.  It is the four
cells the kernel's four-cell little-endian store writes, which is exactly the
four-byte `C` element length the witness header declares, so the store fits the
region it declares and `buildOutput` publishes the whole stored result. -/
def gemmWitnessC : RegionRef := { base := 258, count := 4 }

/-- The witness input configuration.  The entry status is deliberately not
`success`, so that `gemmWitness_eval_status` has content. -/
def gemmWitnessMachine (a b c : Nat) : Machine :=
  { mem := gemmWitnessMem a b c
    scratch := []
    regs := [0, 0, 0, 0]
    vregs := []
    tables := []
    status := Status.arithmeticException.code
    out := [] }

/-- The witness interface: four scalar registers (tile index, `i`, `j`,
accumulator), no vector lane at all, no scratch and no table. -/
def gemmWitnessSig : Sig :=
  { inputType := .bytes 291
    outputType := .unit
    regs := 4, vregs := 0, lanes := 0, scratch := 0, tables := 0, mem := 291
    statusSet := false }

/-- The interface the witness leaves: a status has been constructed and the
output is the four-cell `C` region. -/
def gemmWitnessOutSig : Sig :=
  { gemmWitnessSig with outputType := .bytes 4, statusSet := true }

/-- The `1×1×1` micro-kernel: a blocked traversal in the declared order, the
`i` and `j` axes of the loop nest with their packed index maps, and the `k`
reduction under the modular-`u32` contract, accumulating into register 3. -/
def gemmWitnessKernel (order : TraversalOrder) : Plan :=
  .tiled order Tiling.unit { m := 1, n := 1, k := 1 }
    (.loopNest { indexReg := 1, extent := 1, map := IndexMap.packed ⟨1, 1, 1⟩ }
      (.loopNest { indexReg := 2, extent := 1, map := IndexMap.packed ⟨1, 1, 1⟩ }
        (.seq (.setReg 3 0)
          (.seq (.reduce gemmWitnessContract 3 gemmWitnessA gemmWitnessB)
            (.storeReg gemmWitnessC (IndexMap.packed ⟨1, 1, 1⟩) 4 3)))))

/-- **The anti-vacuity witness.**  A complete released GEMM plan: classify the
raw header, dispatch on the layout class, run the kernel, set the status word,
build the output. -/
def gemmWitness : Plan :=
  .seq
    (.classifyRaw
      (.dispatchLayout
        (.seq (gemmWitnessKernel .ijk) (.setStatus .success))
        (.seq (gemmWitnessKernel .jik) (.setStatus .success))
        (.setStatus .unsupported))
      (.setStatus .invalid)
      (.setStatus .unsupported)
      (.setStatus .resourceExhausted))
    (.buildOutput gemmWitnessC)

/-! ### The witness is inside the released subset and type checks -/

/-- **The released subset is inhabited by a GEMM plan.** -/
theorem gemmWitness_inReleasedSubset : gemmWitness.inReleasedSubset = true := by
  decide

/-- The witness uses no vector operation, so the released profile's lack of
SIMD refuses nothing in it. -/
theorem gemmWitness_usesVector : gemmWitness.usesVector = false := by decide

/-- **The witness type checks** at the scalar interface. -/
theorem gemmWitness_typed : HasType gemmWitnessSig gemmWitness gemmWitnessOutSig := by
  decide

/-- The witness as a `CheckedPlan`. -/
def gemmWitnessChecked : CheckedPlan gemmWitnessSig gemmWitnessOutSig :=
  { plan := gemmWitness, typed := gemmWitness_typed }

/-! ### The witness compiles, validates, and refuses nothing -/

/-- **The compiled witness passes release validation.** -/
theorem gemmWitness_compiles : Wasm.validate (compile gemmWitnessChecked) = true :=
  compile_validates gemmWitnessChecked

/-- **The compilation of the witness contains no `unreachable`**: no node of a
real GEMM was refused by the translation. -/
theorem gemmWitness_no_unreachable :
    listHasUnreachable
      (bodyCode (envOf gemmWitnessSig gemmWitness) gemmWitnessSig.scratch gemmWitness)
      = false :=
  bodyCode_no_unreachable _ _ _ gemmWitness_inReleasedSubset

/-- The emitted artifact of the witness decodes back to a module that passes
release validation. -/
theorem gemmWitness_emit_decodes_valid :
    Wasm.decode (Artifact.emit (compile gemmWitnessChecked)) =
        .ok (compile gemmWitnessChecked) ∧
      Wasm.validate (compile gemmWitnessChecked) = true :=
  ⟨compile_decode_emit _, gemmWitness_compiles⟩

/-! ### What the witness computes -/

/-- The witness descriptor really classifies `valid`: the header cells are the
released ABI literals and the declared kind/mode triple is compatible. -/
theorem gemmWitnessMachine_classify (a b c : Nat) :
    (gemmWitnessMachine a b c).classify = Classification.valid := rfl

/-- The witness descriptor dispatches to the row-major continuation. -/
theorem gemmWitnessMachine_layoutClass (a b c : Nat) :
    (gemmWitnessMachine a b c).layoutClass = LayoutClass.rowMajor := rfl

/-- The witness configuration conforms to the witness interface. -/
theorem gemmWitnessMachine_conforms (a b c : Nat) :
    (gemmWitnessMachine a b c).Conforms gemmWitnessSig :=
  ⟨rfl, rfl, rfl, rfl, Nat.zero_le _⟩

/-- **The witness computes the product.**  On the `1×1×1` valid descriptor the
accumulator register holds exactly the modular-`u32` product of the `A` and `B`
elements. -/
theorem gemmWitness_eval_acc (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).reg 3 = a * b % 4294967296 := by
  show (0 + a * b) % 4294967296 = a * b % 4294967296
  rw [Nat.zero_add]

-- The three evaluation equations below run the whole plan on a 291-cell
-- memory.  The kernel's `storeReg` node deposits four little-endian cells at
-- cell 258 with `List.set`, and unfolding that through the 256-cell header
-- nests the reduction deeper, and costs more, than this file's default
-- budgets; both are therefore raised for the evaluation results below.
set_option maxHeartbeats 4000000
set_option maxRecDepth 100000

/-- The witness sets the released success status word. -/
theorem gemmWitness_eval_status (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).status = Status.success.code := rfl

/-- **The witness publishes its result.**  `buildOutput` publishes the declared
`C` region, which is exactly the four-cell little-endian image of the
accumulated product the kernel stored. -/
theorem gemmWitness_eval_out (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).out =
      leBytes (a * b % 4294967296) 4 := by
  show leBytes ((0 + a * b) % 4294967296) 4 = leBytes (a * b % 4294967296) 4
  rw [Nat.zero_add]

/-- The memory the witness leaves: the entry memory with the four-cell
little-endian image of the accumulated product deposited at `C`.
`gemmWitness_writes_C` reads that image back and `gemmWitness_eval_mem_outside`
is the frame. -/
theorem gemmWitness_eval_mem (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).mem =
      gather (gemmWitnessMem a b c) 258
        (leBytes (a * b % 4294967296) 4) (fun i => i) 4 0 := by
  show gather (gemmWitnessMem a b c) 258
      (leBytes ((0 + a * b) % 4294967296) 4) (fun i => i) 4 0 = _
  rw [Nat.zero_add]

/-- **Frame.**  Every memory cell outside the four the `C` store writes keeps
its entry value: the witness touches nothing but `C`. -/
theorem gemmWitness_eval_mem_outside (a b c x : Nat) (hx : x < 258 ∨ 262 ≤ x) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).mem.getD x 0 =
      (gemmWitnessMem a b c).getD x 0 := by
  rw [gemmWitness_eval_mem]
  exact gather_getD_outside _ _ 4 (gemmWitnessMem a b c) 258 0 x (by omega)

/-- The `alpha` scalar of the witness descriptor, read out of the header field
SPEC §8.3 puts at cells `168..175`. -/
def gemmWitnessAlpha : Nat := leWord gemmWitnessHeader 168 8

/-- The `beta` scalar of the witness descriptor, read out of the header field
SPEC §8.3 puts at cells `184..191`. -/
def gemmWitnessBeta : Nat := leWord gemmWitnessHeader 184 8

/-- The witness descriptor declares `alpha = 1`. -/
theorem gemmWitnessAlpha_eq : gemmWitnessAlpha = 1 := rfl

/-- The witness descriptor declares `beta = 0`. -/
theorem gemmWitnessBeta_eq : gemmWitnessBeta = 0 := rfl

/-- **The witness really computes GEMM: the `C` region holds the result.**
Reading the declared `C` region back little-endian at the stored width after
evaluation gives `alpha · A · B + beta · C` in the modular `u32` ring of the
declared arithmetic contract, with `alpha` and `beta` the scalars this
descriptor declares.

Since the descriptor fixes `alpha = 1` and `beta = 0`, this is the
`alpha = 1, beta = 0` instance of SPEC §8's `C ← alpha · op(A) · op(B) +
beta · C`; the plan carries no scaling node, so nothing here proves the general
`alpha`/`beta` form and nothing here claims to. -/
theorem gemmWitness_writes_C (a b c : Nat) :
    leWord (gemmWitness.eval (gemmWitnessMachine a b c)).mem gemmWitnessC.base 4 =
      (gemmWitnessAlpha * (a * b) + gemmWitnessBeta * c) % 4294967296 := by
  have hlen : (gemmWitnessMem a b c).length = 291 := gemmWitnessMem_length a b c
  have hinside : ∀ k, k < 4 →
      (gather (gemmWitnessMem a b c) 258 (leBytes (a * b % 4294967296) 4)
          (fun i => i) 4 0).getD (258 + k) 0 =
        (leBytes (a * b % 4294967296) 4).getD (0 + k) 0 := by
    intro k hk
    have h := gather_getD_inside (leBytes (a * b % 4294967296) 4) (fun i => i) 4
      (gemmWitnessMem a b c) 258 0 (258 + k) (by omega) (by omega)
      (by rw [hlen]; omega)
    have he : 258 + k - 258 = k := by omega
    rw [h]
    simp only [he, Nat.zero_add]
  have hbase : gemmWitnessC.base = 258 := rfl
  rw [gemmWitness_eval_mem, hbase,
    leWord_ext 4 _ (leBytes (a * b % 4294967296) 4) 258 0 hinside,
    leWord_leBytes]
  have hpow : (256 : Nat) ^ 4 = 4294967296 := by decide
  rw [hpow, Nat.mod_eq_of_lt (Nat.mod_lt _ (by omega)),
    gemmWitnessAlpha_eq, gemmWitnessBeta_eq, Nat.one_mul, Nat.zero_mul,
    Nat.add_zero]

/-- Type safety instantiated at the witness. -/
theorem gemmWitness_eval_conforms (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).Conforms gemmWitnessOutSig :=
  hasType_preservation gemmWitness_typed _ (gemmWitnessMachine_conforms a b c)

/-- **`Plan.inReleasedSubset` is inhabited**, and its inhabitant really is a
compiled, validating, `unreachable`-free artifact. -/
theorem exists_inReleasedSubset_checkedPlan :
    ∃ (s t : Sig) (c : CheckedPlan s t),
      c.plan.inReleasedSubset = true ∧
      Wasm.validate (compile c) = true ∧
      listHasUnreachable (bodyCode (envOf s c.plan) s.scratch c.plan) = false :=
  ⟨gemmWitnessSig, gemmWitnessOutSig, gemmWitnessChecked,
    gemmWitness_inReleasedSubset, gemmWitness_compiles, gemmWitness_no_unreachable⟩

end WasmGemmGnaf.GNAF
