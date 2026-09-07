//! Offline compilation of learned NoWrite decisions with exact serving guards.
//! Geometry is a shortlist, never permission to merge unequal writer inputs.
use super::relation::{boundary_context, boundary_metadata, head, write_choice_from_metadata};
use super::value_lexemes::{LexemeState, WordAtom};
use super::value_types::{ValueEntry, ValueWork};
use super::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;

const CAPACITY: usize = 64;
const WRITER_CAPACITY: usize = 256;

fn capacity(model: &Model) -> usize {
    if model.relation_writer.is_some() {
        WRITER_CAPACITY
    } else {
        CAPACITY
    }
}
const BUCKET_LIMIT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationAdmissionMode {
    Geometric,
    Sparse,
    /// Merge every route into one partition, preserving exact guards/fallback.
    Collapsed,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Signature {
    len: u8,
    primes: [u32; 8],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    boundaries: Option<[u16; 8]>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    root: u16,
    phase_bins: [u8; PHASE_CHANNELS],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    route: Route,
    exact: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Admission {
    schema: String,
    parent: String,
    mode: RelationAdmissionMode,
    entries: Vec<Entry>,
    training: Vec<DocumentReceipt>,
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
fn signature(
    model: &Model,
    words: &[WordAtom],
    addr: &[u32; 16],
    work: &mut ValueWork,
) -> Signature {
    let len = words.len();
    let mut primes = [0; 8];
    primes[..len].copy_from_slice(&addr[..len]);
    work.relations.admission_signature_writes =
        work.relations.admission_signature_writes.saturating_add(9);
    Signature {
        len: len as u8,
        primes,
        boundaries: boundary_context(model).then(|| {
            work.relations.admission_signature_writes =
                work.relations.admission_signature_writes.saturating_add(8);
            boundary_metadata(words, work)
        }),
    }
}

fn route(model: &Model, s: &Signature, work: &mut ValueWork) -> Route {
    let Some(h) = head(model) else {
        return Route::default();
    };
    let context = model
        .relation_writer
        .as_ref()
        .map_or(h.role_context.as_slice(), |w| w.role_context.as_slice());
    let mut root = model.geometry.identity;
    let mut phases = [0_u16; PHASE_CHANNELS];
    for prime in s.primes[..usize::from(s.len)].iter().rev() {
        work.relations.role_path_probes = work.relations.role_path_probes.saturating_add(1);
        if *prime == 0 {
            continue;
        }
        if let Ok(i) = context.binary_search_by(|g| {
            work.relations.role_path_comparisons =
                work.relations.role_path_comparisons.saturating_add(1);
            work.relations.admission_metadata_bytes =
                work.relations.admission_metadata_bytes.saturating_add(4);
            g.prime.cmp(prime)
        }) {
            let g = &context[i];
            root = model.geometry.products
                [model.geometry.row_bases[usize::from(root)] + usize::from(g.leaf)];
            work.h4_reads += 2;
            work.relations.admission_metadata_bytes = work
                .relations
                .admission_metadata_bytes
                .saturating_add(20 + std::mem::size_of::<usize>() as u64); // leaf, phases, row base, product
            for (p, d) in phases.iter_mut().zip(g.phases) {
                *p = p.wrapping_add(d);
            }
            work.relations.role_path_steps = work.relations.role_path_steps.saturating_add(1);
            work.relations.role_phase_additions = work
                .relations
                .role_phase_additions
                .saturating_add(PHASE_CHANNELS as u64);
        }
    }
    Route {
        root,
        phase_bins: phases.map(|p| (p >> 12) as u8),
    }
}

fn exact_cmp(a: &Signature, b: &Signature, work: &mut ValueWork) -> Ordering {
    work.relations.admission_exact_comparisons =
        work.relations.admission_exact_comparisons.saturating_add(1);
    work.relations.admission_metadata_bytes =
        work.relations.admission_metadata_bytes.saturating_add(1);
    let c = a.len.cmp(&b.len);
    if !c.is_eq() {
        return c;
    }
    for (x, y) in a.primes.iter().zip(b.primes) {
        work.relations.admission_exact_comparisons =
            work.relations.admission_exact_comparisons.saturating_add(1);
        work.relations.admission_metadata_bytes =
            work.relations.admission_metadata_bytes.saturating_add(4);
        let c = x.cmp(&y);
        if !c.is_eq() {
            return c;
        }
    }
    if a.boundaries.is_some() || b.boundaries.is_some() {
        work.relations.admission_exact_comparisons =
            work.relations.admission_exact_comparisons.saturating_add(1);
        work.relations.admission_metadata_bytes =
            work.relations.admission_metadata_bytes.saturating_add(1);
        let c = a.boundaries.is_some().cmp(&b.boundaries.is_some());
        if !c.is_eq() {
            return c;
        }
        if let Some((a, b)) = a.boundaries.as_ref().zip(b.boundaries.as_ref()) {
            for (a, b) in a.iter().zip(b) {
                work.relations.admission_exact_comparisons =
                    work.relations.admission_exact_comparisons.saturating_add(1);
                work.relations.admission_metadata_bytes =
                    work.relations.admission_metadata_bytes.saturating_add(2);
                let c = a.cmp(b);
                if !c.is_eq() {
                    return c;
                }
            }
        }
    }
    Ordering::Equal
}

fn route_cmp(a: Route, b: Route, work: &mut ValueWork) -> Ordering {
    work.relations.admission_route_comparisons =
        work.relations.admission_route_comparisons.saturating_add(1);
    work.relations.admission_metadata_bytes =
        work.relations.admission_metadata_bytes.saturating_add(2);
    let c = a.root.cmp(&b.root);
    if !c.is_eq() {
        return c;
    }
    for (x, y) in a.phase_bins.into_iter().zip(b.phase_bins) {
        work.relations.admission_route_comparisons =
            work.relations.admission_route_comparisons.saturating_add(1);
        work.relations.admission_metadata_bytes =
            work.relations.admission_metadata_bytes.saturating_add(1);
        let c = x.cmp(&y);
        if !c.is_eq() {
            return c;
        }
    }
    Ordering::Equal
}

pub(super) fn skip(
    model: &Model,
    gate: &Admission,
    words: &[WordAtom],
    addr: &[u32; 16],
    work: &mut ValueWork,
) -> bool {
    work.relations.admission_queries = work.relations.admission_queries.saturating_add(1);
    let s = signature(model, words, addr, work);
    let found = if gate.mode == RelationAdmissionMode::Sparse {
        gate.entries
            .binary_search_by(|e| exact_cmp(&e.exact, &s, work))
            .is_ok()
    } else {
        let key = if gate.mode == RelationAdmissionMode::Collapsed {
            Route::default()
        } else {
            route(model, &s, work)
        };
        let start = gate
            .entries
            .partition_point(|e| route_cmp(e.route, key, work).is_lt());
        let end = gate
            .entries
            .partition_point(|e| !route_cmp(e.route, key, work).is_gt());
        if end - start > BUCKET_LIMIT {
            work.relations.admission_crowded_fallbacks =
                work.relations.admission_crowded_fallbacks.saturating_add(1);
            false
        } else {
            gate.entries[start..end]
                .iter()
                .any(|e| exact_cmp(&e.exact, &s, work).is_eq())
        }
    };
    if found {
        work.relations.admission_skips = work.relations.admission_skips.saturating_add(1);
    } else {
        work.relations.admission_fallbacks = work.relations.admission_fallbacks.saturating_add(1);
    }
    found
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

fn admission_mut(model: &mut Model) -> Result<&mut Option<Admission>> {
    if let Some(writer) = &mut model.relation_writer {
        return Ok(&mut writer.admission);
    }
    Ok(&mut model
        .response_entry
        .as_mut()
        .and_then(|e| e.copy.as_mut())
        .and_then(|c| c.role_read.as_mut())
        .and_then(|r| r.relations.as_mut())
        .ok_or_else(|| Error("relation admission parent absent".into()))?
        .admission)
}

impl Admission {
    fn bound_parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        *admission_mut(&mut parent)? = None;
        parent.refresh_identity()?;
        if self.parent != parent.artifact_cid() {
            return Err(Error("admission parent differs".into()));
        }
        Ok(parent)
    }

    pub(super) fn compatible_writer(&self, model: &Model) -> bool {
        self.entries.iter().all(|e| {
            if e.exact.boundaries.is_some() != boundary_context(model) {
                return false;
            }
            let mut addr = [0; 16];
            addr[..8].copy_from_slice(&e.exact.primes);
            write_choice_from_metadata(
                model,
                &[WordAtom::default(); 8][..usize::from(e.exact.len)],
                &addr,
                e.exact.boundaries.as_ref(),
                &mut ValueWork::default(),
            )
            .is_none()
        })
    }

    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        if self.schema != "uor-r4.relation-admission/1"
            || self.entries.is_empty()
            || self.entries.len() > capacity(model)
            || self.training.is_empty()
            || self.training.len() > 256
            || head(model).is_none_or(|h| h.schema != "uor-r4.exact-relation/2")
        {
            return Err(Error("invalid relation admission bounds/schema".into()));
        }
        let parent = self.bound_parent(model)?;
        let mut sorted = self.entries.clone();
        sort_entries(&mut sorted, self.mode);
        if sorted != self.entries {
            return Err(Error("admission order differs".into()));
        }
        let mut keys = std::collections::BTreeSet::new();
        for e in &self.entries {
            let n = usize::from(e.exact.len);
            if !(2..=8).contains(&n)
                || e.exact.primes[n..].iter().any(|p| *p != 0)
                || !keys.insert(&e.exact)
                || e.exact.boundaries.is_some() != boundary_context(model)
                || e.exact.boundaries.is_some_and(|b| {
                    b.iter().any(|&v| v > 512) || b[n.saturating_sub(1)..].iter().any(|&v| v != 0)
                })
            {
                return Err(Error("invalid admission exact signature".into()));
            }
            let mut addr = [0; 16];
            addr[..8].copy_from_slice(&e.exact.primes);
            let words = [WordAtom::default(); 8];
            if write_choice_from_metadata(
                &parent,
                &words[..n],
                &addr,
                e.exact.boundaries.as_ref(),
                &mut ValueWork::default(),
            )
            .is_some()
            {
                return Err(Error("admission signature is not NoWrite".into()));
            }
            let expected = if self.mode == RelationAdmissionMode::Collapsed {
                Route::default()
            } else {
                route(&parent, &e.exact, &mut ValueWork::default())
            };
            if expected != e.route {
                return Err(Error("admission geometry differs".into()));
            }
        }
        Ok(())
    }
}

fn sort_entries(entries: &mut [Entry], mode: RelationAdmissionMode) {
    entries.sort_by(|a, b| {
        if mode == RelationAdmissionMode::Sparse {
            a.exact.cmp(&b.exact)
        } else {
            a.route.cmp(&b.route).then(a.exact.cmp(&b.exact))
        }
    });
}

// Propose other phases only for an exactly repeated full construction window.
// These are not observed frequency counts and grant no NoWrite permission;
// every proposal must still pass the unchanged writer below.
fn periodic_phases(counts: &mut BTreeMap<Signature, usize>) -> usize {
    let observed: Vec<_> = counts.keys().cloned().collect();
    let before = counts.len();
    for s in observed {
        if s.len != 8 || s.boundaries.is_some() {
            continue;
        }
        let Some(period) =
            (1..=4).find(|&p| 8 % p == 0 && (0..8).all(|i| s.primes[i] == s.primes[i % p]))
        else {
            continue;
        };
        for shift in 1..period {
            let mut next = s.clone();
            next.primes.rotate_left(shift);
            counts.entry(next).or_insert(0);
        }
    }
    counts.len() - before
}

impl Model {
    /// Compile frequent, exactly reproducible NoWrite decisions of the learned
    /// /2 writer. No writer refit, answer labels, or approximate negative gate.
    pub fn compile_relation_admission(
        &self,
        documents: &[RelationExample],
    ) -> Result<(Model, serde_json::Value)> {
        let h = head(self).ok_or_else(|| Error("relation model absent".into()))?;
        if h.schema != "uor-r4.exact-relation/2"
            || self
                .relation_writer
                .as_ref()
                .map_or(h.admission.is_some(), |w| w.admission.is_some())
            || documents.is_empty()
            || documents.len() > 256
            || documents.iter().map(|d| d.prompt.len()).sum::<usize>() > 1024 * 1024
        {
            return Err(Error("invalid admission construction".into()));
        }
        let mut counts = BTreeMap::<Signature, usize>::new();
        let mut training = Vec::new();
        let mut boundaries = 0;
        for d in documents {
            training.push(super::training::receipt(&Document {
                id: d.id.clone(),
                text: d.prompt.clone(),
            }));
            let mut words = LexemeState::default();
            let mut last = None;
            // /2 discards payload/global poses. The optional boundary law also
            // consumes exact source gap descriptors derived from byte endpoints.
            for (sequence, token) in self.encode(&d.prompt)?.into_iter().enumerate() {
                let single;
                let bytes = if token < LEXICAL_BASE {
                    single = [(token - 2) as u8];
                    &single[..]
                } else {
                    &self.lexical_pieces[(token - LEXICAL_BASE) as usize][..]
                };
                for &b in bytes {
                    words.feed(
                        b,
                        ValueEntry {
                            sequence: sequence as u64,
                            token,
                            cue: 0,
                            pose: self.geometry.identity,
                            phases: [0; PHASE_CHANNELS],
                        },
                        &mut ValueWork::default(),
                    );
                    if words.recent_len < 2 || last == Some(words.recent[0].byte_end) {
                        continue;
                    }
                    last = Some(words.recent[0].byte_end);
                    let n = words.recent_len.min(8);
                    let addr = super::relation::writer_addresses(
                        self,
                        &words.recent[..n],
                        &mut ValueWork::default(),
                    );
                    boundaries += 1;
                    *counts
                        .entry(signature(
                            self,
                            &words.recent[..n],
                            &addr,
                            &mut ValueWork::default(),
                        ))
                        .or_default() += 1;
                }
            }
        }
        let distinct = counts.len();
        let periodic_proposals = if self.relation_writer.is_some() && !boundary_context(self) {
            periodic_phases(&mut counts)
        } else {
            0
        };
        let mut eligible = Vec::new();
        let mut certification_work = ValueWork::default();
        for (s, count) in counts {
            let mut addr = [0; 16];
            addr[..8].copy_from_slice(&s.primes);
            if write_choice_from_metadata(
                self,
                &vec![WordAtom::default(); usize::from(s.len)],
                &addr,
                s.boundaries.as_ref(),
                &mut certification_work,
            )
            .is_none()
            {
                eligible.push((s, count));
            }
        }
        eligible.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let eligible_count = eligible.len();
        eligible.truncate(capacity(self));
        let covered: usize = eligible.iter().map(|(_, n)| n).sum();
        let mut entries: Vec<_> = eligible
            .into_iter()
            .map(|(exact, _)| Entry {
                route: route(self, &exact, &mut ValueWork::default()),
                exact,
            })
            .collect();
        // Retain the previously selected sparse exact lookup for replacement
        // writers. Legacy geometric compilation and its controls stay available.
        let mode = if self.relation_writer.is_some() {
            RelationAdmissionMode::Sparse
        } else {
            RelationAdmissionMode::Geometric
        };
        sort_entries(&mut entries, mode);
        let count = entries.len();
        let gate = Admission {
            schema: "uor-r4.relation-admission/1".into(),
            parent: self.artifact_cid().into(),
            mode,
            entries,
            training,
        };
        let mut model = self.clone();
        *admission_mut(&mut model)? = Some(gate);
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"mode":mode,"parent":self.artifact_cid(),"boundaries":boundaries,"distinct_signatures":distinct,"periodic_phase_proposals":periodic_proposals,"periodic_boundary_proposals_disabled":boundary_context(self),"signature_boundary_context":boundary_context(self),"eligible_no_write_signatures":eligible_count,"entries":count,"capacity":capacity(self),"construction_selected_boundaries":covered,"certification_work":certification_work,"entry_layout_bytes":std::mem::size_of::<Entry>(),"persistent_state_bytes_added":0,"bucket_limit":BUCKET_LIMIT,"scope":"Frequency-selected exact negatives from existing learned /2 writer; no new predictive fit. Full exact prime signature and, when enabled, last-separator/adjacency descriptor guard; geometric mode adds a shortlist. Boundary-aware compilation uses only observed windows and disables periodic proposals. Unmatched signatures and crowded geometric routes fall back to unchanged writer."}),
        ))
    }

    /// Matched routing controls share all exact entries and learned operators.
    pub fn with_relation_admission_mode(&self, mode: RelationAdmissionMode) -> Result<Model> {
        let mut model = self.clone();
        let gate = admission_mut(&mut model)?
            .as_mut()
            .ok_or_else(|| Error("admission absent".into()))?;
        gate.mode = mode;
        for e in &mut gate.entries {
            e.route = if mode == RelationAdmissionMode::Collapsed {
                Route::default()
            } else {
                route(self, &e.exact, &mut ValueWork::default())
            };
        }
        sort_entries(&mut gate.entries, mode);
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_relation_boundary_alias_and_exact_cache_guard() {
        use super::super::relation::{write_features_with_context, write_features_with_metadata};
        use super::super::relation_training::WriterRevision;
        use super::super::writer_refinement::WriterRefinement;
        let docs = [Document {
            id: "boundary-feature".into(),
            text: "in now not".into(),
        }];
        let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
        trainer.train_documents(&docs).unwrap();
        let mut model = trainer.compile().unwrap();
        let words = |text: &str| {
            let mut state = LexemeState::default();
            for byte in text.bytes() {
                state.feed(byte, ValueEntry::default(), &mut ValueWork::default());
            }
            state.finish(&mut ValueWork::default());
            state
        };
        let valid = words("notes say quiet holds ranvi.");
        let crossed = words("ada carries stone holds.\nUser:");
        let valid_words = &valid.recent[..valid.recent_len];
        let crossed_words = &crossed.recent[..crossed.recent_len];
        assert_eq!(valid_words.len(), 5);
        assert_eq!(crossed_words.len(), 5);
        // Frozen dictionary: holds has a prime; the other context words are
        // unknown, and both selected payload positions are role-masked.
        let mut addr = [0; 16];
        addr[1] = 37;
        let features = |model: &Model, words: &[WordAtom]| {
            write_features_with_context(
                model,
                words,
                &addr,
                0,
                2,
                Some(&[]),
                &mut ValueWork::default(),
            )
        };
        let old = features(&model, valid_words);
        assert_eq!(old, features(&model, crossed_words));
        let previous = WriterRevision {
            schema: "uor-r4.relation-writer/1".into(),
            parent: String::new(),
            dictionary: vec![],
            role_context: vec![],
            rows: vec![],
            training: vec![],
            epochs: 1,
            reuse_admission: false,
            admission: None,
        };
        model.relation_writer_refinement = Some(WriterRefinement {
            schema: "uor-r4.relation-writer-refinement/1".into(),
            parent_artifact: String::new(),
            previous,
            max_seconds: 10,
            boundary_context: true,
        });
        let revised = features(&model, valid_words);
        let cross_revised = features(&model, crossed_words);
        assert_eq!(revised.1, old.1 + 2);
        assert_eq!(&revised.0[..old.1], &old.0[..old.1]);
        assert_eq!(&cross_revised.0[..old.1], &old.0[..old.1]);
        assert_ne!(revised, cross_revised);
        let metadata = boundary_metadata(crossed_words, &mut ValueWork::default());
        assert_eq!(metadata[0], u16::from(b'\n') + 257);
        assert_eq!(
            boundary_metadata(valid_words, &mut ValueWork::default())[0],
            33
        );
        let wider = words("notes say quiet holds  ranvi");
        assert_eq!(
            boundary_metadata(&wider.recent[..wider.recent_len], &mut ValueWork::default())[0],
            289
        );
        let newline = words("notes say quiet holds\nranvi");
        assert_eq!(
            boundary_metadata(
                &newline.recent[..newline.recent_len],
                &mut ValueWork::default()
            )[0],
            11
        );
        // Offline certification can discard poses/endpoints only after retaining
        // the complete feature metadata. Replaying dummy words alone would alias.
        assert_eq!(
            cross_revised,
            write_features_with_metadata(
                &model,
                &[WordAtom::default(); 5],
                &addr,
                0,
                2,
                Some(&[]),
                Some(&metadata),
                &mut ValueWork::default()
            )
        );
        let exact = signature(&model, crossed_words, &addr, &mut ValueWork::default());
        let mut gate = Admission {
            schema: "test-only".into(),
            parent: String::new(),
            mode: RelationAdmissionMode::Sparse,
            entries: vec![Entry {
                route: Route::default(),
                exact: exact.clone(),
            }],
            training: vec![],
        };
        assert!(skip(
            &model,
            &gate,
            crossed_words,
            &addr,
            &mut ValueWork::default()
        ));
        assert!(!skip(
            &model,
            &gate,
            valid_words,
            &addr,
            &mut ValueWork::default()
        ));
        let good = signature(&model, valid_words, &addr, &mut ValueWork::default());
        assert_eq!(good.primes, exact.primes);
        assert_eq!(
            exact_cmp(&good, &exact, &mut ValueWork::default()),
            good.cmp(&exact)
        );
        gate.entries[0].exact.boundaries = None;
        assert!(!gate.compatible_writer(&model));
        let wire = serde_json::to_value(&gate.entries[0].exact).unwrap();
        assert!(wire.get("boundaries").is_none());
        let periodic = Signature {
            len: 8,
            primes: [37; 8],
            boundaries: Some([33, 33, 33, 33, 33, 33, 33, 0]),
        };
        let mut counts = BTreeMap::from([(periodic, 2)]);
        assert_eq!(periodic_phases(&mut counts), 0);
        assert_eq!(counts.len(), 1);
    }

    #[test]
    fn native_relation_admission_periodic_phases_preserve_observed_counts() {
        let a = Signature {
            len: 8,
            primes: [79, 73, 79, 73, 79, 73, 79, 73],
            boundaries: None,
        };
        let b = Signature {
            len: 8,
            primes: [73, 79, 73, 79, 73, 79, 73, 79],
            boundaries: None,
        };
        let nonperiodic = Signature {
            len: 8,
            primes: [1, 2, 3, 4, 5, 6, 7, 8],
            boundaries: None,
        };
        let mut counts = BTreeMap::from([(a.clone(), 112), (nonperiodic.clone(), 3)]);
        assert_eq!(periodic_phases(&mut counts), 1);
        assert_eq!(counts.get(&a), Some(&112));
        assert_eq!(counts.get(&b), Some(&0));
        assert_eq!(counts.get(&nonperiodic), Some(&3));
        assert_eq!(periodic_phases(&mut counts), 0);
    }

    #[test]
    fn native_relation_admission_binds_replacement_writer_and_roundtrips() {
        let docs = [Document {
            id: "cache-parent".into(),
            text: "in now not".into(),
        }];
        let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
        trainer.train_documents(&docs).unwrap();
        let mut model = trainer.compile().unwrap();
        // Only the parent-binding boundary is under test; this synthetic writer
        // deliberately has no predictive rows and is not a qualified artifact.
        model.relation_writer = Some(super::super::relation_training::WriterRevision {
            schema: "uor-r4.relation-writer/1".into(),
            parent: model.artifact_cid().into(),
            dictionary: vec![],
            role_context: vec![],
            rows: vec![],
            training: vec![],
            epochs: 1,
            reuse_admission: false,
            admission: None,
        });
        model.refresh_identity().unwrap();
        let original = model.to_bytes().unwrap();
        assert!(
            !serde_json::from_slice::<serde_json::Value>(&original).unwrap()["relation_writer"]
                .as_object()
                .unwrap()
                .contains_key("admission")
        );
        let gate = Admission {
            schema: "uor-r4.relation-admission/1".into(),
            parent: model.artifact_cid().into(),
            mode: RelationAdmissionMode::Sparse,
            entries: vec![],
            training: vec![],
        };
        *admission_mut(&mut model).unwrap() = Some(gate.clone());
        model.refresh_identity().unwrap();
        assert_eq!(
            gate.bound_parent(&model).unwrap().to_bytes().unwrap(),
            original
        );
        let decoded: Admission =
            serde_json::from_slice(&serde_json::to_vec(&gate).unwrap()).unwrap();
        assert_eq!(decoded, gate);
        // Rebinding a cache to changed writer metadata must fail even when the
        // outer model identity is regenerated. Removing it recovers the parent.
        model.relation_writer.as_mut().unwrap().epochs = 2;
        model.refresh_identity().unwrap();
        assert!(gate.bound_parent(&model).is_err());
    }

    #[test]
    fn native_relation_admission_exact_guard_and_crowded_fallback() {
        let docs = [Document {
            id: "admission-guard".into(),
            text: "in now not".into(),
        }];
        let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
        trainer.train_documents(&docs).unwrap();
        let model = trainer.compile().unwrap();
        let mut entries: Vec<_> = (0..9)
            .map(|i| Entry {
                route: Route::default(),
                exact: Signature {
                    len: 2,
                    primes: [i, 0, 0, 0, 0, 0, 0, 0],
                    boundaries: None,
                },
            })
            .collect();
        let mut gate = Admission {
            schema: "test-only".into(),
            parent: String::new(),
            mode: RelationAdmissionMode::Sparse,
            entries: entries.clone(),
            training: vec![],
        };
        let mut addr = [0; 16];
        addr[0] = 4;
        assert!(skip(
            &model,
            &gate,
            &[WordAtom::default(); 2],
            &addr,
            &mut ValueWork::default()
        ));
        addr[1] = 99; // Same coarse route; an unequal exact member cannot skip.
        assert!(!skip(
            &model,
            &gate,
            &[WordAtom::default(); 2],
            &addr,
            &mut ValueWork::default()
        ));
        addr[1] = 0;
        gate.mode = RelationAdmissionMode::Collapsed;
        let mut work = ValueWork::default();
        assert!(!skip(
            &model,
            &gate,
            &[WordAtom::default(); 2],
            &addr,
            &mut work
        ));
        assert_eq!(work.relations.admission_crowded_fallbacks, 1);
        assert_eq!(work.relations.admission_exact_comparisons, 0);
        entries.pop();
        gate.entries = entries;
        assert!(skip(
            &model,
            &gate,
            &[WordAtom::default(); 2],
            &addr,
            &mut ValueWork::default()
        ));
        assert!(!skip(
            &model,
            &gate,
            &[WordAtom::default(); 3],
            &addr,
            &mut ValueWork::default()
        ));
        let mut max_work = ValueWork::default();
        max_work.relations.admission_queries = u64::MAX;
        max_work.relations.admission_signature_writes = u64::MAX;
        max_work.relations.admission_route_comparisons = u64::MAX;
        max_work.relations.admission_exact_comparisons = u64::MAX;
        max_work.relations.admission_metadata_bytes = u64::MAX;
        max_work.relations.admission_skips = u64::MAX;
        assert!(skip(
            &model,
            &gate,
            &[WordAtom::default(); 2],
            &addr,
            &mut max_work
        ));
        assert_eq!(max_work.relations.admission_metadata_bytes, u64::MAX);
        assert_eq!(max_work.relations.admission_queries, u64::MAX);
    }
}
