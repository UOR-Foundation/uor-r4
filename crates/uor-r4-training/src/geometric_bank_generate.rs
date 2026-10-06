//! Target-free composition of actual full-bank Copy and native Generate.
//!
//! Both branches consume the same authentic causal ContextQ4Output. Generate
//! reads its final retained H4 state and full120 POSTSTATE logits, never observed
//! address roots. All source candidates retain occurrence order/provenance.
//! Only the new Copy+Generate pool supplies emission mass; old Period/Stop
//! reductions retained in the Copy preparation trace are discarded diagnostics
//! and real computational overhead, not extra action mass or a hidden bonus.
//!
//! Frozen Copy operators route through their factual native choices. Their
//! existing stopped-route/local-choice and earlier four-coordinate recurrence
//! limitations remain. This API establishes integration, not learned language,
//! geometric advantage, energy savings or durable memory consumption.

use candle_core::{DType, Tensor};
use uor_r4_integer::{
    geometric_cue_carrier::NativeCueCarrier,
    geometric_prefix_transport::NativePrefixTransport,
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::SourceBankSegment,
    geometric_vocabulary_actions::{NativeVocabularyActions, VocabularyActionTrace},
    h4_tables::{H4Code, ROOT_COUNT},
};

use crate::{
    geometric_context::ContextQ4Output,
    geometric_generate_learning::{
        vocabulary_marginal_loss, GenerateLearningOutput, GenerateLearningWeights,
        PreparedGenerateLearning,
    },
    geometric_occurrence_consumer::source_realizer::{
        ComposedCopyBankOutput, PreparedSourceRealizer,
    },
    invalid, Result,
};

pub const CREDIT_SCOPE:&str="same-actual-fullbank-context;final-retained-H4+full120-POSTSTATE-Generate;frozen-native-allsource-Copy-context/cue/prefix-credit;one-common-clipped-fullvocab-token-alias-marginal;no-old-terminals-or-bonus;all-Copy-occurrences-scored-without-selected-winner;earlier-recurrence-ambient4-remains/2";
pub const NO_SOURCE_CREDIT_SCOPE:&str="actual-causal-history-context;final-retained-H4+full120-POSTSTATE-Generate;full-legal-vocabulary;zero-Copy-occurrences;no-fabricated-source-or-initial-token;earlier-recurrence-ambient4-remains/1";

pub struct PreparedBankGenerate<'a, 'source> {
    realizer: &'a PreparedSourceRealizer<'source>,
    generate: &'a GenerateLearningWeights,
    prepared_generate: &'a PreparedGenerateLearning,
    pool: NativeVocabularyActions,
}

/// Target-free result. All native scores and the complete legal action pool
/// exist before `loss` may inspect a target. Context ownership is unambiguous:
/// source banks keep it inside Copy; no-source continuations retain it directly.
pub struct BankGenerateOutput {
    pub actions: VocabularyActionTrace,
    pub generate: GenerateLearningOutput,
    pub copy: Option<ComposedCopyBankOutput>,
    pub no_source_context: Option<ContextQ4Output>,
    pub causal_token_ids: Vec<u32>,
    pub final_state_codes: Vec<H4Code>,
    pub copy_token_ids: Vec<u32>,
    pub copy_scores_q24: Vec<i64>,
    pub credit_scope: &'static str,
}

fn same_binding(a: &SourceActionBinding, b: &SourceActionBinding) -> bool {
    a.tokenizer_sha256() == b.tokenizer_sha256()
        && a.protocol() == b.protocol()
        && a.vocab_size() == b.vocab_size()
        && a.eos_token_id() == b.eos_token_id()
        && a.period_token_id() == b.period_token_id()
}

impl<'a, 'source> PreparedBankGenerate<'a, 'source> {
    pub fn new(
        realizer: &'a PreparedSourceRealizer<'source>,
        generate: &'a GenerateLearningWeights,
        prepared_generate: &'a PreparedGenerateLearning,
        canonical_exp_bytes: &[u8],
    ) -> Result<Self> {
        if !same_binding(realizer.binding(), generate.binding())
            || prepared_generate.native.metadata().tokenizer_sha256
                != generate.binding().tokenizer_sha256()
            || prepared_generate.native.metadata().dialogue_protocol
                != *generate.binding().protocol()
            || prepared_generate.native.vocab_size() != generate.vocab_size()
            || prepared_generate.native.lanes() != generate.lanes()
        {
            return Err(invalid(
                "bank Generate tokenizer/protocol/native decoder binding differs",
            ));
        }
        let pool = NativeVocabularyActions::new(generate.binding().clone(), canonical_exp_bytes)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            realizer,
            generate,
            prepared_generate,
            pool,
        })
    }

    /// Query/prefix/context are causal state inputs, never source candidates.
    /// The old Copy preparer intentionally rejects empty/context-only banks;
    /// use explicit `forward_no_source` instead of inventing a Source record.
    pub fn forward_bank(
        &mut self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
    ) -> Result<BankGenerateOutput> {
        let copy = self.realizer.forward_bank_composed_copy(
            segments,
            query,
            actual_prefix,
            cue,
            prefix,
        )?;
        let bank = &copy.trace.cue_bank.bank;
        let time = copy.context.trace.time;
        if copy.context.trace.batch != 1
            || bank.context.tokens.len() != time
            || bank.context.heads != copy.context.trace.heads
            || bank.context.lanes_per_head != copy.context.trace.lanes_per_head
            || bank.context.states != copy.context.trace.states
            || bank.context.actions != copy.context.trace.actions
            || bank.heads.len() != copy.context.trace.heads
            || bank
                .heads
                .iter()
                .any(|h| h.scores_q24.len() != bank.candidates.len())
            || bank.candidates.is_empty()
            || copy.copy_raw.dims() != [bank.candidates.len()]
            || !copy.copy_raw.device().same_device(self.generate.device())
        {
            return Err(invalid(
                "bank Generate Copy/context replay shape/device differs",
            ));
        }
        let mut ids = Vec::with_capacity(bank.candidates.len());
        let mut scores = Vec::with_capacity(bank.candidates.len());
        for (j, candidate) in bank.candidates.iter().enumerate() {
            if candidate.bank_index != j
                || candidate.context_position >= time
                || bank.context.tokens[candidate.context_position] != candidate.occurrence.token_id
                || !self
                    .generate
                    .binding()
                    .admits_token(candidate.occurrence.token_id)
            {
                return Err(invalid(
                    "bank Generate candidate occurrence/token order differs",
                ));
            }
            let score = bank.heads.iter().try_fold(0i64, |sum, head| {
                let value = head
                    .scores_q24
                    .get(j)
                    .ok_or_else(|| invalid("bank Generate Copy head length differs"))?;
                sum.checked_add(*value)
                    .ok_or_else(|| invalid("bank Generate Copy head sum overflow"))
            })?;
            ids.push(candidate.occurrence.token_id);
            scores.push(score);
        }
        let causal_token_ids = bank.context.tokens.clone();
        let (states, logits) = final_retained_state(&copy.context, self.generate.lanes())?;
        let generated = self.generate.forward_prepared_state_choices(
            self.prepared_generate,
            &states,
            &logits,
        )?;
        let actions = self
            .pool
            .reduce_trace(&generated.scores_q24, &ids, &scores)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(BankGenerateOutput {
            actions,
            generate: generated,
            copy: Some(copy),
            no_source_context: None,
            causal_token_ids,
            final_state_codes: states,
            copy_token_ids: ids,
            copy_scores_q24: scores,
            credit_scope: CREDIT_SCOPE,
        })
    }

    /// Ordinary causal continuation with zero admitted Copy occurrences. IDs
    /// must already contain the actual history/query/prefix in caller order.
    /// Empty input is rejected; there is no fabricated identity/source token.
    pub fn forward_no_source(&mut self, causal_ids: &[u32]) -> Result<BankGenerateOutput> {
        if causal_ids.is_empty()
            || causal_ids
                .iter()
                .any(|id| !self.generate.binding().admits_token(*id))
        {
            return Err(invalid(
                "bank Generate no-source causal token IDs are empty/invalid",
            ));
        }
        let context = self.realizer.context_output(causal_ids, false)?;
        let (states, logits) = final_retained_state(&context, self.generate.lanes())?;
        let generated = self.generate.forward_prepared_state_choices(
            self.prepared_generate,
            &states,
            &logits,
        )?;
        let actions = self
            .pool
            .reduce_trace(&generated.scores_q24, &[], &[])
            .map_err(|e| invalid(e.to_string()))?;
        Ok(BankGenerateOutput {
            actions,
            generate: generated,
            copy: None,
            no_source_context: Some(context),
            causal_token_ids: causal_ids.to_vec(),
            final_state_codes: states,
            copy_token_ids: vec![],
            copy_scores_q24: vec![],
            credit_scope: NO_SOURCE_CREDIT_SCOPE,
        })
    }
}

impl BankGenerateOutput {
    pub fn context(&self) -> Result<&ContextQ4Output> {
        match (&self.copy, &self.no_source_context) {
            (Some(copy), None) => Ok(&copy.context),
            (None, Some(context)) => Ok(context),
            _ => Err(invalid("bank Generate context ownership differs")),
        }
    }
    /// Target enters only after the native score vectors and all legal actions
    /// have been produced. Generate and Copy aliases share the same marginal.
    pub fn loss(&self, target: u32) -> Result<Tensor> {
        vocabulary_marginal_loss(
            &self.actions,
            &self.generate.raw_scores,
            self.copy.as_ref().map(|c| &c.copy_raw),
            target,
        )
    }
}

fn final_retained_state(context: &ContextQ4Output, lanes: usize) -> Result<(Vec<H4Code>, Tensor)> {
    let t = &context.trace;
    let width = t
        .heads
        .checked_mul(t.lanes_per_head)
        .ok_or_else(|| invalid("bank Generate lane width overflow"))?;
    if t.batch != 1
        || t.time == 0
        || width != lanes
        || t.states.len() != t.time
        || t.states.iter().any(|s| s.len() != width)
        || context.state_choices.dims() != [1, t.time, t.heads, t.lanes_per_head, ROOT_COUNT]
        || context.latent_roots.dims() != [1, t.time, t.heads, t.lanes_per_head, 4]
        || context.state_choices.dtype() != DType::F32
        || !context
            .state_choices
            .device()
            .same_device(context.latent_roots.device())
    {
        return Err(invalid(
            "bank Generate actual retained context shape differs",
        ));
    }
    let state = t
        .states
        .last()
        .ok_or_else(|| invalid("bank Generate retained final state absent"))?
        .iter()
        .map(|r| H4Code::try_from(*r).map_err(|e| invalid(e.to_string())))
        .collect::<Result<Vec<_>>>()?;
    let logits = context
        .state_choices
        .narrow(1, t.time - 1, 1)?
        .reshape((width, ROOT_COUNT))?
        .contiguous()?;
    Ok((state, logits))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_context::NativeContextTrace;
    use candle_core::{Device, Var};
    use uor_r4_integer::{
        geometric_no_read::CANONICAL_BASIS_Q25, geometric_potential::AddressLane,
    };
    #[test]
    fn bank_generate_final_retained_state_is_last_time_head_major_and_not_observation() -> Result<()>
    {
        let choices = Var::zeros((1, 2, 2, 2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let states = vec![vec![2u8, 3, 4, 5], vec![6, 7, 8, 9]];
        let latent = states
            .iter()
            .flatten()
            .flat_map(|r| CANONICAL_BASIS_Q25[usize::from(*r)].map(|x| x as f32 / 33_554_432.))
            .collect::<Vec<_>>();
        let context = ContextQ4Output {
            state_logits: choices.as_tensor().clone(),
            state_choices: choices.as_tensor().clone(),
            latent_roots: Tensor::from_vec(latent, (1, 2, 2, 2, 4), &Device::Cpu)?,
            root_logits: Tensor::zeros((1, 2, 2, 2, ROOT_COUNT), DType::F32, &Device::Cpu)?,
            category_logits: Tensor::zeros((1, 2, 2, 2, 33), DType::F32, &Device::Cpu)?,
            trace: NativeContextTrace {
                batch: 1,
                time: 2,
                heads: 2,
                lanes_per_head: 2,
                states,
                actions: vec![vec![1; 4]; 2],
                emitted_roots: vec![1; 8],
                categories: vec![0; 8],
                codes: vec![AddressLane::new(1, 0, false).map_err(|e| invalid(e.to_string()))?; 8],
                coefficient_reads: 0,
            },
        };
        let (codes, final_choices) = final_retained_state(&context, 4)?;
        assert_eq!(
            codes.iter().map(|r| r.index()).collect::<Vec<_>>(),
            vec![6, 7, 8, 9]
        );
        let gradient = final_choices.sum_all()?.backward()?;
        let values = gradient
            .get(choices.as_tensor())
            .ok_or_else(|| invalid("final-state test missing choice gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(values[..4 * ROOT_COUNT].iter().all(|x| *x == 0.));
        assert!(values[4 * ROOT_COUNT..].iter().all(|x| *x == 1.));
        assert!(final_retained_state(&context, 8).is_err());
        Ok(())
    }
}
