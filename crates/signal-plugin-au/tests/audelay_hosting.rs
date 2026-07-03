//! Hosting + discovery round-trip tests against the stock Apple AUDelay
//! (`aufx:dely:appl`) — always present on macOS, including macos-latest CI,
//! so no compiled fixture is needed (the AudioComponent registrar caches
//! directories, which makes temp-bundle fixtures invisible cross-process).
//! macOS only by nature; the whole file compiles away elsewhere.

#![cfg(target_os = "macos")]

use std::path::Path;

use signal_plugin_au::{AuHostAdapter, AuHostedInstance, AU_REGISTRY_COMPONENT_PATH};

const AUDELAY_LOAD_KEY: &str = "aufx:dely:appl";
const AUDELAY_PLUGIN_TYPE_ID: &str = "plugin:au:aufx:dely:appl";

/// AUDelay's stable parameter ids (AudioToolbox `kDelayParam_*`).
const PARAM_WET_DRY_MIX: u32 = 0;
const PARAM_DELAY_TIME: u32 = 1;
const PARAM_FEEDBACK: u32 = 2;
const PARAM_LOPASS_CUTOFF: u32 = 3;

const SAMPLE_RATE_HZ: f64 = 48_000.0;
const MAX_FRAMES: u32 = 256;

fn load_audelay() -> AuHostedInstance {
    AuHostedInstance::load(Path::new(AU_REGISTRY_COMPONENT_PATH), AUDELAY_LOAD_KEY)
        .expect("stock AUDelay should resolve through the system registry")
}

#[test]
fn registry_discovery_finds_stock_audelay() {
    let discovered = AuHostAdapter::default().discover_plugins_from_registry();
    assert!(
        !discovered.is_empty(),
        "a stock macOS registry lists Apple units",
    );
    let delay = discovered
        .iter()
        .find(|plugin| plugin.plugin_type_id.0 == AUDELAY_PLUGIN_TYPE_ID)
        .expect("stock AUDelay in the registry discovery");
    assert_eq!(delay.component_type, "aufx");
    assert_eq!(delay.component_subtype, "dely");
    assert_eq!(delay.manufacturer_code, "appl");
    assert_eq!(delay.load_key(), AUDELAY_LOAD_KEY);
    assert_eq!(delay.descriptor.vendor, "Apple");
    assert_eq!(delay.bundle_root, AU_REGISTRY_COMPONENT_PATH);
    assert!(
        delay.descriptor.parameters.is_empty(),
        "scan-time AU inventory must be empty (real inventory at load)",
    );
    // Registry entries are unique per fourcc triple.
    let mut ids: Vec<&str> = discovered
        .iter()
        .map(|plugin| plugin.plugin_type_id.0.as_str())
        .collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "duplicate registry plugin_type_ids");
}

#[test]
fn load_enumerates_the_known_audelay_parameter_inventory() {
    let instance = load_audelay();
    let parameters = instance.parameters();
    assert!(!parameters.is_empty(), "AUDelay exposes parameters");
    for id in [
        PARAM_WET_DRY_MIX,
        PARAM_DELAY_TIME,
        PARAM_FEEDBACK,
        PARAM_LOPASS_CUTOFF,
    ] {
        assert!(
            parameters.iter().any(|p| p.parameter_id == id),
            "AUDelay parameter id {id} missing from the inventory",
        );
    }
    let wet_dry = parameters
        .iter()
        .find(|p| p.parameter_id == PARAM_WET_DRY_MIX)
        .expect("wet/dry mix descriptor");
    assert!((wet_dry.min_plain - 0.0).abs() < 1e-6);
    assert!((wet_dry.max_plain - 100.0).abs() < 1e-6);
    assert!(wet_dry.flags.automatable);

    let layout = instance.port_layout();
    assert_eq!(layout.main_input_channels, 2);
    assert_eq!(layout.main_output_channels, 2);
    assert!(layout.is_stereo_effect());
}

#[test]
fn fully_dry_audelay_is_identity_within_epsilon() {
    let mut instance = load_audelay();
    assert!(instance.process_session().is_err(), "inactive: no session");
    instance
        .activate(SAMPLE_RATE_HZ, 1, MAX_FRAMES)
        .expect("stereo AUDelay should activate");
    assert!(
        instance.activate(SAMPLE_RATE_HZ, 1, MAX_FRAMES).is_err(),
        "no re-entry",
    );
    // 100% dry: the delay line contributes nothing — output ≈ input.
    instance
        .set_parameter(PARAM_WET_DRY_MIX, 0.0)
        .expect("wet/dry mix set");

    let mut session = instance
        .process_session()
        .expect("active instance builds a session");
    session.start().expect("session start");
    assert!(session.is_processing());

    let frames = 128usize;
    for block in 0..8u32 {
        let input: Vec<f32> = (0..frames * 2)
            .map(|index| ((index as f32) + (block as f32) * 7.0) / 512.0 - 0.4)
            .collect();
        let mut output = vec![0.0f32; frames * 2];
        assert!(
            session.process_interleaved_stereo(&input, &mut output, frames),
            "block {block} should render",
        );
        for (index, (wet, dry)) in output.iter().zip(input.iter()).enumerate() {
            assert!(
                (wet - dry).abs() <= 1e-6,
                "block {block} sample {index}: {wet} vs {dry} (identity, epsilon 1e-6)",
            );
        }
    }

    // In-place path used by the in-process tier.
    let input: Vec<f32> = (0..frames * 2).map(|index| index as f32 / 400.0).collect();
    let mut scratch = input.clone();
    assert!(session.process_in_place(&mut scratch, frames));
    for (index, (wet, dry)) in scratch.iter().zip(input.iter()).enumerate() {
        assert!(
            (wet - dry).abs() <= 1e-6,
            "in-place sample {index}: {wet} vs {dry}",
        );
    }

    session.stop();
    assert!(!session.is_processing());
    drop(session);
    instance.deactivate().expect("deactivate");
    assert!(instance.deactivate().is_err(), "double deactivate rejected");
}

#[test]
fn fully_wet_impulse_lands_at_the_configured_delay() {
    let mut instance = load_audelay();
    instance
        .activate(SAMPLE_RATE_HZ, 1, MAX_FRAMES)
        .expect("stereo AUDelay should activate");
    // 100% wet, no feedback, 50 ms delay → the impulse must reappear at
    // round(0.05 × 48000) = 2400 samples plus AUDelay's one-sample
    // interpolator latency (measured: the allpass fractional-delay stage
    // shifts the peak by exactly +1 at any delay time, with a short
    // exponential smear behind it).
    const AUDELAY_INTERPOLATOR_LATENCY_FRAMES: usize = 1;
    let delay_seconds = 0.05f32;
    let expected_delay_frames = (delay_seconds as f64 * SAMPLE_RATE_HZ).round() as usize
        + AUDELAY_INTERPOLATOR_LATENCY_FRAMES;
    instance
        .set_parameter(PARAM_WET_DRY_MIX, 100.0)
        .expect("wet/dry mix set");
    instance
        .set_parameter(PARAM_FEEDBACK, 0.0)
        .expect("feedback set");
    instance
        .set_parameter(PARAM_DELAY_TIME, delay_seconds)
        .expect("delay time set");

    let mut session = instance.process_session().expect("session");
    session.start().expect("session start");

    let frames = MAX_FRAMES as usize;
    let total_blocks = expected_delay_frames / frames + 2;
    let mut rendered_left: Vec<f32> = Vec::with_capacity(total_blocks * frames);
    for block in 0..total_blocks {
        let mut input = vec![0.0f32; frames * 2];
        if block == 0 {
            input[0] = 1.0; // left-channel impulse at frame 0
        }
        let mut output = vec![0.0f32; frames * 2];
        assert!(session.process_interleaved_stereo(&input, &mut output, frames));
        rendered_left.extend((0..frames).map(|frame| output[frame * 2]));
    }

    let (peak_index, peak_value) = rendered_left
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.abs().total_cmp(&right.1.abs()))
        .expect("rendered output non-empty");
    assert!(
        peak_value.abs() > 0.25,
        "delayed impulse should be clearly audible, peak {peak_value}",
    );
    assert_eq!(
        peak_index, expected_delay_frames,
        "impulse must land at round(delay × rate) + interpolator latency",
    );
    // Before the delay tap the (100% wet) line is silent.
    for (index, sample) in rendered_left[..expected_delay_frames.saturating_sub(4)]
        .iter()
        .enumerate()
    {
        assert!(
            sample.abs() < 1e-3,
            "pre-tap sample {index} should be silent, saw {sample}",
        );
    }

    session.stop();
    drop(session);
    instance.deactivate().expect("deactivate");
}
