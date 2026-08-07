use std::sync::Arc;

use signal_primitives::SampleRate;

use crate::{
    PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventKind, RenderVoiceParam,
};

use super::BinauralVoiceBank;

fn event(kind: RenderPluginEventKind) -> RenderBlockPluginEvent {
    RenderBlockPluginEvent {
        offset_frames: 0,
        channel: 0,
        kind,
    }
}

fn identity_bank(max_voices: usize) -> BinauralVoiceBank {
    BinauralVoiceBank::new(
        vec![Arc::new(vec![1.0, 1.0, 1.0, 1.0])],
        vec![(vec![1.0], vec![1.0]), (vec![0.5], vec![0.25])],
        max_voices,
        4,
        SampleRate(48_000),
    )
}

#[test]
fn voice_start_renders_into_stereo_scratch_additively() {
    let bank = identity_bank(2);
    let mut scratch = vec![0.1f32; 8 * 2]; // pre-existing stage content
    let events = [
        event(RenderPluginEventKind::VoiceStart {
            voice: 0,
            sound: 0,
            gain: 0.5,
        }),
        event(RenderPluginEventKind::VoiceParam {
            voice: 0,
            param: RenderVoiceParam::HrirIndex,
            value: 0.0,
        }),
    ];
    assert!(bank.process_with_events(&mut scratch, 8, 2, &events));
    // First four frames carry the sound (0.5 gain) plus the 0.1 base.
    for frame in 0..4 {
        assert!((scratch[frame * 2] - 0.6).abs() < 1e-6, "frame {frame}");
        assert!((scratch[frame * 2 + 1] - 0.6).abs() < 1e-6);
    }
    // Sound ended; remaining frames are the untouched base.
    for frame in 4..8 {
        assert!((scratch[frame * 2] - 0.1).abs() < 1e-6, "frame {frame}");
    }
    // Fire-and-forget: the voice freed itself at end of sound.
    assert_eq!(bank.active_voices(), 0);
}

#[test]
fn per_ear_responses_apply() {
    let bank = identity_bank(1);
    let mut scratch = vec![0.0f32; 4 * 2];
    let events = [
        event(RenderPluginEventKind::VoiceStart {
            voice: 0,
            sound: 0,
            gain: 1.0,
        }),
        event(RenderPluginEventKind::VoiceParam {
            voice: 0,
            param: RenderVoiceParam::HrirIndex,
            value: 1.0, // (0.5 left, 0.25 right)
        }),
    ];
    assert!(bank.process_with_events(&mut scratch, 4, 2, &events));
    assert!((scratch[0] - 0.5).abs() < 1e-6);
    assert!((scratch[1] - 0.25).abs() < 1e-6);
}

#[test]
fn voice_stop_silences_mid_sound() {
    let bank = identity_bank(1);
    let mut scratch = vec![0.0f32; 2 * 2];
    let start = [
        event(RenderPluginEventKind::VoiceStart {
            voice: 0,
            sound: 0,
            gain: 1.0,
        }),
        event(RenderPluginEventKind::VoiceParam {
            voice: 0,
            param: RenderVoiceParam::HrirIndex,
            value: 0.0,
        }),
    ];
    assert!(bank.process_with_events(&mut scratch, 2, 2, &start));
    assert_eq!(bank.active_voices(), 1);

    let stop = [event(RenderPluginEventKind::VoiceStop { voice: 0 })];
    let mut scratch2 = vec![0.0f32; 2 * 2];
    assert!(bank.process_with_events(&mut scratch2, 2, 2, &stop));
    assert!(scratch2.iter().all(|&s| s.abs() < 1e-6));
    assert_eq!(bank.active_voices(), 0);
}

#[test]
fn hrir_reselect_same_index_is_noop_and_change_crossfades() {
    let bank = identity_bank(1);
    let mut scratch = vec![0.0f32; 2 * 2];
    let events = [
        event(RenderPluginEventKind::VoiceStart {
            voice: 0,
            sound: 0,
            gain: 1.0,
        }),
        event(RenderPluginEventKind::VoiceParam {
            voice: 0,
            param: RenderVoiceParam::HrirIndex,
            value: 0.0,
        }),
        // Same index again: dedup, still snapped (no fade).
        event(RenderPluginEventKind::VoiceParam {
            voice: 0,
            param: RenderVoiceParam::HrirIndex,
            value: 0.0,
        }),
    ];
    assert!(bank.process_with_events(&mut scratch, 2, 2, &events));
    assert!((scratch[0] - 1.0).abs() < 1e-6, "first direction snaps");

    // A real change crossfades: output moves smoothly toward the new
    // response rather than jumping.
    let change = [event(RenderPluginEventKind::VoiceParam {
        voice: 0,
        param: RenderVoiceParam::HrirIndex,
        value: 1.0,
    })];
    let mut scratch2 = vec![0.0f32; 2 * 2];
    assert!(bank.process_with_events(&mut scratch2, 2, 2, &change));
    let first = scratch2[0];
    assert!(first < 1.0 && first > 0.5, "mid-fade sample, got {first}");
}

#[test]
fn wrong_channel_count_bypasses() {
    let bank = identity_bank(1);
    let mut scratch = vec![0.25f32; 4];
    assert!(!bank.process(&mut scratch, 4, 1));
    assert!(scratch.iter().all(|&s| (s - 0.25).abs() < 1e-6));
}

#[test]
fn out_of_range_addressing_counts_unsupported() {
    let bank = identity_bank(1);
    let mut scratch = vec![0.0f32; 2 * 2];
    let events = [
        event(RenderPluginEventKind::VoiceStart {
            voice: 9,
            sound: 0,
            gain: 1.0,
        }),
        event(RenderPluginEventKind::VoiceStart {
            voice: 0,
            sound: 9,
            gain: 1.0,
        }),
        event(RenderPluginEventKind::VoiceParam {
            voice: 0,
            param: RenderVoiceParam::HrirIndex,
            value: 99.0,
        }),
        event(RenderPluginEventKind::NoteOn {
            key: 60,
            velocity: 1.0,
        }),
    ];
    assert!(bank.process_with_events(&mut scratch, 2, 2, &events));
    assert_eq!(bank.unsupported_event_count(), 4);
}

#[test]
fn occlusion_lowpass_attenuates_high_frequencies() {
    // Nyquist-rate content through a 200 Hz one-pole must lose most of
    // its energy; the unoccluded path keeps it.
    let nyquist: Vec<f32> = (0..64)
        .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let bank = BinauralVoiceBank::new(
        vec![Arc::new(nyquist)],
        vec![(vec![1.0], vec![1.0])],
        1,
        4,
        SampleRate(48_000),
    );

    let energy = |bank: &BinauralVoiceBank, occlude: bool| -> f32 {
        bank.reset();
        let mut events = vec![
            event(RenderPluginEventKind::VoiceStart {
                voice: 0,
                sound: 0,
                gain: 1.0,
            }),
            event(RenderPluginEventKind::VoiceParam {
                voice: 0,
                param: RenderVoiceParam::HrirIndex,
                value: 0.0,
            }),
        ];
        if occlude {
            events.push(event(RenderPluginEventKind::VoiceParam {
                voice: 0,
                param: RenderVoiceParam::OcclusionCutoffHz,
                value: 200.0,
            }));
        }
        let mut scratch = vec![0.0f32; 64 * 2];
        assert!(bank.process_with_events(&mut scratch, 64, 2, &events));
        scratch.iter().map(|s| s * s).sum()
    };

    let open = energy(&bank, false);
    let occluded = energy(&bank, true);
    assert!(
        occluded < open * 0.05,
        "occluded energy {occluded} should be far below open {open}"
    );
}

#[test]
fn events_apply_at_their_frame_offset() {
    let bank = identity_bank(1);
    let mut scratch = vec![0.0f32; 16 * 2];
    // Start at frame 6; stop at frame 9 -> exactly three sounding frames.
    let events = [
        RenderBlockPluginEvent {
            offset_frames: 6,
            channel: 0,
            kind: RenderPluginEventKind::VoiceStart {
                voice: 0,
                sound: 0,
                gain: 1.0,
            },
        },
        RenderBlockPluginEvent {
            offset_frames: 6,
            channel: 0,
            kind: RenderPluginEventKind::VoiceParam {
                voice: 0,
                param: RenderVoiceParam::HrirIndex,
                value: 0.0,
            },
        },
        RenderBlockPluginEvent {
            offset_frames: 9,
            channel: 0,
            kind: RenderPluginEventKind::VoiceStop { voice: 0 },
        },
    ];
    assert!(bank.process_with_events(&mut scratch, 16, 2, &events));
    for frame in 0..6 {
        assert!(
            scratch[frame * 2].abs() < 1e-6,
            "pre-start frame {frame} sounded"
        );
    }
    for frame in 6..9 {
        assert!(
            (scratch[frame * 2] - 1.0).abs() < 1e-6,
            "frame {frame} should sound"
        );
    }
    for frame in 9..16 {
        assert!(
            scratch[frame * 2].abs() < 1e-6,
            "post-stop frame {frame} sounded"
        );
    }
}

#[test]
fn polyphony_sums_voices() {
    let bank = identity_bank(2);
    let mut scratch = vec![0.0f32; 2 * 2];
    let events = [
        event(RenderPluginEventKind::VoiceStart {
            voice: 0,
            sound: 0,
            gain: 1.0,
        }),
        event(RenderPluginEventKind::VoiceParam {
            voice: 0,
            param: RenderVoiceParam::HrirIndex,
            value: 0.0,
        }),
        event(RenderPluginEventKind::VoiceStart {
            voice: 1,
            sound: 0,
            gain: 0.5,
        }),
        event(RenderPluginEventKind::VoiceParam {
            voice: 1,
            param: RenderVoiceParam::HrirIndex,
            value: 0.0,
        }),
    ];
    assert!(bank.process_with_events(&mut scratch, 2, 2, &events));
    assert!((scratch[0] - 1.5).abs() < 1e-6);
    assert_eq!(bank.active_voices(), 2);
}
