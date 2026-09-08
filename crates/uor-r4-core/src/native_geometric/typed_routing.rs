//! Learned geometric metadata selection; exact numeric payloads remain separate.
use super::source_routing::SourceRouting;
use super::value_types::{ValueFeature, ValueRecord, ValueState, ValueWork, VALUES};
use super::word_copy_types::WordCopyAddress;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TypedRouting {
    pub router: SourceRouting,
    pub dictionary: Vec<WordCopyAddress>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fold_ascii_case: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub canonical_copy_aliases: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub local_query: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub operand_provenance: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub literal_answers: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initialization_artifact: Option<String>,
}

/// Offline witness for continuing the shared typed-role router beneath frozen descendants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TypedRoleRefinement {
    pub parent_artifact: String,
    pub previous: TypedRouting,
}
impl TypedRoleRefinement {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let active = model
            .typed_roles
            .as_ref()
            .ok_or_else(|| Error("refined typed roles absent".into()))?;
        active.validate(model, true)?;
        if active.router.parent_artifact != self.parent_artifact {
            return Err(Error("typed role refinement parent differs".into()));
        }
        let mut parent = model.clone();
        parent.typed_role_refinement = None;
        parent.typed_roles = Some(self.previous.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("typed role refinement frozen parent differs".into()));
        }
        parent.validate()?;
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("typed role refinement identity differs".into()));
        }
        Ok(())
    }
}

pub(super) struct TypedContext {
    pub literal_component: bool,
    pub addresses: [u32; 16],
    pub depths: Option<[u8; 16]>,
    pub origins: Option<[u64; 16]>,
    pub provenance: Option<[[u8; 16]; 16]>,
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// Depth of exact Add computation, preserving depth across Copy aliases.
/// Unknown/evicted ancestry remains 255. Fixed-capacity relaxation does not
/// assume source order and terminates after at most VALUES passes.
pub(super) fn lineage_depths(values: &ValueState, work: &mut ValueWork) -> [u8; 16] {
    let mut depth = [255; 16];
    for (i, r) in values.sources.iter().enumerate() {
        work.routing.sources_examined += 1;
        if !r.derived {
            depth[i] = 0;
        }
    }
    for _ in 0..VALUES {
        let mut changed = false;
        for (i, r) in values.sources.iter().enumerate() {
            work.routing.sources_examined += 1;
            if depth[i] != 255 {
                continue;
            }
            let Some(d) = r.derivation else {
                continue;
            };
            let mut parents = [255; 2];
            for (j, source) in values.sources.iter().enumerate() {
                work.routing.sources_examined += 1;
                for k in 0..2 {
                    work.routing.comparisons += 1;
                    work.routing.logical_bytes_read += 16;
                    if source.id == d.operand_ids[k] && source.id < r.id {
                        parents[k] = depth[j];
                    }
                }
            }
            let value = if d.action == ValueAction::Copy {
                parents[0]
            } else if parents[0] != 255 && parents[1] != 255 {
                parents[0].max(parents[1]) + 1
            } else {
                255
            };
            if value != 255 {
                depth[i] = value;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    depth
}

pub(super) fn copy_origins(values: &ValueState, work: &mut ValueWork) -> [u64; 16] {
    let mut out = [u64::MAX; 16];
    for (i, record) in values.sources.iter().enumerate() {
        let mut id = record.id;
        for _ in 0..VALUES {
            let found = values.sources.iter().find(|r| {
                work.routing.sources_examined += 1;
                work.routing.comparisons += 1;
                work.routing.logical_bytes_read += 16;
                r.id == id
            });
            let Some(d) = found
                .and_then(|r| r.derivation)
                .filter(|d| d.action == ValueAction::Copy)
            else {
                break;
            };
            if d.operand_ids[0] >= id {
                break;
            }
            id = d.operand_ids[0];
        }
        out[i] = id;
    }
    out
}

/// Same operation support as the original i != j candidate contract: observing
/// a Copy must not manufacture a reflexive Add edge for one computation.
pub(super) fn alias_self_add(
    values: &ValueState,
    a: ValueRecord,
    b: ValueRecord,
    origins: Option<&[u64; 16]>,
    work: &mut ValueWork,
) -> bool {
    let Some(origins) = origins else {
        return false;
    };
    let mut pair = [u64::MAX; 2];
    for (i, r) in values.sources.iter().enumerate() {
        work.routing.sources_examined += 1;
        work.routing.comparisons += 2;
        work.routing.logical_bytes_read += 32;
        if r.id == a.id {
            pair[0] = origins[i];
        }
        if r.id == b.id {
            pair[1] = origins[i];
        }
    }
    pair[0] != u64::MAX && pair[0] == pair[1]
}

/// Query-to-literal cue matches transported through exact Copy/Add ancestry.
/// Four cue-position bits are unioned across parents; bit 4 denotes unavailable
/// ancestry. Numeric payloads and word identities are never used as distances.
/// Scratch is fixed at 256 bytes. No record or query is mutated.
pub(super) fn operand_provenance(values: &ValueState, work: &mut ValueWork) -> [[u8; 16]; 16] {
    let mut out = [[16; 16]; 16];
    let Some(words) = &values.lexemes else {
        return out;
    };
    let mut known = [false; 16];
    for (i, source) in values.sources.iter().enumerate() {
        work.routing.sources_examined += 1;
        if source.derived {
            continue;
        }
        known[i] = true;
        out[i] = [0; 16];
        let Some(cues) = source.lexical else {
            out[i] = [16; 16];
            continue;
        };
        for (q, query) in words.queries[..words.query_len].iter().enumerate() {
            work.routing.context_tokens_read += 1;
            work.routing.comparisons += 1;
            work.routing.logical_bytes_read += 16;
            if values.query_boundary.is_some_and(|start| query.end < start) {
                continue;
            }
            for (c, cue) in cues.iter().enumerate() {
                work.lexical_comparisons += 1;
                work.routing.logical_bytes_read += 2;
                if cue.len == 0 || cue.len != query.len {
                    continue;
                }
                let mut equal = true;
                for j in 0..usize::from(cue.len) {
                    work.lexical_byte_comparisons += 1;
                    work.routing.logical_bytes_read += 2;
                    if cue.bytes[j].to_ascii_lowercase() != query.bytes[j].to_ascii_lowercase() {
                        equal = false;
                        break;
                    }
                }
                if equal {
                    out[i][q] |= 1 << c;
                }
            }
        }
    }
    for _ in 0..VALUES {
        let mut changed = false;
        for (i, source) in values.sources.iter().enumerate() {
            work.routing.sources_examined += 1;
            if known[i] {
                continue;
            }
            let Some(d) = source.derivation else {
                continue;
            };
            let mut parents = [None; 2];
            for (j, r) in values.sources.iter().enumerate() {
                work.routing.sources_examined += 1;
                for k in 0..2 {
                    work.routing.comparisons += 1;
                    work.routing.logical_bytes_read += 16;
                    if r.id == d.operand_ids[k] && r.id < source.id && known[j] {
                        parents[k] = Some(j);
                    }
                }
            }
            let Some(a) = parents[0] else {
                continue;
            };
            let b = if d.action == ValueAction::Copy {
                a
            } else {
                let Some(b) = parents[1] else {
                    continue;
                };
                b
            };
            for q in 0..16 {
                work.routing.logical_bytes_read += 2;
                out[i][q] = out[a][q] | out[b][q];
            }
            known[i] = true;
            changed = true;
        }
        if !changed {
            break;
        }
    }
    out
}

pub(super) fn context(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut ValueWork,
) -> Option<TypedContext> {
    let mut block = model.typed_routing.as_ref()?;
    if matches!(
        control,
        Control::LearnedRoutingDisabled | Control::GeometryDisabled | Control::H4Disabled
    ) {
        return None;
    }
    let mut derived = 0;
    for source in &values.sources {
        work.routing.sources_examined += 1;
        derived += usize::from(source.derived);
    }
    let literal_component = derived == 0 && model.typed_literals.is_some();
    let roles = if literal_component {
        &model.typed_literals
    } else {
        &model.typed_roles
    };
    // Old shared artifacts retain their behavior; dedicated literal parameters
    // never replace the accepted computed-result component.

    let literal = derived == 0
        && !values.sources.is_empty()
        && roles.as_ref().is_some_and(|b| b.literal_answers);
    if derived == 0 && !literal {
        return None;
    }
    let depths =
        if derived >= 2 || (derived >= 1 && model.typed_role_refinement.is_some()) || literal {
            roles.as_ref().map(|roles| {
                block = roles;
                lineage_depths(values, work)
            })
        } else {
            None
        };
    let origins =
        (depths.is_some() && block.canonical_copy_aliases).then(|| copy_origins(values, work));
    Some(TypedContext {
        literal_component,
        addresses: addresses(block, values, work),
        depths,
        origins,
        provenance: block
            .operand_provenance
            .then(|| operand_provenance(values, work)),
    })
}

pub(super) fn addresses(
    block: &TypedRouting,
    values: &ValueState,
    work: &mut ValueWork,
) -> [u32; 16] {
    let mut out = [0; 16];
    let Some(words) = &values.lexemes else {
        return out;
    };
    for (i, word) in words.queries[..words.query_len].iter().enumerate() {
        work.routing.context_tokens_read += 1;
        if block.local_query {
            work.routing.comparisons += 1;
            work.routing.logical_bytes_read += 16;
            if values.query_boundary.is_some_and(|start| word.end < start) {
                continue;
            }
        }
        let found = block.dictionary.binary_search_by(|d| {
            work.routing.comparisons += 1;
            for j in 0..usize::from(word.len.min(d.len)) {
                work.lexical_byte_comparisons += 1;
                work.routing.logical_bytes_read += 2;
                let byte = if block.fold_ascii_case {
                    word.bytes[j].to_ascii_lowercase()
                } else {
                    word.bytes[j]
                };
                let cmp = d.bytes[j].cmp(&byte);
                if !cmp.is_eq() {
                    return cmp;
                }
            }
            d.len.cmp(&word.len)
        });
        out[i] = found.map_or(0, |i| block.dictionary[i].prime);
    }
    out
}

#[cfg(test)]
pub(super) fn features(
    values: &ValueState,
    operands: Option<(ValueRecord, ValueRecord)>,
    addr: &[u32; 16],
    work: &mut ValueWork,
) -> ([ValueFeature; 36], usize) {
    features_with_depths(values, operands, addr, None, work)
}

pub(super) fn features_with_depths(
    values: &ValueState,
    operands: Option<(ValueRecord, ValueRecord)>,
    addr: &[u32; 16],
    depths: Option<&[u8; 16]>,
    work: &mut ValueWork,
) -> ([ValueFeature; 36], usize) {
    let mut out = [ValueFeature::default(); 36];
    let (flags, rank_a, rank_b) = if let Some((a, b)) = operands {
        let mut ra = VALUES;
        let mut rb = VALUES;
        for (rank, r) in values.sources.iter().rev().enumerate() {
            work.routing.sources_examined += 1;
            work.routing.comparisons += 2;
            if r.id == a.id {
                ra = rank;
            }
            if r.id == b.id {
                rb = rank;
            }
        }
        (u64::from(a.derived) | (u64::from(b.derived) << 1), ra, rb)
    } else {
        (0, VALUES, VALUES)
    };
    out[0] = ValueFeature {
        kind: 0,
        a: flags,
        b: 0,
    };
    out[1] = ValueFeature {
        kind: 1,
        a: rank_a as u64,
        b: rank_b as u64,
    };
    let mut n = 2;
    if let Some(depths) = depths {
        let mut pair = [255_u64; 2];
        if let Some((a, b)) = operands {
            for (i, r) in values.sources.iter().enumerate() {
                work.routing.sources_examined += 1;
                work.routing.comparisons += 2;
                if r.id == a.id {
                    pair[0] = u64::from(depths[i]);
                }
                if r.id == b.id {
                    pair[1] = u64::from(depths[i]);
                }
            }
            // Retain recency for literals only; derived candidates use lineage.
            if a.derived {
                out[1].a = VALUES as u64;
            }
            if b.derived {
                out[1].b = VALUES as u64;
            }
        }
        out[n] = ValueFeature {
            kind: 4,
            a: pair[0],
            b: pair[1],
        };
        n += 1;
    }
    // Ordered exact word identities, not distances computed from hash bits.
    for (position, &prime) in addr.iter().enumerate() {
        if prime == 0 {
            continue;
        }
        out[n] = ValueFeature {
            kind: 2,
            a: u64::from(prime),
            b: 0,
        };
        n += 1;
        out[n] = ValueFeature {
            kind: 3,
            a: position as u64,
            b: u64::from(prime),
        };
        n += 1;
    }
    (out, n)
}

pub(super) fn features_with_provenance(
    values: &ValueState,
    operands: Option<(ValueRecord, ValueRecord)>,
    addr: &[u32; 16],
    depths: Option<&[u8; 16]>,
    provenance: Option<&[[u8; 16]; 16]>,
    work: &mut ValueWork,
) -> ([ValueFeature; 52], usize) {
    let (base, mut n) = features_with_depths(values, operands, addr, depths, work);
    let mut out = [ValueFeature::default(); 52];
    out[..n].copy_from_slice(&base[..n]);
    if let Some(provenance) = provenance {
        let mut pair = [[0_u8; 16]; 2];
        if let Some((a, b)) = operands {
            for (i, r) in values.sources.iter().enumerate() {
                work.routing.sources_examined += 1;
                work.routing.comparisons += 2;
                work.routing.logical_bytes_read += 24;
                if r.id == a.id {
                    pair[0] = provenance[i];
                    work.routing.logical_bytes_read += 16;
                }
                if r.id == b.id {
                    pair[1] = provenance[i];
                    work.routing.logical_bytes_read += 16;
                }
            }
        }
        for q in 0..16 {
            out[n] = ValueFeature {
                kind: 5,
                a: q as u64,
                b: u64::from(pair[0][q]) | (u64::from(pair[1][q]) << 5),
            };
            n += 1;
        }
    }
    (out, n)
}

pub(super) fn score(
    model: &Model,
    values: &ValueState,
    operands: Option<(ValueRecord, ValueRecord)>,
    action: usize,
    context: &TypedContext,
    control: Control,
    work: &mut ValueWork,
) -> i64 {
    let Some(block) = (if context.literal_component {
        &model.typed_literals
    } else if context.depths.is_some() {
        &model.typed_roles
    } else {
        &model.typed_routing
    }) else {
        return 0;
    };
    let (f, n) = features_with_provenance(
        values,
        operands,
        &context.addresses,
        context.depths.as_ref(),
        context.provenance.as_ref(),
        work,
    );
    let router = if !context.literal_component
        && context.depths.is_some()
        && matches!(
            control,
            Control::MixedOperatorsDisabled | Control::MixedInitialDisabled
        ) {
        model
            .mixed_operators
            .as_ref()
            .map_or(&block.router, |w| &w.previous_roles)
    } else if context.literal_component && control == Control::InstructionBindingDisabled {
        model
            .instruction_binding
            .as_ref()
            .map_or(&block.router, |w| &w.previous_literals)
    } else if context.literal_component && control == Control::LiteralRefinementDisabled {
        model
            .literal_routing_refinement
            .as_ref()
            .map_or(&block.router, |w| &w.previous)
    } else {
        &block.router
    };
    let state = router.encode(model, &f[..n], control, &mut work.routing);
    router.score(model, state, action, &mut work.routing)
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END
