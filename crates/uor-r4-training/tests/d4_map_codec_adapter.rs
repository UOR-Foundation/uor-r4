use candle_core::Device;
use std::sync::Arc;
use uor_r4_training::geometric_stack::{
    D11Interim, D4Grouped4BitAdapter, HeadCompensatedMapCodec, MapCodec, ReadScore, StackArch,
    StackConfig, StackModel,
};
use uor_r4_training::stack_export::check_export_representation;
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
