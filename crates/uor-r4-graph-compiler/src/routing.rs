use crate::induction::Cover;
use crate::induction::Observation;
use uor_r4_graph_format::OP_HALT;

pub fn synthesize_routing_program(_cover: &Cover, _observations: &[Observation]) -> Vec<u8> {
    vec![OP_HALT]
}
