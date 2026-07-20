pub fn compute_merkle_root_and_proof(
    leaves: &[&[u8]],
    target_idx: usize,
) -> Option<([u8; 32], Vec<[u8; 32]>)> {
    if leaves.is_empty() {
        return None;
    }
    let mut current_level: Vec<[u8; 32]> = leaves.iter().map(|l| blake3::hash(l).into()).collect();
    let mut proof = Vec::new();
    let mut idx = target_idx;

    while current_level.len() > 1 {
        let sibling_idx = if idx % 2 == 0 {
            if idx + 1 < current_level.len() {
                idx + 1
            } else {
                idx
            }
        } else {
            idx - 1
        };
        proof.push(current_level[sibling_idx]);

        let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
        for i in (0..current_level.len()).step_by(2) {
            let left = current_level[i];
            let right = if i + 1 < current_level.len() {
                current_level[i + 1]
            } else {
                left
            };
            let mut hasher = blake3::Hasher::new();
            hasher.update(&left);
            hasher.update(&right);
            next_level.push(hasher.finalize().into());
        }
        current_level = next_level;
        idx /= 2;
    }

    Some((current_level[0], proof))
}

pub fn verify_merkle_proof(
    root: &[u8; 32],
    leaf: &[u8],
    proof: &[[u8; 32]],
    target_idx: usize,
) -> bool {
    let mut current: [u8; 32] = blake3::hash(leaf).into();
    let mut idx = target_idx;
    for &sibling in proof {
        let mut hasher = blake3::Hasher::new();
        if idx % 2 == 0 {
            hasher.update(&current);
            hasher.update(&sibling);
        } else {
            hasher.update(&sibling);
            hasher.update(&current);
        }
        current = hasher.finalize().into();
        idx /= 2;
    }
    current == *root
}
