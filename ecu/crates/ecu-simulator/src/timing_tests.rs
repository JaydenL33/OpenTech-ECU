//! Analytical references, not Speeduino binary parity or hardware timing tests.
use super::*;
use ecu_calibration::wizard::{generate_starter, WizardRequest};
use ecu_scheduler::build_schedule;

fn fixture(rpm: f32, request: ScheduleRequest) -> SimEngine {
    let cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
    let mut engine = SimEngine::new(cal, scenario::by_id("idle_warmup").unwrap()).unwrap();
    // Establish crank-only sync using a separate integer-timestamp 36-1 source.
    // Output timing tests isolate the scheduler from plant and decoder dynamics.
    engine.decoder = MissingToothDecoder::new(MissingToothConfig {
        cam_required: false,
        ..MissingToothConfig::default()
    })
    .unwrap();
    for rev in 0..3 {
        for tooth in 0..35 {
            engine.decoder.on_tooth(1000 + (rev * 36 + tooth) * 1000);
        }
    }
    assert!(engine.decoder.is_sequential());
    engine.schedule = build_schedule(&[1], &request).unwrap();
    engine.true_rpm = rpm;
    engine.fuel_cut = false;
    engine.spark_cut = false;
    engine
}

#[test]
fn wrap_events_have_crossing_timestamps_and_correct_cycles() {
    let mut engine = fixture(
        7500.0,
        ScheduleRequest {
            rpm: 7500.0,
            advance_deg: 0.0,
            injection_end_btdc_deg: 5.0,
            pulse_width_ms: 0.5,
            dwell_ms: 0.5,
            ..ScheduleRequest::default()
        },
    );
    // 45 degrees in one millisecond, from 690 through 0 to 15.
    engine.true_angle_deg = 15.0;
    engine.cycle_count = 8;
    engine.cycle_wrapped = true;
    engine.apply_events(1_000_000, 690.0);
    let outputs = engine.simulated_output_events();
    assert_eq!(outputs.len(), 3);
    assert_eq!(outputs[0].kind, EventKind::InjectionStart);
    assert_eq!(outputs[1].kind, EventKind::DwellStart);
    assert_eq!(outputs[2].kind, EventKind::Spark);
    // Injection: 692.5 deg; dwell: 697.5 deg; spark: 720 deg. Hand vector.
    assert_eq!(
        outputs.iter().map(|e| e.time_us).collect::<Vec<_>>(),
        [1_000_056, 1_000_167, 1_000_667]
    );
    assert_eq!(
        outputs.iter().map(|e| e.cycle).collect::<Vec<_>>(),
        [7, 7, 8]
    );
    assert_eq!(
        outputs.iter().map(|e| e.sequence).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert_eq!(outputs[0].requested_duration_ms, 0.5);
    assert_eq!(outputs[1].requested_duration_ms, 0.5);
}

#[test]
fn constant_rpm_outputs_match_independent_f64_time_reference() {
    for rpm in [600.0_f32, 1000.0, 3000.0, 6000.0, 7500.0, 9000.0] {
        let request = ScheduleRequest {
            rpm,
            pulse_width_ms: 4.0,
            dwell_ms: 3.0,
            advance_deg: 20.0,
            injection_end_btdc_deg: 240.0,
            ..ScheduleRequest::default()
        };
        let mut engine = fixture(rpm, request);
        let degrees_per_us = f64::from(rpm) * 6.0 / 1_000_000.0;
        let mut step = 0_u64;
        // Exercise several wraps with an unchanged request and exact analytical
        // plant phase. Neither expected angles nor expected times use scheduler
        // events or the production crossing-time expression.
        while step as f64 * 1000.0 * degrees_per_us < 2160.0 {
            let start_angle = step as f64 * 1000.0 * degrees_per_us;
            let end_angle = (step + 1) as f64 * 1000.0 * degrees_per_us;
            let prev = (start_angle % 720.0) as f32;
            engine.true_angle_deg = (end_angle % 720.0) as f32;
            engine.cycle_wrapped = end_angle / 720.0 >= (start_angle / 720.0).floor() + 1.0;
            engine.cycle_count = (end_angle / 720.0).floor() as u64;
            engine.apply_events(step * 1000, prev);
            step += 1;
        }
        let outputs = engine.simulated_output_events();
        assert_eq!(outputs.len(), 9, "rpm={rpm}");
        for (index, event) in outputs.iter().enumerate() {
            let cycle = index / 3;
            let expected_us = match index % 3 {
                // Injection ends 240 deg BTDC; spark is 20 deg BTDC.
                0 => (cycle as f64 * 720.0 + 480.0) / degrees_per_us - 4000.0,
                1 => (cycle as f64 * 720.0 + 700.0) / degrees_per_us - 3000.0,
                _ => (cycle as f64 * 720.0 + 700.0) / degrees_per_us,
            };
            assert!(
                (event.time_us as f64 - expected_us).abs() <= 1.0,
                "rpm={rpm} event={event:?} expected_us={expected_us}"
            );
            assert_eq!(event.cycle, cycle as u64);
            assert_eq!(event.sequence, index as u64);
            if index % 3 == 2 {
                assert_eq!(
                    event.time_us - outputs[index - 1].time_us,
                    3000,
                    "rpm={rpm}"
                );
            }
        }
    }
}

#[test]
fn cuts_and_lost_sync_suppress_command_markers() {
    for (fuel_cut, spark_cut, lost_sync, expected) in [
        (
            true,
            false,
            false,
            vec![EventKind::DwellStart, EventKind::Spark],
        ),
        (false, true, false, vec![EventKind::InjectionStart]),
        (true, true, false, vec![]),
        (false, false, true, vec![]),
    ] {
        let mut engine = fixture(3000.0, ScheduleRequest::default());
        engine.fuel_cut = fuel_cut;
        engine.spark_cut = spark_cut;
        if lost_sync {
            engine.decoder.tick(1_000_000);
            assert_eq!(engine.decoder.state(), TriggerSync::Lost);
        }
        // Cross all three commands using test-only phase intervals.
        for angle in 1..=720 {
            engine.true_angle_deg = (angle % 720) as f32;
            engine.cycle_wrapped = angle == 720;
            engine.cycle_count = u64::from(angle == 720);
            engine.apply_events(angle * 1000, (angle - 1) as f32);
        }
        assert_eq!(
            engine
                .simulated_output_events()
                .iter()
                .map(|e| e.kind)
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn seeded_runs_export_repeatable_bounded_traces_with_loss_accounting() {
    let run = |seed| {
        let cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
        let mut scenario = scenario::by_id("mixture_sweep").unwrap();
        scenario.seed = seed;
        let mut engine = SimEngine::new(cal, scenario).unwrap();
        engine.run_to_end();
        (engine.log_file(), engine.simulated_output_trace())
    };
    let (log, trace) = run(42);
    let repeated = run(42);
    assert_eq!(
        serde_json::to_vec(&log).unwrap(),
        serde_json::to_vec(&repeated.0).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&trace).unwrap(),
        serde_json::to_vec(&repeated.1).unwrap()
    );
    let different = run(43);
    assert_ne!(log.frames, different.0.frames);
    assert_ne!(trace.outputs, different.1.outputs);
    assert_eq!(trace.metadata.seed, 42);
    assert!(trace.simulation_only);
    assert!(!trace.ignition_policy.hardware_eligible());
    assert_eq!(trace.outputs.len(), SIMULATED_OUTPUT_EVENT_CAPACITY);
    assert!(trace.events_dropped > 0);
    assert_eq!(
        trace.events_emitted,
        trace.events_dropped + trace.outputs.len() as u64
    );
    assert_eq!(trace.outputs[0].sequence, trace.events_dropped);
    assert_eq!(
        trace.outputs.last().unwrap().sequence + 1,
        trace.events_emitted
    );
    assert!(trace.outputs.iter().any(|e| e.time_us % 1000 != 0));
    for pair in trace.outputs.windows(2) {
        assert!(pair[0].time_us <= pair[1].time_us);
        assert!(pair[0].cycle <= pair[1].cycle);
        assert_eq!(pair[1].sequence, pair[0].sequence + 1);
    }
    assert!(trace.outputs.last().unwrap().time_us <= trace.captured_until_us);
    let json = serde_json::to_vec(&trace).unwrap();
    assert_eq!(
        serde_json::from_slice::<SimulatedOutputTrace>(&json).unwrap(),
        trace
    );
}
