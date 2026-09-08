//! Learned numeric-versus-lexical admission before exact payload execution.
//! The inherited numeric ranking and source/NoRead selector remain unchanged.
use super::source_routing::SourceRouting;
use super::typed_routing::TypedContext;
use super::value_types::{ValueFeature, ValueRecord, ValueState, ValueWork};
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JointAdmission {
    pub router: SourceRouting,
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// Exact support predicate, without constructing a result or numeral. This
/// preserves the parent's first-valid proposal before applying learned admission.
pub(super) fn legal(action: ValueAction, a: i64, b: i64) -> bool {
    match action {
        ValueAction::Copy => true,
        ValueAction::Add => !((b > 0 && a > i64::MAX - b) || (b < 0 && a < i64::MIN - b)),
        ValueAction::Sub => a.checked_sub(b).is_some(),
    }
}

pub(super) fn features(
    values: &ValueState,
    action: ValueAction,
    operands: (ValueRecord, ValueRecord),
    margin: i64,
    context: &TypedContext,
    work: &mut ValueWork,
) -> ([ValueFeature; 54], usize) {
    let (base, n) = super::typed_routing::features_with_provenance(
        values,
        Some(operands),
        &context.addresses,
        context.depths.as_ref(),
        context.provenance.as_ref(),
        work,
    );
    let mut out = [ValueFeature::default(); 54];
    out[..n].copy_from_slice(&base[..n]);
    out[n] = ValueFeature {
        kind: 6,
        a: match action {
            ValueAction::Copy => 0,
            ValueAction::Add => 1,
            ValueAction::Sub => 2,
        },
        b: 0,
    };
    // A categorical margin from the frozen numeric selector, not a comparison
    // with independently fitted word scores and not a numeric payload feature.
    out[n + 1] = ValueFeature {
        kind: 7,
        a: margin.clamp(0, 127) as u64,
        b: 0,
    };
    (out, n + 2)
}

pub(super) fn permits(
    model: &Model,
    values: &ValueState,
    action: ValueAction,
    operands: (ValueRecord, ValueRecord),
    margin: i64,
    context: &TypedContext,
    control: Control,
    work: &mut ValueWork,
) -> bool {
    let Some(gate) = &model.joint_admission else {
        return true;
    };
    // This revision addresses literal-versus-word interference. Derived roles
    // retain the complete earlier decision path and pay no admission work.
    if !context.literal_component || control == Control::JointAdmissionDisabled {
        return true;
    }
    let (f, n) = features(values, action, operands, margin, context, work);
    work.admission_decisions += 1;
    work.routing.predictions += 1;
    let pose = gate
        .router
        .encode(model, &f[..n], control, &mut work.routing);
    let numeric = gate.router.score(model, pose, 0, &mut work.routing);
    let lexical = gate.router.score(model, pose, 1, &mut work.routing);
    work.routing.comparisons += 1;
    if lexical > numeric {
        work.admission_rejections += 1;
        false
    } else {
        true
    }
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admission_legality_matches_exact_add_and_sub_at_boundaries() {
        for a in [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX] {
            for b in [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX] {
                assert_eq!(legal(ValueAction::Add, a, b), a.checked_add(b).is_some());
                assert_eq!(legal(ValueAction::Sub, a, b), a.checked_sub(b).is_some());
                assert!(legal(ValueAction::Copy, a, b));
            }
        }
    }
}
