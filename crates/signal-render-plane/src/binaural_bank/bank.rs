use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use signal_dsp::{BinauralConvolver, DspKernel as _, OnePoleLowPass};
use signal_primitives::{FrequencyHz, SampleRate};

use crate::{RenderBlockPluginEvent, RenderPluginEventKind, RenderVoiceParam};

use super::types::{BankHrir, BankSound, BankState, VoiceSlot, OCCLUSION_OPEN_HZ};

/// A binaural voice-bank processor. Install on a stereo `Sum` stage with
/// `accepts_live_events`, then drive it with the `Voice*` event family.
pub struct BinauralVoiceBank {
    sounds: Arc<Vec<BankSound>>,
    hrirs: Arc<Vec<BankHrir>>,
    state: Mutex<BankState>,
    pub(crate) unsupported_events: AtomicU64,
}

impl std::fmt::Debug for BinauralVoiceBank {
    /// Reports bank shape and the unsupported-event counter. The voice state
    /// is behind a mutex that the render path holds per block, so `fmt` does
    /// not take it.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BinauralVoiceBank")
            .field("sounds", &self.sounds.len())
            .field("hrirs", &self.hrirs.len())
            .field(
                "unsupported_events",
                &self
                    .unsupported_events
                    .load(std::sync::atomic::Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
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

    pub(crate) fn render(
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
            while next_event < events.len() && (events[next_event].offset_frames as usize) <= cursor
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
            RenderPluginEventKind::VoiceParam {
                voice,
                param,
                value,
            } => {
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
