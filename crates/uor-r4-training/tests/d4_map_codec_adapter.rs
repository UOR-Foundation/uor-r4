use candle_core::Device;
use std::sync::Arc;
use uor_r4_integer::stack::IntegerStackModel;
use uor_r4_training::geometric_stack::{
    D11Interim, D4Grouped4BitAdapter, HeadCompensatedMapCodec, MapCodec, ReadScore, StackAdamW,
    StackArch, StackConfig, StackModel,
};
use uor_r4_training::stack_export::{check_export_representation, export_stack};
use uor_r4_training::Result;

#[test]
fn native_grouped_4bit_codec_matches_d11_interim_bit_for_bit() -> Result<()> {
    let values: Vec<f32> = (0..64 * 32)
        .map(|i| (((i as f32) * 0.17) % 7.0 - 3.5) * 0.1)
        .collect();
    let d11 = D11Interim;
    let native_rtn = D4Grouped4BitAdapter::rtn();

    let rt_d11 = d11.round_trip(&values, 64, 32)?;
    let rt_native = native_rtn.round_trip(&values, 64, 32)?;
    assert_eq!(
        rt_d11, rt_native,
        "Native Grouped4BitCodec via D4Grouped4BitAdapter must match D11Interim round-trip bit for bit"
    );

    let dyn_codec: Arc<dyn MapCodec> = Arc::new(native_rtn);
    assert_eq!(dyn_codec.name(), "native-d11-grouped-4bit-g32-rtn");
    let rt_dyn = dyn_codec.round_trip(&values, 64, 32)?;
    assert_eq!(rt_dyn, rt_d11);

    // Test with MinimumMseScale mode as well
    let native_opt = D4Grouped4BitAdapter::min_mse();
    let dyn_opt: Arc<dyn MapCodec> = Arc::new(native_opt);
    assert_eq!(dyn_opt.name(), "native-d11-grouped-4bit-g32-min-mse");
    let rt_opt = dyn_opt.round_trip(&values, 64, 32)?;
    assert_eq!(rt_opt.len(), 64 * 32);

    Ok(())
}

#[test]
fn head_compensated_codec_produces_finite_values() -> Result<()> {
    let values: Vec<f32> = (0..64 * 32)
        .map(|i| (((i as f32) * 0.31) % 5.0 - 2.5) * 0.1)
        .collect();
    let codec_all = HeadCompensatedMapCodec::all_maps();
    assert_eq!(codec_all.name(), "native-d4-head-compensated-all-maps");
    let rt_all = codec_all.round_trip(&values, 64, 32)?;
    assert_eq!(rt_all.len(), 64 * 32);
    assert!(rt_all.iter().all(|v| v.is_finite()));

    let codec_head = HeadCompensatedMapCodec::head_only();
    assert_eq!(codec_head.name(), "native-d4-head-compensated-head-only");
    // When rows <= cols, head_only falls back to D11Interim:
    let rt_head = codec_head.round_trip(&values, 32, 64)?;
    let rt_d11 = D11Interim.round_trip(&values, 32, 64)?;
    assert_eq!(rt_head, rt_d11);

    // Test explicit target shape with head_only_for:
    let codec_targeted = HeadCompensatedMapCodec::head_only_for(64, 32);
    let rt_targeted_match = codec_targeted.round_trip(&values, 64, 32)?;
    let rt_comp = uor_r4_integer::codec::apply_codec_arm(
        &values,
        64,
        32,
        uor_r4_integer::codec::CodecArm::HeadCompensated,
        0,
    )?;
    assert_eq!(rt_targeted_match, rt_comp);

    // Non-matching shape falls back to D11Interim:
    let rt_targeted_nonmatch = codec_targeted.round_trip(&values, 32, 64)?;
    assert_eq!(rt_targeted_nonmatch, rt_d11);

    Ok(())
}

#[test]
fn qat_adapters_integrate_with_stack_model_and_export_contract() -> Result<()> {
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 96,
        width: 64,
        heads: 2,
        mlp_hidden: 40,
        context: 16,
        pattern: "rar".into(),
        read: ReadScore::Lorentz,
        rotation: true,
        seed: 29,
        memory: None,
    };
    let device = Device::Cpu;
    let mut model = StackModel::new(config, &device)?;

    // 1. Set served representation with D4Grouped4BitAdapter
    model.set_served_representation(Some(Arc::new(D4Grouped4BitAdapter::rtn())))?;
    assert_eq!(
        model.served_codec().unwrap().name(),
        "native-d11-grouped-4bit-g32-rtn"
    );

    let tmp_root = std::env::temp_dir().join(format!("d4-qat-test-{}", std::process::id()));
    let save_dir = tmp_root.join("saved_model");
    model.save(&save_dir)?;

    let record = StackModel::saved_served_representation(&save_dir)?;
    assert_eq!(
        record.as_ref().unwrap().codec,
        "native-d11-grouped-4bit-g32-rtn"
    );

    // Verify export representation check passes for interim-compatible QAT model
    check_export_representation(record.as_ref(), false)?;
    // Calibrated export should be rejected for a QAT model
    assert!(check_export_representation(record.as_ref(), true).is_err());

    // 2. Set served representation with HeadCompensatedMapCodec
    model.set_served_representation(Some(Arc::new(HeadCompensatedMapCodec::head_only())))?;
    assert_eq!(
        model.served_codec().unwrap().name(),
        "native-d4-head-compensated-head-only"
    );

    // Clean up
    let _ = std::fs::remove_dir_all(&tmp_root);
    Ok(())
}

#[test]
fn qat_end_to_end_gradient_step_export_and_integer_serving_chain() -> Result<()> {
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 64,
        width: 32,
        heads: 1,
        mlp_hidden: 32,
        context: 8,
        pattern: "r".into(),
        read: ReadScore::Dot,
        rotation: false,
        seed: 42,
        memory: None,
    };
    let device = Device::Cpu;
    let mut model = StackModel::new(config, &device)?;

    // 1. Enable QAT with D4Grouped4BitAdapter
    let adapter = Arc::new(D4Grouped4BitAdapter::rtn());
    model.set_served_representation(Some(adapter))?;

    // 2. Compute loss on dummy inputs
    let ids: Vec<u32> = vec![1, 5, 12, 18];
    let targets: Vec<u32> = vec![5, 12, 18, 22];
    let loss = model.loss(&ids, &targets, 1, 4)?;
    let initial_loss = loss.to_scalar::<f32>()?;
    assert!(initial_loss.is_finite(), "initial QAT loss must be finite");

    // 3. Backward pass: gradients flow through StraightThrough op to master variables
    let grads = loss.backward()?;
    let embed_var = model.variables().get("embedding.weight").unwrap();
    let embed_grad = grads.get(embed_var.as_tensor());
    assert!(embed_grad.is_some(), "gradient must reach embedding.weight");
    assert!(
        embed_grad.unwrap().sqr()?.sum_all()?.to_scalar::<f32>()? > 0.0,
        "gradient on embedding.weight must be non-zero"
    );

    // 4. Optimizer update modifies master parameters
    let mut optimizer = StackAdamW::new(&model, 0.01, 1.0)?;
    let norm = optimizer.update(&model, &grads, 0.05)?;
    assert!(norm > 0.0, "gradient norm must be positive");

    // 5. Save model and verify metadata
    let tmp_root = std::env::temp_dir().join(format!("d4-qat-e2e-{}", std::process::id()));
    let save_dir = tmp_root.join("saved_model");
    model.save(&save_dir)?;

    let record = StackModel::saved_served_representation(&save_dir)?;
    assert_eq!(
        record.as_ref().unwrap().codec,
        "native-d11-grouped-4bit-g32-rtn"
    );

    // 6. Export stack artifact without calibration (using QAT weights)
    check_export_representation(record.as_ref(), false)?;
    let (lut_bytes, _summary) = export_stack(&model, "qat-e2e-test".into(), None)?;
    assert!(!lut_bytes.is_empty());

    let lut_path = tmp_root.join("model.lut");
    std::fs::write(&lut_path, &lut_bytes)?;

    // 7. Load into IntegerStackModel and verify native integer serving execution
    let integer_model = IntegerStackModel::load(&lut_path)
        .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
    assert_eq!(integer_model.shape().vocab, 64);
    assert_eq!(integer_model.shape().width, 32);

    let mut session = integer_model.session();
    let logits = session
        .step(ids[0])
        .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
    assert_eq!(logits.len(), 64, "logits length must match vocabulary size");
    let next_token = uor_r4_integer::stack::stack_argmax(logits);
    assert!(next_token < 64, "argmax token must be within vocabulary");

    // Clean up
    let _ = std::fs::remove_dir_all(&tmp_root);
    Ok(())
}
