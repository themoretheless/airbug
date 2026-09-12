use rbench::*;
fn main() -> Result<()> {
    let mut recorder = Recorder::new();
    recorder.case(Case {
        id: "application/frame".into(),
        contract: std::collections::BTreeMap::from([(
            "fixture".into(),
            "demonstration-no-gpu".into(),
        )]),
        metrics: vec![
            Metric::duration(
                "cpu",
                "CPU fixture only; no GPU/present",
                "individual operation",
            ),
            Metric::duration("gpu", "GPU pass", "individual pass"),
        ],
    })?;
    for _ in 0..30 {
        let start = std::time::Instant::now();
        std::hint::black_box((0..1000u64).sum::<u64>());
        recorder.observe("application/frame", "cpu", start.elapsed().as_nanos())?;
    }
    recorder.unavailable(
        "application/frame",
        "gpu",
        Availability::Unsupported("this example does not create a GPU adapter".into()),
    )?;
    let run = recorder.finish()?;
    println!("RBENCH_RESULT={}", serde_json::to_string(&run)?);
    Ok(())
}
