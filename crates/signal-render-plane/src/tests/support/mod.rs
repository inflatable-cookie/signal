//! Shared test fixtures and helpers.

mod plan;
mod processors;
mod stream;

pub(in crate::tests) use plan::*;
pub(in crate::tests) use processors::*;
pub(in crate::tests) use stream::*;

use super::*;

/// Run blocks until the transport edge ramp has fully opened.
pub(super) fn warm_up(executor: &mut RenderPlaneExecutor, blocks: usize) {
    let mut frames = [0.0f32; 512];
    for _ in 0..blocks {
        executor.render_block(&mut frames);
    }
}
/// DC-1.0 stereo samples clip filling its window exactly, with fades.
pub(super) fn dc_clip(
    clip_id: u64,
    start_frames: u64,
    end_frames: u64,
    fade_in_frames: u32,
    fade_out_frames: u32,
) -> RenderClipSpec {
    let frames = (end_frames - start_frames) as usize;
    RenderClipSpec {
        clip_id,
        start_frames,
        end_frames,
        source: RenderSource::Samples(RenderSampleBuffer::stereo(
            48_000,
            vec![1.0f32; frames * 2].into(),
        )),
        loop_source: false,
        fade_in_frames,
        fade_out_frames,
    }
}
pub(super) fn max_left_step(frames: &[f32]) -> f32 {
    frames
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0f32, f32::max)
}

pub(super) fn event_buffer(events: Vec<RenderPluginEvent>) -> RenderPluginEventBuffer {
    RenderPluginEventBuffer {
        events: events.into(),
    }
}

pub(super) fn live_note_on(frame: u64, key: u8, velocity: f32) -> RenderPluginEvent {
    RenderPluginEvent {
        frame,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn { key, velocity },
    }
}

pub(super) fn live_note_off(frame: u64, key: u8) -> RenderPluginEvent {
    RenderPluginEvent {
        frame,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key },
    }
}
pub(super) fn note(
    start_frame: u64,
    duration_frames: u64,
    degree: i32,
    velocity: f32,
) -> RenderNote {
    RenderNote {
        start_frame,
        duration_frames,
        degree,
        pitch_intent: None,
        velocity,
    }
}
pub(super) fn note_buffer(notes: Vec<RenderNote>) -> RenderNoteBuffer {
    RenderNoteBuffer {
        notes: notes.into(),
    }
}

/// Offline-render `frame_count` frames of `spec` from `start_frame` and
/// return the LEFT channel (channels are identical for note sources).
pub(super) fn render_notes_left(
    spec: &RenderPlanSpec,
    start_frame: u64,
    frame_count: u64,
) -> Vec<f32> {
    let output = crate::render_plan_to_pcm(
        spec,
        &crate::OfflineRenderOptions {
            start_frame,
            frame_count,
            ..crate::OfflineRenderOptions::default()
        },
    )
    .expect("offline note render");
    output
        .master
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect()
}
pub(super) fn soak_tests_enabled() -> bool {
    if std::env::var("SIGNAL_SOAK_TESTS").as_deref() == Ok("1") {
        return true;
    }
    eprintln!("SKIPPED: wall-clock soak test; set SIGNAL_SOAK_TESTS=1 (or run `effigy test:soak`)");
    false
}
/// FNV-1a 64 over the bit pattern of rendered samples.
pub(super) fn fnv1a_hash_pcm(frames: &[f32]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for sample in frames {
        for byte in sample.to_bits().to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}
/// Recorded output hash for `golden_render_hash_is_stable` (captured on
/// the test's first run; see the regeneration note in the test body).
pub(super) const GOLDEN_RENDER_HASH: u64 = 0x494b_7128_ef17_1a6a;
