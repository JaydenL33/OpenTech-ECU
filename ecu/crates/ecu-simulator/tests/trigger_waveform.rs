//! Independent analytical references; no Speeduino binary or hardware parity.
use ecu_calibration::{generate_starter, WizardRequest};
use ecu_simulator::trigger_waveform::{TriggerInput, TriggerWaveform, WaveformError};
use ecu_simulator::{
    scenario::by_id, SimEngine, SimulatedTriggerTrace, SIMULATED_TRIGGER_EVENT_CAPACITY,
};
use ecu_trigger::{MissingToothConfig, MissingToothDecoder, SyncState};
use ecu_types::TriggerPattern;

fn config(teeth: u16, missing: u16, offset: f32) -> MissingToothConfig {
    MissingToothConfig {
        teeth,
        missing,
        sync_offset_deg: offset,
        ..Default::default()
    }
}

#[test]
fn full_cycles_match_independent_f64_wheel_vectors() {
    let mut compared = 0;
    for (teeth, missing) in [(36, 1), (36, 2), (60, 2), (7, 2)] {
        for offset in [0.0, 25.0, 95.0, 355.0] {
            let cfg = config(teeth, missing, offset);
            // Deliberately between crank teeth, including the 60-2 gap.
            let waveform = TriggerWaveform::new(cfg, 707.5).unwrap();
            for rpm in [600_u64, 1000, 3000, 6000, 7500, 9000] {
                let period_us = 120_000_000 / rpm;
                let edges = waveform.edges(0.0, 720.0, 1000, period_us).unwrap();
                let mut reference = Vec::new();
                // Independent nominal slot enumeration with exclusion predicate.
                for slot in 0..2 * teeth {
                    if slot % teeth >= teeth - missing {
                        continue;
                    }
                    let phase =
                        (f64::from(slot) * 360.0 / f64::from(teeth) + f64::from(offset)) % 720.0;
                    reference.push((
                        if phase == 0.0 { 720.0 } else { phase },
                        TriggerInput::Crank,
                    ));
                }
                let phase = (707.5 + f64::from(offset)) % 720.0;
                reference.push((phase, TriggerInput::Cam));
                reference.sort_by(|a, b| a.0.total_cmp(&b.0));
                assert_eq!(edges.len(), 2 * usize::from(teeth - missing) + 1);
                for (edge, (angle, input)) in edges.iter().zip(reference) {
                    assert_eq!(edge.input, input);
                    let expected_us = 1000.0 + angle / 720.0 * period_us as f64;
                    assert!(
                        (edge.time_us as f64 - expected_us).abs() <= 1.01,
                        "{cfg:?} rpm={rpm} edge={edge:?} expected={expected_us}"
                    );
                    assert!((f64::from(edge.angle_deg) - angle % 720.0).abs() < 0.0002);
                    compared += 1;
                }
            }
        }
    }
    println!("independent full-cycle edge comparisons: {compared}; tolerance <= 1.01 us");
}

#[test]
fn independent_cam_and_split_intervals_preserve_order_and_endpoints() {
    let waveform = TriggerWaveform::new(config(60, 2, 0.0), 707.5).unwrap();
    // Last present crank at 702°, missing positions 708°/714°, then tooth 0.
    let edges = waveform.edges(700.0, 25.0, 1000, 2500).unwrap();
    assert_eq!(
        edges
            .iter()
            .map(|e| (e.input, e.angle_deg, e.time_us))
            .collect::<Vec<_>>(),
        [
            (TriggerInput::Crank, 702.0, 1200),
            (TriggerInput::Cam, 707.5, 1750),
            (TriggerInput::Crank, 0.0, 3000)
        ]
    );
    let mut split = waveform.edges(700.0, 7.5, 1000, 750).unwrap();
    split.extend(waveform.edges(707.5, 17.5, 1750, 1750).unwrap());
    assert_eq!(edges, split);
    assert!(waveform.edges(707.5, 0.0, 0, 1000).unwrap().is_empty());

    let coincident = TriggerWaveform::new(config(36, 1, 0.0), 690.0).unwrap();
    let pair = coincident.edges(680.0, 10.0, 0, 1000).unwrap();
    assert_eq!(pair.len(), 2);
    assert_eq!(pair[0].input, TriggerInput::Crank);
    assert_eq!(pair[1].input, TriggerInput::Cam);
    assert_eq!(pair[0].time_us, pair[1].time_us);
}

#[test]
fn cam_in_missing_region_decodes_without_a_synthetic_crank_tooth() {
    let cfg = config(60, 2, 0.0);
    let waveform = TriggerWaveform::new(cfg, 707.5).unwrap();
    let mut decoder = MissingToothDecoder::new(cfg).unwrap();
    decoder.set_cam_window(Some((340.0, 355.0))).unwrap();
    for cycle in 0..4 {
        let edges = waveform
            .edges(0.0, 720.0, 1000 + cycle * 120_000, 120_000)
            .unwrap();
        assert_eq!(
            edges
                .iter()
                .filter(|e| e.input == TriggerInput::Crank)
                .count(),
            116
        );
        for edge in edges {
            match edge.input {
                TriggerInput::Crank => {
                    decoder.on_tooth(edge.time_us);
                }
                TriggerInput::Cam => {
                    decoder.on_cam(edge.time_us);
                    assert_eq!(decoder.state(), SyncState::Sequential);
                    assert!((decoder.cycle_angle_at(edge.time_us).unwrap() - 707.5).abs() < 0.01);
                }
            }
        }
    }
    assert_eq!(decoder.status().error_count, 0);
}

#[test]
fn invalid_waveforms_and_unbounded_steps_are_rejected() {
    for cam in [f32::NAN, f32::INFINITY, -1.0, 720.0] {
        assert!(matches!(
            TriggerWaveform::new(config(36, 1, 0.0), cam),
            Err(WaveformError::InvalidCamPhase)
        ));
    }
    assert!(matches!(
        TriggerWaveform::new(config(0, 1, 0.0), 690.0),
        Err(WaveformError::Configuration(_))
    ));
    let waveform = TriggerWaveform::new(config(36, 1, 0.0), 690.0).unwrap();
    for (start, advance, time, duration) in [
        (f32::NAN, 1.0, 0, 1000),
        (720.0, 1.0, 0, 1000),
        (0.0, f32::INFINITY, 0, 1000),
        (0.0, -1.0, 0, 1000),
        (0.0, 721.0, 0, 1000),
        (0.0, 1.0, 0, 0),
        (0.0, 1.0, u64::MAX, 1000),
    ] {
        assert_eq!(
            waveform.edges(start, advance, time, duration),
            Err(WaveformError::InvalidStep)
        );
    }
}

#[test]
fn decoder_matches_independent_offset_and_phase_vectors() {
    for (teeth, missing) in [(36, 1), (36, 2), (60, 2)] {
        for offset in [0_u64, 25, 95, 355] {
            let cfg = config(teeth, missing, offset as f32);
            let mut decoder = MissingToothDecoder::new(cfg).unwrap();
            decoder.set_cam_window(Some((320.0, 340.0))).unwrap();
            // Independent integer-degree / timestamp source, not TriggerWaveform.
            // At 1000 rpm exactly 500/3 us per degree. Cam at relative 690°.
            let mut reference = Vec::new();
            for rev in 0..8_u64 {
                for tooth in 0..u64::from(teeth - missing) {
                    let angle = offset + rev * 360 + tooth * 360 / u64::from(teeth);
                    reference.push((angle, TriggerInput::Crank));
                }
                if rev % 2 == 1 {
                    reference.push((offset + rev * 360 + 330, TriggerInput::Cam));
                }
            }
            reference.sort_by_key(|e| e.0);
            for (angle, input) in reference {
                let time = 1000 + angle * 500 / 3;
                match input {
                    TriggerInput::Crank => {
                        decoder.on_tooth(time);
                    }
                    TriggerInput::Cam => {
                        decoder.on_cam(time);
                    }
                }
                let status = decoder.status();
                if angle >= offset + 720 {
                    assert_eq!(
                        status.state,
                        SyncState::Sequential,
                        "{cfg:?} angle={angle} {status:?}"
                    );
                    let expected = (angle % 720) as f32;
                    let actual = status.cycle_angle_deg.unwrap();
                    assert!(
                        (actual - expected).abs() < 0.001,
                        "{cfg:?} angle={angle} actual={actual}"
                    );
                    // Extrapolate through crank 360° and cycle 720°, including
                    // the missing gap, before the next capture is delivered.
                    let extrapolated = decoder.cycle_angle_at(time + 1000).unwrap();
                    let expected = (expected + 6.0) % 720.0;
                    let error = (extrapolated - expected + 360.0).rem_euclid(720.0) - 360.0;
                    assert!(error.abs() < 0.01, "{cfg:?} angle={angle} error={error}");
                }
            }
            assert_eq!(decoder.status().error_count, 0);
            assert!((decoder.rpm() - 1000.0).abs() < 0.01);
        }
    }
}

fn run_sim(teeth: u16, missing: u16, offset: f32, seed: u64, duration_ms: u64) -> SimEngine {
    let mut cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
    cal.engine.trigger.pattern = TriggerPattern::MissingTooth { teeth, missing };
    cal.engine.trigger.sync_offset_deg = offset;
    cal.seal();
    let mut scenario = by_id("idle_warmup").unwrap();
    scenario.seed = seed;
    scenario.duration_ms = duration_ms;
    let mut engine = SimEngine::new(cal, scenario).unwrap();
    engine.run_to_end();
    engine
}

#[test]
fn general_wheels_run_through_real_simulator_and_export_bounded_seeded_inputs() {
    for (teeth, missing) in [(36, 2), (60, 2)] {
        for offset in [0.0, 95.0, 355.0] {
            let engine = run_sim(teeth, missing, offset, 42, 5000);
            let trace = engine.simulated_trigger_trace();
            assert!(trace.simulation_only);
            assert!(!engine
                .simulated_output_trace()
                .ignition_policy
                .hardware_eligible());
            assert!(!engine.simulated_output_events().is_empty());
            assert_eq!(
                trace.inputs.last().unwrap().decoder.state,
                SyncState::Sequential
            );
            assert_eq!(trace.inputs.last().unwrap().decoder.error_count, 0);
            for event in &trace.inputs {
                if event.edge.input == TriggerInput::Crank {
                    let relative = (event.edge.angle_deg - offset).rem_euclid(360.0);
                    assert!(relative < f32::from(teeth - missing) * 360.0 / f32::from(teeth));
                }
                if let Some(angle) = event.decoder.cycle_angle_deg {
                    let error = (angle - event.edge.angle_deg + 360.0).rem_euclid(720.0) - 360.0;
                    // Cam may be between teeth; status is the last capture angle.
                    if event.edge.input == TriggerInput::Crank {
                        assert!(error.abs() < 0.001);
                    }
                }
            }
        }
    }
    let trace = run_sim(60, 2, 95.0, 42, 15000).simulated_trigger_trace();
    let repeated = run_sim(60, 2, 95.0, 42, 15000).simulated_trigger_trace();
    assert_eq!(
        serde_json::to_vec(&trace).unwrap(),
        serde_json::to_vec(&repeated).unwrap()
    );
    assert_ne!(
        trace.inputs,
        run_sim(60, 2, 95.0, 43, 15000)
            .simulated_trigger_trace()
            .inputs
    );
    assert_eq!(trace.metadata.seed, 42);
    assert_eq!(trace.inputs.len(), SIMULATED_TRIGGER_EVENT_CAPACITY);
    assert!(trace.events_dropped > 0);
    assert_eq!(
        trace.events_emitted,
        trace.events_dropped + trace.inputs.len() as u64
    );
    assert_eq!(trace.inputs[0].sequence, trace.events_dropped);
    assert_eq!(
        trace.inputs.last().unwrap().sequence + 1,
        trace.events_emitted
    );
    for pair in trace.inputs.windows(2) {
        assert!(pair[0].edge.time_us <= pair[1].edge.time_us);
        assert_eq!(pair[1].sequence, pair[0].sequence + 1);
    }
    assert!(trace.inputs.last().unwrap().edge.time_us <= trace.captured_until_us);
    assert_eq!(
        serde_json::from_slice::<SimulatedTriggerTrace>(&serde_json::to_vec(&trace).unwrap())
            .unwrap(),
        trace
    );
}
