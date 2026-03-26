# Zenodo Deposit Metadata — Angular Manifold Routing

> **Prepare this file before uploading to Zenodo.**  
> Fill in the `[FILL IN]` fields and finalize the license section before uploading.

---

## Title

Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization

---

## Authors

| Field | Value |
|---|---|
| Name | Allard, Casey |
| Affiliation | Independent Researcher |
| Email | caseyallard@hotmail.com |
| ORCID | [FILL IN — optional: https://orcid.org/xxxx-xxxx-xxxx-xxxx] |

---

## Description / Abstract

*(Copy the paper abstract here before uploading. The version below is the locked submission abstract.)*

We show that the same angular non-uniformity of L2-normalized token embeddings that enables TurboQuant's extreme data compression also enables sublinear routing computation in transformer-style architectures. A fixed Hopf fibration map exploits this structure to produce a routing footprint scaling as K^0.572 vs K^1.0 for dense routing — an advantage that persists at K=5000 (ratio 2.6–2.8×). In a 2-layer trainable language model, fixed geometric routing replaces a learned top-1 gate with only 8% validation perplexity cost and no learned gate matrix, while using 46 of 64 effective expert paths at convergence (1.4× more efficient than dense routing). A second-dataset replication on WikiText-2 (confirmed 2 seeds) finds a HOPF/BASELINE ratio of 1.081 — numerically identical to the PTB confirmed ratio — under identical training conditions. This result is scoped to the 2-layer toy-scale trainable setting and should not be read as a claim of broad MoE replacement or large-scale transformer substitution. Taken together with TurboQuant, this work suggests the angular non-uniformity of embeddings has engineering consequences in both data compression and routing computation.

---

## Keywords

*(Enter these as individual keywords in Zenodo's keyword field.)*

- transformer routing
- Hopf fibration
- mixture of experts
- sparse routing
- angular non-uniformity
- sublinear compute
- language model efficiency
- geometric deep learning
- TurboQuant

---

## Upload Type

Software + Publication (type: preprint)

---

## License

> ⚠️ **Finalize the license before uploading. Choose one option per component below.**

### Paper (main.pdf, main.tex, references.bib)

- **Recommended:** Creative Commons Attribution 4.0 International (CC BY 4.0)
  - Standard for academic preprints; allows reuse with attribution
  - Compatible with arXiv's default CC BY 4.0 option

### Code and Scripts (hopf_routing_demo.py, generate_ppmi_proxy.py, generate_figures.py)

- **Recommended: MIT License** — permissive, reuse-friendly, minimal boilerplate
- Alternative: Apache 2.0 — similar permissiveness, adds patent clause

### To finalize:
1. Add a `LICENSE` file to `papers/release_bundle/` with the chosen code license text.
2. Note in the Zenodo description whether the paper and code use different licenses.
3. Select the matching license in Zenodo's license dropdown.

---

## Related Identifiers

*(Add these after arXiv submission. Leave blank until IDs are assigned.)*

| Relationship | Identifier | Note |
|---|---|---|
| Is supplemented by | `[FILL IN arXiv ID]` | arXiv preprint |
| Is cited by | `[FILL IN arXiv ID]` | Same — fill in once submitted |
| Is source code of | `[FILL IN arXiv ID]` | Reproduction bundle for the paper |

---

## Access Rights

Open access (recommended for public preprint release)

---

## Journal / Conference

Leave blank (this is a preprint deposit)

---

## Version

1.0.0 — Initial release (concurrent with arXiv submission)

---

## Notes / Additional Description

*(Optional field in Zenodo. Suggested text:)*

> This record contains the reproduction bundle for the preprint "Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization." The bundle includes the manuscript PDF, LaTeX source, bibliography, figure generation code, and a numpy-only standalone demo that reproduces Table 3 of the paper. The demo requires a PPMI-SVD embedding built from the public WikiText-2 corpus; a generation script (generate_ppmi_proxy.py) is included. The full experimental increment records are available in the preprint's artifact bundle.

---

## Checklist Before Uploading

- [ ] arXiv ID obtained and added to `related_identifiers` above
- [ ] arXiv ID added to `main.tex` abstract/footnote and PDF recompiled
- [ ] License finalized and LICENSE file added to bundle
- [ ] `CITATION.cff` or BibTeX citation block finalized (see README.md in bundle)
- [ ] Zip the bundle: `cd papers && zip -r angular_manifold_routing_bundle.zip release_bundle/`
- [ ] Upload zip to https://zenodo.org/ under "New upload"
- [ ] Reserve DOI before publishing (Zenodo allows pre-reservation)
- [ ] Add Zenodo DOI to paper (`\footnote{Zenodo: \url{https://doi.org/10.5281/zenodo.XXXXXX}}`) and recompile
- [ ] Publish Zenodo record
- [ ] Record Zenodo DOI in `docs/research/ACTIVE_STATE.md` and `docs/DECISIONS.md`
