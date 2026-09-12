use rbench::*;
fn main() -> Result<()> {
    let mut recorder = Recorder::new();
    recorder.case(Case {
        id: "pipeline".into(),
        contract: Default::default(),
        metrics: vec![],
    })?;
    recorder.phase("pipeline", "generate", "seeded CPU data generation")?;
    recorder.phase("pipeline", "checksum", "CPU checksum")?;
    let data = recorder.measure("pipeline", "generate", || Seeded::new(42).bytes(4096))?;
    let checksum = recorder.measure("pipeline", "checksum", || {
        data.iter().map(|b| u64::from(*b)).sum::<u64>()
    })?;
    std::hint::black_box(checksum);
    println!(
        "RBENCH_RESULT={}",
        serde_json::to_string(&recorder.finish()?)?
    );
    Ok(())
}
