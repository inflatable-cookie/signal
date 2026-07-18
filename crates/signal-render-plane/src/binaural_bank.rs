//! Binaural voice bank: a [`PluginBlockProcessor`] hosting N one-shot voice
//! slots, each rendered through its own crossfading HRTF convolver — the
//! "option B" per-voice model from
//! `docs/research/binaural-render-plane-integration-v1.md`.
//!
//! Voices live *inside* the processor, so spawning a game sound is a live
//! event, not a plan recompile: `VoiceStart { voice, sound, gain }` begins a
//! preloaded mono sound on a slot, `VoiceParam { HrirIndex }` retargets the
//! slot's ear responses (crossfaded by [`signal_dsp::BinauralConvolver`]) as
//! the source moves, `VoiceStop` silences it, and a voice frees itself when
//! its sound ends. Slot allocation and stealing policy stay with the sender
//! (the game engine) — the bank plays what it is told on the slot it is
//! told.
//!
//! The bank **adds** its stereo output into the stage scratch (composes with
//! whatever the Sum stage already carries). Stereo stages only; any other
//! channel count bypasses. Events apply **sample-accurately**: the block is
//! rendered in segments split at each event's `offset_frames`, so a
//! `VoiceStart` at offset 96 begins exactly there.
//!
//! Real-time safety: sounds and HRIR tables are `Arc`-shared and immutable;
//! all per-voice state is preallocated at construction. `process` takes the
//! state through a `try_lock` (never blocks — contention can only come from
//! a control-thread `reset`, which is rare and tolerates one bypassed
//! block).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use signal_dsp::{BinauralConvolver, DspKernel as _, OnePoleLowPass};
use signal_primitives::{FrequencyHz, SampleRate};

use crate::{
    PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventKind, RenderVoiceParam,
};

/// One preloaded mono sound.
pub type BankSound = Arc<Vec<f32>>;

/// One HRIR ear pair (left taps, right taps).
pub type BankHrir = (Vec<f32>, Vec<f32>);

/// Cutoffs at/above this disable the occlusion filter entirely.
const OCCLUSION_OPEN_HZ: f32 = 20_000.0;

struct VoiceSlot {
    convolver: BinauralConvolver,
    occlusion: OnePoleLowPass,
    occluded: bool,
    sound: Option<BankSound>,
    playhead: usize,
    gain: f32,
    /// Which HRIR index is loaded (dedup: re-selecting it is a no-op).
    hrir_index: Option<usize>,
}

impl VoiceSlot {
    fn active(&self) -> bool {
        self.sound.is_some()
    }

    fn stop(&mut self) {
        self.sound = None;
        self.playhead = 0;
    }
}

struct BankState {
    slots: Vec<VoiceSlot>,
}

/// A binaural voice-bank processor. Install on a stereo `Sum` stage with
/// `accepts_live_events`, then drive it with the `Voice*` event family.
pub struct BinauralVoiceBank {
    sounds: Arc<Vec<BankSound>>,
    hrirs: Arc<Vec<BankHrir>>,
    state: Mutex<BankState>,
    unsupported_events: AtomicU64,
}

impl BinauralVoiceBank {
    /// Build a bank with `max_voices` slots over preloaded `sounds` and an
    /// HRIR table. `crossfade_samples` is the per-slot response-swap window
    /// (see [`signal_dsp::DEFAULT_HRIR_CROSSFADE_SAMPLES`]). All allocation
    /// happens here.
    pub fn new(
        sounds: Vec<BankSound>,
        hrirs: Vec<BankHrir>,
        max_voices: usize,
        crossfade_samples: usize,
        sample_rate: SampleRate,
    ) -> Self {
        let max_taps = hrirs
            .iter()
            .map(|(l, r)| l.len().max(r.len()))
            .max()
            .unwrap_or(1);
        let slots = (0..max_voices)
            .map(|_| VoiceSlot {
                convolver: BinauralConvolver::with_capacity(max_taps, crossfade_samples),
                occlusion: OnePoleLowPass::new(sample_rate, FrequencyHz(OCCLUSION_OPEN_HZ)),
                occluded: false,
                sound: None,
                playhead: 0,
                gain: 1.0,
                hrir_index: None,
            })
            .collect();
        Self {
            sounds: Arc::new(sounds),
            hrirs: Arc::new(hrirs),
            state: Mutex::new(BankState { slots }),
            unsupported_events: AtomicU64::new(0),
        }
    }

    /// Number of currently sounding voices (control-thread telemetry).
    pub fn active_voices(&self) -> usize {
        self.state
            .lock()
            .map(|state| state.slots.iter().filter(|slot| slot.active()).count())
            .unwrap_or(0)
    }

    /// Silence every slot and clear convolver histories (control thread).
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            for slot in &mut state.slots {
                slot.stop();
                slot.convolver.reset();
                slot.hrir_index = None;
                slot.occluded = false;
                slot.occlusion.reset();
            }
        }
    }

    fn apply_event(&self, state: &mut BankState, event: &RenderBlockPluginEvent) {
        match event.kind {
            RenderPluginEventKind::VoiceStart { voice, sound, gain } => {
                let (Some(slot), Some(sound)) = (
                    state.slots.get_mut(voice as usize),
                    self.sounds.get(sound as usize),
                ) else {
                    self.unsupported_events.fetch_add(1, Ordering::Relaxed);
                    return;
                };
                slot.sound = Some(Arc::clone(sound));
                slot.playhead = 0;
                slot.gain = gain.max(0.0);
                // A restarted slot must open on its next selected direction,
                // not fade from the previous voice's response — and start
                // unoccluded.
                slot.hrir_index = None;
                slot.occluded = false;
                slot.occlusion.reset();
            }
            RenderPluginEventKind::VoiceStop { voice } => {
                match state.slots.get_mut(voice as usize) {
                    Some(slot) => slot.stop(),
                    None => {
                        self.unsupported_events.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            RenderPluginEventKind::VoiceParam { voice, param, value } => {
                let Some(slot) = state.slots.get_mut(voice as usize) else {
                    self.unsupported_events.fetch_add(1, Ordering::Relaxed);
                    return;
                };
                match param {
                    RenderVoiceParam::Gain => slot.gain = value.max(0.0),
                    RenderVoiceParam::OcclusionCutoffHz => {
                        if value >= OCCLUSION_OPEN_HZ {
                            slot.occluded = false;
                        } else {
                            slot.occluded = true;
                            slot.occlusion.set_cutoff_hz(FrequencyHz(value.max(20.0)));
                        }
                    }
                    RenderVoiceParam::HrirIndex => {
                        let index = value as usize;
                        let Some((left, right)) = self.hrirs.get(index) else {
                            self.unsupported_events.fetch_add(1, Ordering::Relaxed);
                            return;
                        };
                        if slot.hrir_index == Some(index) {
                            return; // same measurement cell — nothing to do
                        }
                        if slot.hrir_index.is_none() {
                            slot.convolver.set_response(left, right); // first direction snaps
                        } else {
                            slot.convolver.crossfade_to(left, right);
                        }
                        slot.hrir_index = Some(index);
                    }
                }
            }
            // Non-voice events are not this processor's vocabulary.
            _ => {
                self.unsupported_events.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn render(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        if channels != 2 {
            return false; // bypass: this bank renders binaural stereo only
        }
        // Never block the audio thread; contention (control-thread reset) is
        // rare and one bypassed block is the correct degradation.
        let Ok(mut state) = self.state.try_lock() else {
            return false;
        };

        // Sample-accurate event application: walk the block in segments split
        // at event offsets (the executor delivers events sorted by
        // `offset_frames`, all `< frame_count`).
        let mut cursor = 0usize;
        let mut next_event = 0usize;
        while cursor < frame_count {
            while next_event < events.len()
                && (events[next_event].offset_frames as usize) <= cursor
            {
                self.apply_event(&mut state, &events[next_event]);
                next_event += 1;
            }
            let segment_end = events
                .get(next_event)
                .map(|event| (event.offset_frames as usize).min(frame_count))
                .unwrap_or(frame_count)
                .max(cursor + 1);
            Self::render_segment(&mut state, scratch, cursor, segment_end);
            cursor = segment_end;
        }
        // Contract says offsets < frame_count, but drain defensively so a
        // misbehaving sender cannot wedge events forever.
        while next_event < events.len() {
            self.apply_event(&mut state, &events[next_event]);
            next_event += 1;
        }
        true
    }

    /// Render frames `[start, end)` of the current block for every slot.
    fn render_segment(state: &mut BankState, scratch: &mut [f32], start: usize, end: usize) {
        for slot in &mut state.slots {
            let Some(sound) = slot.sound.as_ref().map(Arc::clone) else {
                continue;
            };
            let samples = &sound[slot.playhead.min(sound.len())..];
            let take = samples.len().min(end - start);
            for (offset, &sample) in samples[..take].iter().enumerate() {
                let mut sample = sample * slot.gain;
                if slot.occluded {
                    sample = slot.occlusion.process_sample(sample);
                }
                let (l, r) = slot.convolver.process_sample(sample);
                let frame = start + offset;
                scratch[frame * 2] += l;
                scratch[frame * 2 + 1] += r;
            }
            // Let the convolver tail ring out past the sound's end within
            // this segment (silence input keeps the FIR draining).
            for frame in (start + take)..end {
                let (l, r) = slot.convolver.process_sample(0.0);
                scratch[frame * 2] += l;
                scratch[frame * 2 + 1] += r;
            }
            slot.playhead += take;
            if slot.playhead >= sound.len() {
                slot.stop(); // fire-and-forget: the voice frees itself
            }
        }
    }
}

impl PluginBlockProcessor for BinauralVoiceBank {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.render(scratch, frame_count, channels, &[])
    }

    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.render(scratch, frame_count, channels, events)
    }

    fn unsupported_event_count(&self) -> u64 {
        self.unsupported_events.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: RenderPluginEventKind) -> RenderBlockPluginEvent {
        RenderBlockPluginEvent { offset_frames: 0, channel: 0, kind }
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
            event(RenderPluginEventKind::VoiceStart { voice: 0, sound: 0, gain: 0.5 }),
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
            event(RenderPluginEventKind::VoiceStart { voice: 0, sound: 0, gain: 1.0 }),
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
            event(RenderPluginEventKind::VoiceStart { voice: 0, sound: 0, gain: 1.0 }),
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
            event(RenderPluginEventKind::VoiceStart { voice: 0, sound: 0, gain: 1.0 }),
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
            event(RenderPluginEventKind::VoiceStart { voice: 9, sound: 0, gain: 1.0 }),
            event(RenderPluginEventKind::VoiceStart { voice: 0, sound: 9, gain: 1.0 }),
            event(RenderPluginEventKind::VoiceParam {
                voice: 0,
                param: RenderVoiceParam::HrirIndex,
                value: 99.0,
            }),
            event(RenderPluginEventKind::NoteOn { key: 60, velocity: 1.0 }),
        ];
        assert!(bank.process_with_events(&mut scratch, 2, 2, &events));
        assert_eq!(bank.unsupported_event_count(), 4);
    }

    #[test]
    fn occlusion_lowpass_attenuates_high_frequencies() {
        // Nyquist-rate content through a 200 Hz one-pole must lose most of
        // its energy; the unoccluded path keeps it.
        let nyquist: Vec<f32> = (0..64).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
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
                event(RenderPluginEventKind::VoiceStart { voice: 0, sound: 0, gain: 1.0 }),
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
                kind: RenderPluginEventKind::VoiceStart { voice: 0, sound: 0, gain: 1.0 },
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
            assert!(scratch[frame * 2].abs() < 1e-6, "pre-start frame {frame} sounded");
        }
        for frame in 6..9 {
            assert!((scratch[frame * 2] - 1.0).abs() < 1e-6, "frame {frame} should sound");
        }
        for frame in 9..16 {
            assert!(scratch[frame * 2].abs() < 1e-6, "post-stop frame {frame} sounded");
        }
    }

    #[test]
    fn polyphony_sums_voices() {
        let bank = identity_bank(2);
        let mut scratch = vec![0.0f32; 2 * 2];
        let events = [
            event(RenderPluginEventKind::VoiceStart { voice: 0, sound: 0, gain: 1.0 }),
            event(RenderPluginEventKind::VoiceParam {
                voice: 0,
                param: RenderVoiceParam::HrirIndex,
                value: 0.0,
            }),
            event(RenderPluginEventKind::VoiceStart { voice: 1, sound: 0, gain: 0.5 }),
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
}
