//! K-coordinate row-parallel scoring on the CSR bucketed writer.
//!
//! The rows-K baseline and eval kernels (`rowsk_source.rs`) with the full-scan
//! `writer` replaced by `bucket_writer` in `IdOrder` with no planted fault, the
//! walk qualified exact against the full scan in `m1-faster-writer` (#2840).
//! The records they write (`base`, `cost`, `flag`) have the rows-K layout, so
//! the unchanged rows-K reducer reads them. This is a separate module: the
//! bucket text with the relation sweep of the read rewritten to one pass per
//! sweep (the same integer sums; see `LINEAR_SWEEP_HEAD`), and two entries
//! appended. The eval kernel also reuses a row's baseline read for every lane
//! whose candidate cannot change it.
//!
//! The bucket text has no Pick read, so this module puts the marked Pick
//! lines of `discrete.cu` back, unchanged, at their two places. It admits
//! only the outer address metric; the device selects it only for such models.

pub const ENTRIES: [&str; 9] = [
    "discrete_bucket_build",
    "discrete_bucket_update",
    "discrete_bucket_writer_eval",
    "discrete_fullscan_writer_eval",
    "discrete_bucket_forward",
    "discrete_bucket_counterfactual",
    "discrete_bucket_evaluate",
    "discrete_counterfactual_rowskb_baseline",
    "discrete_counterfactual_rowskb_eval",
];

/// Append the two bucketed rows-K entries to the generated bucket source.
pub fn generate(bucket_source: &str, discrete_source: &str) -> Result<String, String> {
    let bucket_source = &restore_pick(bucket_source, discrete_source)?;
    generate_read(bucket_source)
}

const PICK_BEGIN: &str = "// PICK-BEGIN";
const PICK_END: &str = "// PICK-END\n";
const READ_DISPATCH_COMMENT: &str = "// Mode dispatch: Route (default), Flock, or Relation.";
const READ_SIGNATURE: &str = "__device__ bool read_vector(const int* input,const U* ids,const U* routes,const int* gates,const U* plan,const U* g,const U* s,const U* position_steps,U row,U kind,U index,U candidate,int* output,U* trace) {\n";

/// The two marked Pick blocks of `discrete.cu`, byte for byte: the read's
/// functions, and its dispatch line inside `read_vector`.
fn pick_blocks(discrete_source: &str) -> Result<[&str; 2], String> {
    let mut blocks = Vec::new();
    let mut rest = discrete_source;
    let mut offset = 0;
    while let Some(begin) = rest.find(PICK_BEGIN) {
        let end = rest[begin..]
            .find(PICK_END)
            .ok_or("rows-K bucket anchor changed: unterminated Pick block")?
            + begin
            + PICK_END.len();
        blocks.push(&discrete_source[offset + begin..offset + end]);
        offset += end;
        rest = &rest[end..];
    }
    match blocks[..] {
        [functions, dispatch] => Ok([functions, dispatch]),
        _ => Err("rows-K bucket anchor changed: expected two Pick blocks".into()),
    }
}

/// The bucket text with the Pick read put back where `discrete.cu` has it.
fn restore_pick(bucket_source: &str, discrete_source: &str) -> Result<String, String> {
    if bucket_source.contains("MODE_PICK") {
        return Err("rows-K bucket anchor changed: bucket text carries a Pick read".into());
    }
    let [functions, dispatch] = pick_blocks(discrete_source)?;
    if !functions.contains("__device__ bool pick_vector(") || !dispatch.contains("pick_vector(") {
        return Err("rows-K bucket anchor changed: Pick blocks differ".into());
    }
    if bucket_source.matches(READ_DISPATCH_COMMENT).count() != 1
        || bucket_source.matches(READ_SIGNATURE).count() != 1
        || discrete_source.matches(READ_SIGNATURE).count() != 1
    {
        return Err("rows-K bucket anchor changed: read dispatch differs".into());
    }
    let with_functions = bucket_source.replacen(
        READ_DISPATCH_COMMENT,
        &format!("{functions}{READ_DISPATCH_COMMENT}"),
        1,
    );
    Ok(with_functions.replacen(READ_SIGNATURE, &format!("{READ_SIGNATURE}{dispatch}"), 1))
}

fn generate_read(bucket_source: &str) -> Result<String, String> {
    if bucket_source.matches("__device__ W bucket_writer(").count() != 1 {
        return Err("rows-K bucket anchor changed: bucket_writer missing".into());
    }
    if !bucket_source.contains("extern \"C\" __global__ void discrete_bucket_build(") {
        return Err("rows-K bucket anchor changed: index build missing".into());
    }
    // The relation sweep this module replaces. If its text changes, the linear
    // sweep must be re-derived and re-reviewed rather than silently diverge.
    if bucket_source.matches(ORDINARY_SWEEP).count() != 1 {
        return Err("rows-K bucket anchor changed: relation sweep differs".into());
    }
    let sweep = format!("{LINEAR_SWEEP_HEAD}{ORDINARY_SWEEP}{LINEAR_SWEEP_TAIL}");
    let read = bucket_source.replacen(ORDINARY_SWEEP, &sweep, 1);
    Ok(format!("{LINEAR_SWEEP_DEFINES}{read}\n{ROWSKB}"))
}

/// Exact text of the relation sweep loop in `discrete.cu` (`flock_vector`).
pub const ORDINARY_SWEEP: &str = include_str!("rowskb_ordinary_sweep.txt");

// The ordinary sweep rescans every node for each node and edge type. The same
// sums in one pass per sweep:
//   edge 0 (parents) and edge 1 (siblings): each internal node p scatters into
//     its distinct children; a parent always follows its children (the plan
//     check above the read), so `p > i` holds for every parent the scan found.
//   edge 2 (left neighbour at the level) and edge 4 (latest same-frame node at
//     the level): tables entered after node i is scored, so they hold exactly
//     the nodes j < i the ordinary scan visits.
//   edge 3 (latest earlier leaf with the same token id): fixed for the row.
// Integer addition is order-free, and "the last match wins" is the latest
// table entry. A plan deeper than the tables keeps the ordinary loop.
const LINEAR_SWEEP_HEAD: &str = r#"            bool linear=s[2]<=MAX_POSITIONS;
            for(U node=0;node<nodes;++node)if(level[node]>=ROWSKB_LEVELS)linear=false;
            if(linear) {
                short same_id[MAX_NODES];
                if(gate_value(gates,s[1]*s[5]+3u,kind,index,candidate)!=0) {
                    for(U i=0;i<nodes;++i) {
                        same_id[i]=-1;
                        if(level[i]!=0 || last[i]>=length)continue;
                        U tid=ids[row*s[2]+last[i]];
                        for(U j=i;j-->0;)if(last[j]<length && level[j]==0 && ids[row*s[2]+last[j]]==tid){same_id[i]=(short)j;break;}
                    }
                }
                for(U sweep=0;sweep<sweeps;++sweep) {
                    int up[MAX_NODES],side[MAX_NODES];
                    int adjacent[ROWSKB_LEVELS*MAX_POSITIONS],same_frame[ROWSKB_LEVELS*16];
                    for(U i=0;i<nodes;++i){up[i]=0;side[i]=0;}
                    for(U i=0;i<ROWSKB_LEVELS*MAX_POSITIONS;++i)adjacent[i]=0;
                    for(U i=0;i<ROWSKB_LEVELS*16;++i)same_frame[i]=0;
                    for(U p=0;p<nodes;++p) {
                        if(plan[p*4+3]!=ABSENT || last[p]>=length)continue;
                        U c0=plan[p*4],c1=plan[p*4+1],c2=plan[p*4+2];
                        up[c0]+=prev[p];
                        if(c1!=c0)up[c1]+=prev[p];
                        if(c2!=c0 && c2!=c1)up[c2]+=prev[p];
                        ROWSKB_SIDE(c0)
                        if(c1!=c0)ROWSKB_SIDE(c1)
                        if(c2!=c0 && c2!=c1)ROWSKB_SIDE(c2)
                    }
                    for(U i=0;i<nodes;++i) {
                        if(last[i]>=length)continue;
                        U li=level[i];
                        for(U e=0;e<EDGE_TYPES;++e) {
                            int gt=gate_value(gates,s[1]*s[5]+li*EDGE_TYPES+e,kind,index,candidate);
                            if(gt==0)continue;
                            int esum=0;
                            if(e==0) {
                                if(plan[i*4+3]==ABSENT) {
                                    for(U c=0;c<3;++c){U ch=plan[i*4+c];if(last[ch]<length)esum+=prev[ch];}
                                }
                                esum+=up[i];
                            } else if(e==1) {
                                esum=side[i];
                            } else if(e==2) {
                                if(first[i]>0)esum=adjacent[li*MAX_POSITIONS+first[i]-1];
                            } else if(e==3) {
                                if(li==0 && same_id[i]>=0)esum=prev[same_id[i]];
                            } else if(e==4) {
                                esum=same_frame[li*16+frame[i]];
                            }
                            rel[i]+=gt>0?esum:-esum;
                        }
                        adjacent[li*MAX_POSITIONS+last[i]]+=prev[i];
                        same_frame[li*16+frame[i]]=prev[i];
                    }
                    for(U i=0;i<nodes;++i)prev[i]=rel[i];
                }
            } else {
"#;
const LINEAR_SWEEP_TAIL: &str = "            }\n";
const LINEAR_SWEEP_DEFINES: &str = r#"
// ---- Linear relation sweep (rows-K bucket module) ----
#define ROWSKB_LEVELS 6u
// The ordinary sibling chain for node i under parent (c0, c1, c2), verbatim.
#define ROWSKB_SIDE(i) {if((i)==c0 && last[c1]<length)side[(i)]+=prev[c1];else if((i)==c1){if(last[c0]<length)side[(i)]+=prev[c0];if(last[c2]<length)side[(i)]+=prev[c2];}else if((i)==c2 && last[c1]<length)side[(i)]+=prev[c1];}
"#;

const ROWSKB: &str = r#"
// ---- K-coordinate row-parallel scoring on the bucketed writer ----
// Same records as the rows-K kernels:
//   base[row*2] baseline terminal status, base[row*2+1] baseline eligibility;
//   cost[c*rows*16 + row*16 + lane], flag[...] low 16 bits the virtual terminal
//   status, bit 16 the virtual token-rival flag.
#define ROWSKB_TR_BIT 0x10000u
#define ROWSKB_STATUS_MASK 0xffffu
__device__ bool rowskb_refused(const U* s,U kind,U index) {
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION || s[8]==MODE_PICK;
    return (kind==GATE_KIND && !gated) || (kind==0u && gated) || (kind==GATE_KIND && index>=gate_count(s));
}
extern "C" __global__ void discrete_counterfactual_rowskb_baseline(
    const int* table,const U* routes,const int* gates,const U* ids,const int* input,
    const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,
    const U* offsets,const U* members,const U* cells,const U* absent_count,U* base,int* qbase
) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int baseline[MAX_WIDTH];U prediction,tr=0u,ar,arithmetic_status=0u;
    U row_status=0u,row_eligible=0u;
    if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2u,0u,0u,baseline,0))row_status=2u;
    else {
        W loss=bucket_writer(
            baseline,table,g,s,offsets,members,cells,absent_count,
            targets[row],2u,0u,0u,BUCKET_WALK_ID_ORDER,BUCKET_FAULT_NONE,
            &prediction,&tr,&ar,0,0,0,0,&arithmetic_status,0,0
        );
        if(loss==~(W)0)row_status=arithmetic_status?arithmetic_status:8u;
        else {row_eligible=tr!=0u;U width=s[1]*8u;for(U d=0;d<width;++d)qbase[row*width+d]=baseline[d];}
    }
    base[row*2u]=row_status;base[row*2u+1u]=row_eligible;
}
extern "C" __global__ void discrete_counterfactual_rowskb_eval(
    const int* table,const U* routes,const int* gates,const U* ids,const int* input,
    const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,
    const U* offsets,const U* members,const U* cells,const U* absent_count,
    U kind,const U* indices,U k,const U* base,const int* qbase,W* cost,U* flag
) {
    U per=s[4]*16u;U pair=blockIdx.x*blockDim.x+threadIdx.x;
    if(per==0u)return;
    U c=pair/per,rest=pair%per,row=rest/16u,candidate=rest%16u;
    if(c>=k || row>=s[4])return;
    U index=indices[c];U slot=c*per+row*16u+candidate;cost[slot]=~(W)0;flag[slot]=0u;
    if(rowskb_refused(s,kind,index) || (kind==GATE_KIND && candidate>=3u))return;
    if(base[row*2u]!=0u || base[row*2u+1u]==0u)return;
    int q[MAX_WIDTH];U prediction,tr=0u,ar,arithmetic_status=0u;
    // The read takes a candidate only through input_block (a vector move whose
    // token is one of this row's context ids) and gate_value (a gate move). A
    // lane with neither reads exactly the baseline vector stored for the row.
    bool reuse=candidate==0u && (kind==1u || kind==GATE_KIND);
    // The Pick read takes one gate per block, at block*levels; a move of any
    // other gate cannot change it.
    if(!reuse && kind==GATE_KIND && s[8]==MODE_PICK && s[5]!=0u && index%s[5]!=0u)reuse=true;
    if(!reuse && kind==1u){
        U token=index/s[1];const U* window=ids+row*s[2];reuse=true;
        for(U p=0;p<s[2];++p)if(window[p]==token){reuse=false;break;}
    }
    if(reuse){U width=s[1]*8u;for(U d=0;d<width;++d)q[d]=qbase[row*width+d];}
    else if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,kind,index,candidate,q,0)){flag[slot]=2u;return;}
    W loss=bucket_writer(
        q,table,g,s,offsets,members,cells,absent_count,
        targets[row],kind,index,candidate,BUCKET_WALK_ID_ORDER,BUCKET_FAULT_NONE,
        &prediction,&tr,&ar,0,0,0,0,&arithmetic_status,0,0
    );
    cost[slot]=loss;flag[slot]=(arithmetic_status&ROWSKB_STATUS_MASK)|(loss!=~(W)0 && tr!=0u?ROWSKB_TR_BIT:0u);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn bucket() -> String {
        crate::bucket_source::generate(include_str!("discrete.cu")).expect("bucket module")
    }

    #[test]
    fn rowskb_module_appends_exactly_two_entries_to_the_bucket_module() {
        let bucket = bucket();
        let discrete = include_str!("discrete.cu");
        let rowskb = generate(&bucket, discrete).expect("rows-K bucket module");
        // The Pick read is put back byte for byte, where discrete.cu has it:
        // its functions before the read dispatch, its dispatch line first in it.
        let [functions, dispatch] = pick_blocks(discrete).expect("pick blocks");
        assert_eq!(rowskb.matches(functions).count(), 1);
        assert!(rowskb.contains(&format!("{functions}{READ_DISPATCH_COMMENT}")));
        assert!(rowskb.contains(&format!("{READ_SIGNATURE}{dispatch}")));
        assert!(discrete.contains(&format!("{READ_SIGNATURE}{dispatch}")));
        // With those two blocks taken out again, the bucket text is carried
        // unchanged outside the relation sweep.
        let without_pick = rowskb.replacen(functions, "", 1).replacen(dispatch, "", 1);
        let at = bucket.find(ORDINARY_SWEEP).expect("ordinary sweep");
        assert!(without_pick.contains(&bucket[..at]));
        assert!(without_pick.contains(&bucket[at + ORDINARY_SWEEP.len()..]));
        // The ordinary loop stays as the fallback branch, once.
        assert_eq!(rowskb.matches(ORDINARY_SWEEP).count(), 1);
        assert_eq!(
            include_str!("discrete.cu").matches(ORDINARY_SWEEP).count(),
            1
        );
        let entries: Vec<&str> = rowskb
            .match_indices("extern \"C\" __global__ void ")
            .map(|(at, prefix)| {
                let name = &rowskb[at + prefix.len()..];
                &name[..name.find('(').expect("entry signature")]
            })
            .collect();
        assert_eq!(entries, ENTRIES);
    }

    #[test]
    fn rowskb_kernels_call_the_exact_walk_only() {
        assert_eq!(
            ROWSKB
                .matches("BUCKET_WALK_ID_ORDER,BUCKET_FAULT_NONE")
                .count(),
            2
        );
        assert!(!ROWSKB.contains("SERVING_PRUNED"));
        assert!(!ROWSKB.contains(" writer("));
    }

    #[test]
    fn changed_anchors_are_refused() {
        let discrete = include_str!("discrete.cu");
        assert!(generate("no bucket writer here", discrete).is_err());
        let bucket = bucket();
        assert!(generate(&format!("{bucket}\n// MODE_PICK"), discrete).is_err());
        // A kernel source with one Pick block, or none, is refused.
        assert!(generate(&bucket, &bucket).is_err());
    }
}
