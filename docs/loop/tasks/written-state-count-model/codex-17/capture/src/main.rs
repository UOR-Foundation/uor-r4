fn main() -> uor_value_rank_capture::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.as_slice() == ["synthetic"] {
        serde_json::to_writer(
            std::io::stdout().lock(),
            &uor_value_rank_capture::synthetic_fixture()?,
        )?;
        println!();
        return Ok(());
    }
    uor_value_rank_capture::capture(&args)
}
