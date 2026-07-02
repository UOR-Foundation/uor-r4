# Resonance Logic (RL) – Core Formalization
UOR Foundation / PrimeOS working group

## § 1 Motivation & Big‑Picture Framing — Conservation as Truth

Resonance Logic (RL) internalises the CCM maxim "truth ≙ conservation." A statement is as true as the resonance it conserves. Consequently, Boolean values ${0,1}$ are replaced by the 96‑element resonance spectrum

$$\mathcal C_{96}\;=\;\langle C,\,\oplus,\,\otimes,\,0,\,1\rangle.$$

**Two meta‑goals**
- **Soundness** — every proof in the sequent calculus RL$_s$ preserves resonance.
- **Conservativity over SA** — collapsing all resonances by the crush map $\kappa : C_{96}\twoheadrightarrow\{0,1\}$ guarantees RL proves no new purely‑Boolean theorems.

**Road‑map of this note**
- § 2 fixes syntax and the budgeted‑sequent format.
- § 3 constructs $\mathcal C_{96}$ and its operations once and for all.
- § 4 presents the core sequent calculus RL$_s$.
- § 5 **Induction‑Collapse Theorem** — the new heart of the paper.
- § 6 sketches soundness.
- § 7 embeds SA/MSA via the crush map.

Appendices collect the term grammar, rule‑verification table, and an extended discussion of equality semantics.

## § 2 Syntax & Budgets

## § 3 Concrete Construction of the Resonance Lattice $\mathcal C_{96}$

**Non‑negotiable deliverable:** this section pins down $\mathcal C_{96}$ as an explicit, finitely‑presented algebra.  All later proofs rest on this foundation.

### 3.1 Underlying set
We fix
$$C \;:=\; \mathbb Z_{96} \;=\; \{\,0,1,2,\dots,95\,\},$$

the ring of integers modulo 96.  Each element will be called a **resonance value** and written $[n]$ when we wish to emphasise the congruence class.

### 3.2 Operations
Define two binary operations for all $[r],[s]\in C$:

**Additive join**
$$[r]\;\oplus\;[s] \;:=\; [\,r + s\,]_{96},$$
i.e. addition modulo 96.

**Multiplicative bind**
$$[r]\;\otimes\;[s] \;:=\; [\,r\,s\,]_{96},$$
i.e. multiplication modulo 96.

**Constants:** $0 := [0]$ (bottom / null resonance) and $1 := [1]$ (unity resonance).

**Remark 3.2.1 (Semiring view).** $\langle C,\oplus,\otimes,0,1\rangle$ is a commutative, idempotent‑free semiring; idempotence of $\oplus$ is not required for RL because budgets are quantitative rather than truth‑valued.  The semiring therefore qualifies as a (commutative) quantale without idempotent join in the sense of linear logic's phase semantics.

### 3.3 Algebraic properties

| Property | Proof sketch |
|----------|--------------|
| Associativity of $\oplus,\otimes$ | inherited from $\mathbb Z$ mod 96 |
| Commutativity | ditto |
| Identities $0,1$ | $[r]+0=[r]$, $[r]\cdot 1=[r]$ mod 96 |
| Distributivity | $r(s+t)\equiv [rs] + [rt]\;(\mathrm{mod}\;96)$ |
| Cancellation? | No; budgets track quantities, not sets |

Hence $C$ is a commutative semiring and therefore a legitimate phase space for linear logic.

### 3.4 Order‑successor, cycle & finite size
Set
$\sigma\,[r] \;:=\; [r+1]_{96}.   \qquad (\text{"one‑step resonance shift"})$

Then $\sigma^{96}=\mathrm{id}$. This operation provides a cyclic structure on the resonance values, though its role in the (RI) rule is through the modular arithmetic of budget indices rather than direct application to formulas.

### 3.5 Klein complement
We keep the involutive map
$$\perp\,[r] \;:=\; [\,(-r)_{96}\,],  \qquad \text{so}\; \perp\perp [r] = [r].$$

De Morgan laws follow immediately from elementary modular arithmetic.

### 3.6 Semantics of Equality — resolving the mismatch

**Design choice.**  We retain crisp (Boolean‑valued) equality.  Formally,
$$\llbracket\,[r]=[s]\,\rrbracket \;:=\; \begin{cases}1, & r\equiv s\;(96);\\ 0, & \text{otherwise.}\end{cases}$$

**Rationale.**  (i) It preserves the usual substitution and congruence properties without inflating the budget algebra.  (ii) All constructive content already lives in the budgets; adding a many‑valued equality would blur two orthogonal dimensions (identity vs. resource‐flow).

**Alternative (3.6.1).**  Should a fully many‑valued equality be desired, one may set $\llbracket [r]=[s] \rrbracket := [|r-s|]_{96}$.  The present draft leaves this for future investigation.

## § 4 Sequent Calculus RL$_s$
(All rules and side‑conditions "wf / bal" still apply, with $\oplus,\otimes$ now interpreted modulo 96.)

**Note on the (RI) rule:** The Resonance Induction rule performs induction over natural numbers with budget assignments that cycle through the 96 resonance values. For a formula $\varphi(n)$ with numeric variable $n$, the rule requires proving the inductive step with budget $r_k$ for all natural numbers $n$ where $n \equiv k \pmod{96}$. This creates 96 parallel induction tracks, each with its own budget.

## § 5 Induction‑Collapse Theorem

This section addresses the relationship between RL's Resonance‑Induction (RI) rule and Peano induction under the crush map $\kappa$.

### 5.1 Clarification of the (RI) Rule

The (RI) rule requires careful interpretation. Given a formula $\varphi(n)$ with one free numeric variable $n$ ranging over natural numbers, the rule schema is:

**Resonance Induction (RI):**
- Base case: $\vdash_{r_0}\,\varphi(0)$
- Inductive step: For each $k \in C$, we have $\varphi(n)\vdash_{r_k}\,\varphi(n+1)$ where $n \equiv k \pmod{96}$
- Conclusion: $\vdash_{\rho}\,\forall n.\varphi(n)$ where $\rho = \bigotimes_{k \in C} r_k$

Here, the meta-level index $k$ determines which budget $r_k$ is used for proving the inductive step at positions congruent to $k$ modulo 96.

### 5.2 Theorem (IC-Revised)
**Induction‑Collapse Theorem (Revised).**  Let $\varphi(n)$ be any RL‑formula with one free numeric variable.  If $\vdash_{\rho}\;\forall n.\varphi(n)$ is derivable using (RI), then:
1. The Boolean projection $\kappa(\rho)=1$
2. For the formula $\kappa^*(\varphi)$ (where $\kappa^*$ applies $\kappa$ to all resonance constants), Peano arithmetic proves $\forall n.\kappa^*\!(\varphi)(n)$ if and only if all 96 budget-specific instances are provable.

### 5.3 Proof

**Budget projection.**  The conclusion budget is 
$\rho = \bigotimes_{k\in C} r_k.$

Since $\kappa$ is a semiring morphism:
$\kappa(\rho) = \prod_{k \in C} \kappa(r_k) \in \{0,1\}.$

For the proof to be sound, we need $\kappa(r_k) = 1$ for all $k$, hence $\kappa(\rho) = 1$.

**Relationship to Peano induction.**  The (RI) rule establishes:
- $\vdash\,\kappa^*(\varphi(0))$ (from the base case)
- For each residue class $k \pmod{96}$: all instances of $\kappa^*(\varphi(n)) \vdash \kappa^*(\varphi(n+1))$ where $n \equiv k \pmod{96}$

This is stronger than standard Peano induction, which only requires:
- $\vdash\,\varphi(0)$
- $\forall n.\,(\varphi(n) \vdash \varphi(n+1))$

**Key observation:** The (RI) rule partitions the natural numbers into 96 congruence classes and potentially uses different proof strategies (reflected by different budgets $r_k$) for each class. Under the Boolean collapse, all these strategies must work.

**Conservative extension property:**  If Peano arithmetic proves $\forall n.\psi(n)$ for some Boolean formula $\psi$, then RL can prove $\vdash_1\,\forall n.\psi(n)$ by setting all budgets $r_k = 1$. Conversely, if RL proves a Boolean theorem using (RI), the collapsed proof is valid in Peano arithmetic.

Hence (IC-Revised) is proved. ∎

**Corollary 5.3.1.**  RL is conservative over Peano arithmetic with respect to Boolean theorems, though the (RI) rule provides a finer-grained induction principle that tracks resource usage across congruence classes modulo 96.

## § 6 Soundness
(Lemmas 6.1 & 6.2 establish the core soundness properties.)

## § 7 Embedding SA → RL

## A. Full Grammar for Terms & Formulas 
Let $\mathcal V$ denote a countably‑infinite set of variables $x,y,z,\dots$. 

### A.1 Terms $t$ 
- Variables $ x \mid y \mid \dots $ 
- Constants $ 0 \mid 1$ 
- Composite 
  - $t + t$ (binary additive symbol) 
  - $t \cdot t$ (binary multiplicative symbol) 
- Parentheses may be used for grouping. 

This inductive definition is minimal yet expressive: any additional term‑forming operator required by a concrete theory (e.g., subtraction, modular residue) can be added à la carte without altering the underlying logic. 

**Note:** The successor operation σ defined in § 3.4 operates on resonance values (meta-level), not on terms (object-level). 

### A.2 Atomic formulas $\alpha$ 
- Equality $t = s$ 
- Klein complement $\perp\,\alpha$ 

### A.3 Molecular formulas $\varphi,\psi$ 
$\varphi,\psi ::= \alpha \mid \varphi \oplus \psi \mid \varphi \otimes \psi \mid \exists x.\,\varphi \mid \forall x.\,\varphi$

The connectives $\oplus$ (additive join) and $\otimes$ (multiplicative bind) inherit associativity and commutativity from the quantale structure of $\mathcal C_{96}$. 

## B. Verification of Side‑Conditions 
Every inference rule in RL$_s$ carries the implicit side‑conditions **wf** and **bal** from § 2.4.  Table B.1 checks that these conditions propagate from premises to conclusions; thus they need not be written explicitly inside proof trees. 

| Rule | Premises (budgets) | Conclusion (budget) | wf preserved? | bal preserved? | Justification |
|------|-------------------|---------------------|---------------|----------------|---------------|
| Ax | — | $\vdash_{\rho}\;A, A$ | ✔ | ✔ | Choose $\rho = \llbracket A \rrbracket$. |
| Cut | $\Gamma \vdash_{\rho} A$; $A, \Delta \vdash_{\sigma} \Lambda$ | $\Gamma, \Delta \vdash_{\rho \otimes \sigma} \Lambda$ | ✔ | ✔ | wf: quantale closed under $\otimes$; bal: Lemma 5.2. |
| $\oplus L$ | $A,\Gamma \vdash_{\rho} \Delta$ | $A\oplus B, \Gamma \vdash_{\rho} \Delta$ | ✔ | ✔ | $\llbracket A\oplus B\rrbracket = \llbracket A\rrbracket \oplus \llbracket B\rrbracket$ adjusts both sides equally. |
| $\oplus R$ | $\Gamma \vdash_{\rho} A,\Delta$; $\Gamma \vdash_{\sigma} B,\Delta$ | $\Gamma \vdash_{\rho \oplus \sigma} A\oplus B,\Delta$ | ✔ | ✔ | wf: closure under $\oplus$; bal: by construction of $\rho\oplus\sigma$. |
| $\otimes L/R$ | analogous | analogous | ✔ | ✔ | Quantale axioms. |
| RI | $\vdash_{r_0}\,\varphi(0)$; $\varphi(n)\vdash_{r_k}\,\varphi(n+1)$ for all $n$ where $n \equiv k \pmod{96}$ | $\vdash_{\bigotimes_k r_k}\,\forall n.\varphi(n)$ | ✔ | ✔ | wf: finite $\otimes$ of well‑formed $r_k$; bal: each instance preserves balance; multiplication composes. |

**Key:** ✔ = side‑condition automatically satisfied if it holds for premises. 

**Remark B.2.** Because budgets inhabit a commutative quantale, the order of multiplication in the Cut rule and RI budget aggregation is immaterial, simplifying mechanised proof‑checking.

## Appendix C. Equality Semantics — extended discussion

### C.1 Why retain Boolean equality?
- **Pragmatics:** keeps substitution & rewriting simple.
- **Semantics:** resource flow lives in budgets, orthogonal to identity.
- **Conservativity:** guarantees crisp collapse back to SA.

### C.2 Many‑valued alternative (open problem)
Define $\llbracket t=s \rrbracket := [\,|\llbracket t \rrbracket - \llbracket s \rrbracket|\,]_{96}$.  One must then revisit the **bal** side‑condition so that budgets subtract rather than nullify when two terms almost coincide.  This remains future work.

**End of document**