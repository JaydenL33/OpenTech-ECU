use ecu_calibration::{generate_starter, hex_sha256, Calibration, WizardRequest};
use ecu_scheduler::{build_schedule, EventKind, ScheduleRequest};
use ecu_simulator::{scenario::by_id, SimEngine};
use ecu_trigger::MissingToothConfig;
use ecu_types::{EngineConfig, EngineState, SyncState};

const BASELINE_CALIBRATION_SHA256: &str =
    "d9ddcdf2ef600a610ae319698b97eedacbad94291e67c8bf7099edb9ae5e5a89";
const BASELINE_IDLE_WARMUP_LOG_SHA256: &str =
    "ff6d5a675d321f073cea847aa9c16683e7a48364c65198c4db6c9851d98a2c9a";

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
fn inline4_outputs_match_pre_change_baseline() {
    let cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
    assert_eq!(
        hex_sha256(&serde_json::to_vec_pretty(&cal).unwrap()),
        BASELINE_CALIBRATION_SHA256
    );
    let mut engine = SimEngine::new(cal, by_id("idle_warmup").unwrap()).unwrap();
    engine.run_to_end();
    assert_eq!(
        hex_sha256(&serde_json::to_vec_pretty(&engine.log_file()).unwrap()),
        BASELINE_IDLE_WARMUP_LOG_SHA256
    );
}
