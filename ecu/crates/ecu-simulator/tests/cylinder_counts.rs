use ecu_calibration::{generate_starter, hex_sha256, Calibration, WizardRequest};
use ecu_scheduler::{build_schedule, EventKind, ScheduleRequest};
use ecu_simulator::{scenario::by_id, SimEngine};
use ecu_trigger::MissingToothConfig;
use ecu_types::{EngineConfig, EngineState, SyncState};

const BASELINE_CALIBRATION_SHA256: &str =
    "d9ddcdf2ef600a610ae319698b97eedacbad94291e67c8bf7099edb9ae5e5a89";
const BASELINE_IDLE_WARMUP_LOG_SHA256: &str =
    "9bc9b970cb5558432f692cd3615920bd11050260072b389b937453661a09f418";

fn calibration_for(n: u8) -> Calibration {
    generate_starter(
        &WizardRequest {
            engine: EngineConfig::virtual_sequential(n).unwrap(),
            ..WizardRequest::default_inline4()
        },
        1,
    )
    .unwrap()
}

#[test]
fn every_cylinder_count_runs_sequential_with_even_intervals() {
    for n in 1..=8 {
        let cal = calibration_for(n);
        let order = cal.engine.firing_order.clone();
        let mut engine = SimEngine::new(cal, by_id("idle_warmup").unwrap()).unwrap();
        engine.run_to_end();
        assert!(
            engine
                .frames()
                .iter()
                .any(|f| f.engine_state == EngineState::Running),
            "n={n} never reached Running"
        );
        assert_eq!(
            engine.last_frame().unwrap().sync,
            SyncState::Sequential,
            "n={n}"
        );
        let events = engine.simulated_output_events();
        for cylinder in 1..=n {
            for kind in [EventKind::InjectionStart, EventKind::Spark] {
                assert!(
                    events
                        .iter()
                        .any(|e| e.cylinder == cylinder && e.kind == kind),
                    "n={n} cylinder={cylinder} kind={kind:?}"
                );
            }
        }
        assert!(events.iter().all(|e| e.channel == e.cylinder), "n={n}");
        let last = events.last().unwrap().cycle;
        assert!(last >= 5, "n={n} last={last}");
        for cycle in last - 4..last {
            for kind in [EventKind::InjectionStart, EventKind::Spark] {
                let cycle_events: Vec<_> = events
                    .iter()
                    .filter(|e| e.cycle == cycle && e.kind == kind)
                    .collect();
                assert_eq!(
                    cycle_events.len(),
                    n as usize,
                    "n={n} cycle={cycle} kind={kind:?}"
                );
                for pair in cycle_events.windows(2) {
                    let (a, b) = (pair[0], pair[1]);
                    let interval = (b.angle_deg - a.angle_deg).rem_euclid(720.0);
                    assert!(
                        (interval - 720.0 / n as f32).abs() < 1e-3,
                        "n={n} cycle={cycle} kind={kind:?} interval={interval}"
                    );
                    let position = order.iter().position(|&c| c == a.cylinder).unwrap();
                    assert_eq!(
                        b.cylinder,
                        order[(position + 1) % order.len()],
                        "n={n} cycle={cycle} kind={kind:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn cylinder_to_tooth_mapping_matches_decision_table() {
    let references: [&[(u8, u16, f32)]; 8] = [
        &[(1, 0, 0.0)],
        &[(1, 0, 0.0), (2, 36, 0.0)],
        &[(1, 0, 0.0), (2, 24, 0.0), (3, 48, 0.0)],
        &[(1, 0, 0.0), (3, 18, 0.0), (4, 36, 0.0), (2, 54, 0.0)],
        &[
            (1, 0, 0.0),
            (2, 14, 4.0),
            (4, 28, 8.0),
            (5, 43, 2.0),
            (3, 57, 6.0),
        ],
        &[
            (1, 0, 0.0),
            (5, 12, 0.0),
            (3, 24, 0.0),
            (6, 36, 0.0),
            (2, 48, 0.0),
            (4, 60, 0.0),
        ],
        &[
            (1, 0, 0.0),
            (3, 10, 2.8571),
            (5, 20, 5.7143),
            (7, 30, 8.5714),
            (2, 41, 1.4286),
            (4, 51, 4.2857),
            (6, 61, 7.1429),
        ],
        &[
            (1, 0, 0.0),
            (8, 9, 0.0),
            (4, 18, 0.0),
            (3, 27, 0.0),
            (6, 36, 0.0),
            (5, 45, 0.0),
            (7, 54, 0.0),
            (2, 63, 0.0),
        ],
    ];
    for n in 1..=8 {
        let layout = EngineConfig::virtual_sequential(n)
            .unwrap()
            .cylinder_layout()
            .unwrap();
        let expected = references[n as usize - 1];
        assert_eq!(layout.len(), expected.len());
        for (slot, &(cylinder, tooth, offset)) in layout.iter().zip(expected) {
            let reference = MissingToothConfig::default()
                .tooth_reference(slot.tdc_deg)
                .unwrap();
            assert_eq!(slot.cylinder, cylinder, "n={n}");
            assert_eq!(reference.tooth, tooth, "n={n} cylinder={cylinder}");
            assert!(
                (reference.offset_deg - offset).abs() < 1e-3,
                "n={n} cylinder={cylinder}: {reference:?}"
            );
            assert_ne!(reference.tooth % 36, 35);
        }
    }
}

#[test]
fn layout_tdc_matches_scheduler_spark_angles() {
    for n in 1..=8 {
        let engine = EngineConfig::virtual_sequential(n).unwrap();
        let schedule = build_schedule(
            &engine.firing_order,
            &ScheduleRequest {
                advance_deg: 0.0,
                ..ScheduleRequest::default()
            },
        )
        .unwrap();
        for slot in engine.cylinder_layout().unwrap() {
            let spark = schedule
                .events()
                .iter()
                .find(|e| e.cylinder == slot.cylinder && e.kind == EventKind::Spark)
                .unwrap();
            assert_eq!(
                spark.angle_deg, slot.tdc_deg,
                "n={n} cylinder={}",
                slot.cylinder
            );
            assert_eq!(spark.output_channel, slot.coil_channel);
            let injection = schedule
                .events()
                .iter()
                .find(|e| e.cylinder == slot.cylinder && e.kind == EventKind::InjectionStart)
                .unwrap();
            assert_eq!(injection.output_channel, slot.injector_channel);
        }
    }
}

#[test]
fn inline4_matches_post_rust_trg_002_baseline_with_stable_sync() {
    let cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
    assert_eq!(
        hex_sha256(&serde_json::to_vec_pretty(&cal).unwrap()),
        BASELINE_CALIBRATION_SHA256
    );
    let mut engine = SimEngine::new(cal, by_id("idle_warmup").unwrap()).unwrap();
    engine.run_to_end();
    assert!(!engine.frames().is_empty());
    let first = engine
        .frames()
        .iter()
        .position(|f| f.sync == SyncState::Sequential)
        .expect("idle_warmup must acquire sequential sync");
    let first_ms = engine.frames()[first].scenario_time_ms;
    assert!(first_ms <= 600, "first sequential sync at {first_ms} ms");
    let dropouts = engine.frames()[first..]
        .iter()
        .filter(|f| f.sync != SyncState::Sequential)
        .count();
    assert_eq!(dropouts, 0);
    let mut repeat = SimEngine::new(
        generate_starter(&WizardRequest::default_inline4(), 1).unwrap(),
        by_id("idle_warmup").unwrap(),
    )
    .unwrap();
    repeat.run_to_end();
    assert_eq!(
        hex_sha256(&serde_json::to_vec_pretty(&engine.log_file()).unwrap()),
        hex_sha256(&serde_json::to_vec_pretty(&repeat.log_file()).unwrap())
    );
    println!("idle_warmup: first sequential frame {first}, {first_ms} ms; post-sync dropouts {dropouts}; repeat hash identical");
    // Post-RUST-TRG-002 baseline: one extra acquisition revolution changes
    // the closed-loop plant trajectory. Prior hash:
    // ff6d5a675d321f073cea847aa9c16683e7a48364c65198c4db6c9851d98a2c9a.
    // `accel_enrich_pct` was added to the frame after this baseline was taken.
    // The simulator never models enrichment, so it must be exactly 0.0 on every
    // frame; the line is then removed (with the comma it leaves behind on the
    // preceding field) so the post-RUST-TRG-002 normalized transcript hash must match.
    let pretty = String::from_utf8(serde_json::to_vec_pretty(&engine.log_file()).unwrap()).unwrap();
    let mut kept: Vec<&str> = Vec::new();
    let mut removed = 0usize;
    for line in pretty.lines() {
        if line.trim_start().starts_with("\"accel_enrich_pct\"") {
            assert_eq!(line.trim(), "\"accel_enrich_pct\": 0.0");
            let prev = kept.pop().expect("field is never first");
            kept.push(prev.strip_suffix(',').expect("field follows another field"));
            removed += 1;
        } else {
            kept.push(line);
        }
    }
    assert!(
        removed > 100,
        "expected one field per frame, removed {removed}"
    );
    assert_eq!(
        hex_sha256(kept.join("\n").as_bytes()),
        BASELINE_IDLE_WARMUP_LOG_SHA256
    );
}
