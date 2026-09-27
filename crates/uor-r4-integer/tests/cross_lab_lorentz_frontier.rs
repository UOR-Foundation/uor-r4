use std::path::Path;
use uor_r4_integer::{IntegerModel, ReadGeometry, ReadMode};

const LORENTZ_MODEL_PATH: &str =
    "../../docs/evidence/native-lorentz-packet-2026-09-26/packed/qat-lorentzflat_s1";
const DOT_MODEL_PATH: &str =
    "../../docs/evidence/native-lorentz-packet-2026-09-26/packed/qat-dot_s1";
const TABLES_PATH: &str = "../../docs/evidence/native-lorentz-packet-2026-09-26/packed/tables";
const TOTAL: u64 = 1u64 << 48;

#[test]
fn test_cross_lab_lorentz_loading_and_inference() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model_path = manifest_dir.join(LORENTZ_MODEL_PATH);
    let tables_path = manifest_dir.join(TABLES_PATH);

    if !model_path.exists() || !tables_path.exists() {
        eprintln!("Skipping: lorentz packet not present at {model_path:?}");
        return;
    }

    let model = IntegerModel::load_with_tables(&model_path, &tables_path)
        .expect("must load qat-lorentzflat_s1 with arcosh tables");

    assert_eq!(model.config().read_geometry, ReadGeometry::Lorentz);
    assert_eq!(model.config().width, 128);
    assert_eq!(model.config().context, 256);

    let mut session = model.new_session();
    let step1 = model
        .step(&mut session, 42, ReadMode::Enabled)
        .expect("step 1 with lorentz read enabled");

    let sum1: u64 = step1.probabilities.iter().sum();
    assert_eq!(sum1, TOTAL, "probabilities must sum to 1.0 (Q48 TOTAL)");

    let step2 = model
        .step(&mut session, 100, ReadMode::Enabled)
        .expect("step 2 with lorentz read enabled");

    let sum2: u64 = step2.probabilities.iter().sum();
    assert_eq!(sum2, TOTAL, "probabilities must sum to 1.0 (Q48 TOTAL)");
    assert_eq!(session.len(), 2);

    // Causal memory ablation on Lorentz model: NoRead must disable read masses
    let step_noread = model
        .step(&mut session, 200, ReadMode::NoRead)
        .expect("step with NoRead");
    assert_eq!(step_noread.no_read_mass, TOTAL);
    assert!(step_noread.read_masses.iter().all(|&m| m == 0));
}

#[test]
fn test_cross_lab_dot_loading_and_inference() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model_path = manifest_dir.join(DOT_MODEL_PATH);
    let tables_path = manifest_dir.join(TABLES_PATH);

    if !model_path.exists() || !tables_path.exists() {
        eprintln!("Skipping: dot packet not present at {model_path:?}");
        return;
    }

    let model = IntegerModel::load_with_tables(&model_path, &tables_path)
        .expect("must load qat-dot_s1 with tables");

    assert_eq!(model.config().read_geometry, ReadGeometry::Dot);
    assert_eq!(model.config().width, 128);
    assert_eq!(model.config().context, 256);

    let mut session = model.new_session();
    let step = model
        .step(&mut session, 42, ReadMode::Enabled)
        .expect("step with dot read enabled");

    let sum: u64 = step.probabilities.iter().sum();
    assert_eq!(sum, TOTAL, "probabilities must sum to 1.0 (Q48 TOTAL)");
}

#[test]
fn test_cross_lab_lorentz_causal_nll_advantage() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model_path = manifest_dir.join(LORENTZ_MODEL_PATH);
    let tables_path = manifest_dir.join(TABLES_PATH);

    if !model_path.exists() || !tables_path.exists() {
        return;
    }

    let model = IntegerModel::load_with_tables(&model_path, &tables_path)
        .expect("must load qat-lorentzflat_s1");

    // Sequence of repeated/pattern tokens to give memory read an active causal signal
    let sequence: Vec<u32> = vec![
        10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 10, 20,
        30, 40, 50, 60, 70, 80, 90, 100,
    ];

    // Compute NLL under ReadMode::Enabled
    let mut session_read = model.new_session();
    let mut nll_read = 0.0f64;
    for i in 0..sequence.len() - 1 {
        let current = sequence[i];
        let target = sequence[i + 1] as usize;
        let step = model
            .step(&mut session_read, current, ReadMode::Enabled)
            .expect("step read");
        let prob = (step.probabilities[target] as f64) / (TOTAL as f64);
        let clipped = prob.max(1e-12);
        nll_read -= clipped.ln();
    }

    // Compute NLL under ReadMode::NoRead
    let mut session_noread = model.new_session();
    let mut nll_noread = 0.0f64;
    for i in 0..sequence.len() - 1 {
        let current = sequence[i];
        let target = sequence[i + 1] as usize;
        let step = model
            .step(&mut session_noread, current, ReadMode::NoRead)
            .expect("step noread");
        let prob = (step.probabilities[target] as f64) / (TOTAL as f64);
        let clipped = prob.max(1e-12);
        nll_noread -= clipped.ln();
    }

    let avg_nll_read = nll_read / ((sequence.len() - 1) as f64);
    let avg_nll_noread = nll_noread / ((sequence.len() - 1) as f64);
    let delta_nll = avg_nll_noread - avg_nll_read;

    println!(
        "Lorentz Causal Ablation: Read NLL = {avg_nll_read:.4}, NoRead NLL = {avg_nll_noread:.4}, Delta NLL = {delta_nll:.4} nats"
    );

    // Memory read must improve or at least match prediction on repeated sequence
    assert!(
        delta_nll >= 0.0,
        "Lorentz memory read must provide positive causal predictive benefit"
    );
}

#[test]
fn test_cross_lab_lorentz_conversational_step() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model_path = manifest_dir.join(LORENTZ_MODEL_PATH);
    let tables_path = manifest_dir.join(TABLES_PATH);

    if !model_path.exists() || !tables_path.exists() {
        eprintln!("Skipping: lorentz packet not present at {model_path:?}");
        return;
    }

    let model = IntegerModel::load_with_tables(&model_path, &tables_path)
        .expect("must load qat-lorentzflat_s1 with arcosh tables");

    assert_eq!(model.config().read_geometry, ReadGeometry::Lorentz);
    assert_eq!(model.config().width, 128);

    let mut session = model.new_conversational_session();

    // 1. Persistent persona ingestion
    model
        .step_conversational_into(
            &mut session,
            42,
            uor_r4_integer::SlotTarget::Persistent,
            ReadMode::Enabled,
        )
        .expect("step persistent persona with Lorentz model");
    assert_eq!(session.persistent_len(), 1);
    assert_eq!(session.persistent_key_norms.len(), 1);
    assert!(
        session.persistent_key_norms[0] > 0,
        "Lorentz key norm must be positive"
    );

    session.seal_persistent();
    assert!(session.is_persistent_sealed());

    // 2. Dialogue slot step
    session.start_turn();
    model
        .step_conversational_into(
            &mut session,
            100,
            uor_r4_integer::SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .expect("step dialogue turn with Lorentz model");
    assert_eq!(session.dialogue_len(), 1);
    assert!(
        session.dialogue_key_norms[0] > 0,
        "Lorentz dialogue key norm must be positive"
    );

    // 3. Probabilities validity
    let p_sum: u64 = session.last_probabilities()[..model.config().vocab_size]
        .iter()
        .sum();
    assert_eq!(p_sum, TOTAL, "probabilities must sum to 1.0 (TOTAL)");

    // 4. NoRead step on Lorentz model in conversational mode
    model
        .step_conversational_into(
            &mut session,
            105,
            uor_r4_integer::SlotTarget::Dialogue,
            ReadMode::NoRead,
        )
        .expect("step dialogue NoRead with Lorentz model");
    assert_eq!(session.last_no_read_mass(), TOTAL);
    assert!(session.last_read_masses().iter().all(|&m| m == 0));
}
