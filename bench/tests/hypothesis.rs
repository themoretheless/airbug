use airbug_bench::{Case, Metric, Recorder, Run, analysis, hypothesis};
fn run(offset: u128, processes: u32, batches: u64) -> Run {
    let mut recorder = Recorder::new();
    for id in ["first", "second"] {
        recorder
            .case(Case {
                id: id.into(),
                contract: Default::default(),
                metrics: vec![Metric::duration("wall", "test", "batch_total")],
            })
            .unwrap();
        recorder.observe(id, "wall", offset).unwrap();
    }
    let mut run = recorder.finish().unwrap();
    let originals = run.observations.clone();
    run.observations.clear();
    for original in originals {
        for process in 0..processes {
            for sequence in 0..batches {
                let mut observation = original.clone();
                observation.process = process;
                observation.sequence = sequence;
                observation.operations = sequence + 1;
                observation.value =
                    Some(((offset + u128::from(process)) * u128::from(sequence + 1)).to_string());
                run.observations.push(observation);
            }
        }
    }
    run
}
#[test]
fn test_uses_process_units_normalizes_batches_and_reports_family_alpha() {
    let baseline = run(100, 12, 3);
    let candidate = run(300, 12, 5);
    let config = hypothesis::Config {
        resamples: 2000,
        seed: 19,
    };
    let rows =
        analysis::compare_with_hypothesis(&baseline, Some(&candidate), 5.0, 0.05, config).unwrap();
    for row in &rows {
        let test = row.hypothesis.as_ref().unwrap();
        assert_eq!(test.test.baseline_units, 12);
        assert_eq!(test.test.candidate_units, 12);
        assert_eq!(test.significance_level, 0.025);
        assert_eq!(test.rejects_zero_effect, Some(true));
        let expected = hypothesis::welch(
            &(100..112).map(f64::from).collect::<Vec<_>>(),
            &(300..312).map(f64::from).collect::<Vec<_>>(),
            config,
        )
        .unwrap();
        assert_eq!(test.test, expected);
    }
    let broad_margin =
        analysis::compare_with_hypothesis(&baseline, Some(&candidate), 250.0, 0.05, config)
            .unwrap();
    assert!(
        broad_margin
            .iter()
            .all(|row| row.decision == analysis::Decision::WithinMargin)
    );
    assert!(
        broad_margin
            .iter()
            .all(|row| row.hypothesis.as_ref().unwrap().rejects_zero_effect == Some(true))
    );
    let exported =
        analysis::compare_with_distribution(&baseline, Some(&candidate), 5.0, 0.05, config)
            .unwrap();
    for (summary, export) in rows.iter().zip(&exported) {
        let original = summary.hypothesis.as_ref().unwrap();
        let retained = export.hypothesis.as_ref().unwrap();
        assert_eq!(original.test, retained.test);
        assert!(original.null_distribution.is_none());
        assert_eq!(
            retained.null_distribution.as_ref().unwrap().len(),
            config.resamples
        );
        let encoded = serde_json::to_string(retained).unwrap();
        let decoded: analysis::HypothesisResult = serde_json::from_str(&encoded).unwrap();
        assert_eq!(retained.null_distribution, decoded.null_distribution);
    }
    let json = serde_json::to_string(&rows).unwrap();
    let decoded: Vec<analysis::Comparison> = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded[0].hypothesis.as_ref().unwrap().test.seed, 19);
    assert!(airbug_bench::report::comparison(&rows).contains("Welch"));
}
#[test]
fn one_process_with_many_batches_has_no_p_value_and_pairs_are_not_reinterpreted() {
    let baseline = run(100, 1, 100);
    let candidate = run(300, 1, 100);
    let rows = analysis::compare(&baseline, Some(&candidate), 5.0, 0.05).unwrap();
    assert!(
        rows.iter()
            .all(|row| row.hypothesis.as_ref().unwrap().test.p_value.is_none())
    );
    let mut paired = run(100, 12, 2);
    for observation in &mut paired.observations {
        observation.variant = "baseline".into();
        observation.pair = Some(observation.process);
        observation.process *= 2;
    }
    let additions: Vec<_> = paired
        .observations
        .iter()
        .map(|o| {
            let mut candidate = o.clone();
            candidate.variant = "candidate".into();
            candidate.process += 1;
            candidate.value =
                Some((candidate.value.as_ref().unwrap().parse::<u128>().unwrap() * 2).to_string());
            candidate
        })
        .collect();
    paired.observations.extend(additions);
    let rows = analysis::compare(&paired, None, 5.0, 0.05).unwrap();
    assert!(rows.iter().all(|row| row.hypothesis.is_none()));
}

#[cfg(feature = "macros")]
#[airbug_bench::group(hypothesis_resamples = 128)]
mod imported_analysis {
    #[bench]
    fn inherited() {
        panic!("analysis configuration must be lazy");
    }
    #[bench(noise_threshold_percent = 0.0, hypothesis_seed = 0)]
    fn explicit_zero() {
        panic!("analysis configuration must be lazy");
    }
}
#[cfg(feature = "macros")]
#[airbug_bench::group(significance_level = 0.01, noise_threshold_percent = 8.0, hypothesis_seed = 19, groups = [crate::imported_analysis])]
mod analysis_defaults {
    #[bench]
    fn inherited() {
        panic!("analysis configuration must be lazy");
    }
    #[group(noise_threshold_percent = 2.0)]
    mod nested {
        #[bench(hypothesis_resamples = 256)]
        fn explicit() {
            panic!("analysis configuration must be lazy");
        }
    }
}
#[cfg(feature = "macros")]
#[test]
fn analysis_attributes_inherit_per_field_across_imported_groups() {
    let mut suite = airbug_bench::Suite::new("scope");
    analysis_defaults::__airbug_register_group(&mut suite);
    let settings = suite
        .comparison_settings(&airbug_bench::Selection::default())
        .unwrap();
    assert_eq!(settings.len(), 4);
    for (id, config) in settings {
        assert_eq!(config.significance_level, 0.01, "{id}");
        if id.ends_with("explicit_zero") {
            assert_eq!(config.noise_threshold_percent, 0.0);
            assert_eq!(config.hypothesis.seed, 0);
            assert_eq!(config.hypothesis.resamples, 128);
        } else if id.ends_with("explicit") {
            assert_eq!(config.noise_threshold_percent, 2.0);
            assert_eq!(config.hypothesis.resamples, 256);
        } else {
            assert_eq!(config.noise_threshold_percent, 8.0);
            assert_eq!(config.hypothesis.seed, 19);
            if id.contains("imported_analysis") {
                assert_eq!(config.hypothesis.resamples, 128);
            }
        }
    }
}

#[test]
fn signed_baselines_preserve_numeric_direction_and_zero_remains_undefined() {
    fn signed(value: f64) -> Run {
        let mut run = run(1, 12, 2);
        for case in &mut run.cases {
            case.metrics[0].id = "balance".into();
            case.metrics[0].unit = "units".into();
            case.contract
                .insert("measurement.domain.balance".into(), "signed".into());
        }
        for o in &mut run.observations {
            o.metric = "balance".into();
            o.value = Some((value * o.operations as f64).to_string());
        }
        run.validate().unwrap();
        run
    }
    use analysis::Decision::{Improvement, Inconclusive, Regression};
    for (before, after, percent, decision) in [
        (-3.0, -6.0, 100.0, Improvement),
        (-3.0, 3.0, -200.0, Regression),
        (3.0, -3.0, -200.0, Improvement),
    ] {
        let baseline = signed(before);
        let candidate = signed(after);
        for paired in [false, true] {
            let rows = if paired {
                let mut joint = baseline.clone();
                for o in &mut joint.observations {
                    o.variant = "baseline".into();
                    o.pair = Some(o.process);
                }
                joint
                    .observations
                    .extend(candidate.observations.iter().cloned().map(|mut o| {
                        o.pair = Some(o.process);
                        o
                    }));
                analysis::compare(&joint, None, 5.0, 0.05).unwrap()
            } else {
                analysis::compare(&baseline, Some(&candidate), 5.0, 0.05).unwrap()
            };
            for row in rows {
                assert_eq!(row.change_percent, Some(percent));
                assert_eq!(row.interval_percent, Some((percent, percent)));
                assert_eq!(row.decision, decision);
            }
        }
    }
    for sign in [1.0, -1.0] {
        let mut baseline = signed(3.0);
        for o in &mut baseline.observations {
            let value = if o.process < 6 { 3.0 } else { 6.0 };
            o.value = Some((sign * value * o.operations as f64).to_string());
        }
        let candidate = signed(-6.0 * sign);
        for row in analysis::compare(&baseline, Some(&candidate), 5.0, 0.05).unwrap() {
            // A denominator spanning 3..6 (or -6..-3) has ratio endpoints
            // -2 and -1, hence ordered changes of -300% and -200%.
            assert_eq!(row.interval_percent, Some((-300.0, -200.0)));
            assert_eq!(
                row.decision,
                if sign > 0.0 { Improvement } else { Regression }
            );
        }
    }
    let mut mixed = signed(3.0);
    for o in &mut mixed.observations {
        o.variant = "baseline".into();
        o.pair = Some(o.process);
        if o.process < 5 {
            o.value = Some((-3.0 * o.operations as f64).to_string());
        }
    }
    mixed
        .observations
        .extend(signed(6.0).observations.into_iter().map(|mut o| {
            o.pair = Some(o.process);
            o
        }));
    for row in analysis::compare(&mixed, None, 5.0, 0.05).unwrap() {
        assert_eq!(row.change_percent, None);
        assert_eq!(row.interval_percent, None);
        assert_eq!(row.decision, Inconclusive);
        assert!(row.note.contains("mixed signs"));
    }
    for row in analysis::compare(&signed(0.0), Some(&signed(-3.0)), 5.0, 0.05).unwrap() {
        assert_eq!(row.change_percent, None);
        assert_eq!(row.interval_percent, None);
        assert_eq!(row.decision, Inconclusive);
        assert!(row.note.contains("zero baseline"));
    }
}
