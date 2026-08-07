use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::plan::RenderPlan;
use crate::sample_buffer::XRUN_INTERVAL_FACTOR;
use crate::METER_SLOT_CAPACITY;

use super::super::command::SharedState;
use super::RenderPlaneExecutor;

impl RenderPlaneExecutor {
    /// Publish callback-health counters: count, duration (last/max), and
    /// inferred xruns. An xrun is an interval since the previous callback
    /// longer than [`XRUN_INTERVAL_FACTOR`] × the block duration at the
    /// active plan's rate; without a plan no xrun can be inferred (the
    /// expected cadence is unknown) but count and duration still publish.
    pub(in crate::plane::executor) fn publish_callback_health(
        &mut self,
        callback_start: Instant,
        samples_len: usize,
    ) {
        let shared = &self.shared;
        shared.callback_count.fetch_add(1, Ordering::Relaxed);
        let duration_micros = callback_start.elapsed().as_micros() as u64;
        shared
            .last_callback_duration_micros
            .store(duration_micros, Ordering::Relaxed);
        shared
            .max_callback_duration_micros
            .fetch_max(duration_micros, Ordering::Relaxed);
        if let (Some(previous), Some(plan)) = (self.last_callback_instant, self.plan.as_ref()) {
            let stream_channels = self
                .stream_channels
                .map(|channels| channels as usize)
                .unwrap_or(plan.stream_channels)
                .max(1);
            let frame_count = samples_len / stream_channels;
            let block_seconds = frame_count as f64 / plan.sample_rate_hz.max(1) as f64;
            let interval = callback_start.duration_since(previous).as_secs_f64();
            if frame_count > 0 && interval > block_seconds * XRUN_INTERVAL_FACTOR {
                shared.xrun_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        self.last_callback_instant = Some(callback_start);
    }

    /// Write per-stage peak/RMS for this block into the shared meter table
    /// and stamp the plan generation. Levels are taken from each stage's
    /// scratch (pre-consumption) and scaled by the stage's end-of-block
    /// smoothed gain so fader moves read on the meters — a per-block
    /// approximation of the post-fader level (the transport edge ramp is
    /// not included). Cheap loops over already-rendered scratch: no
    /// allocation. Stages past [`METER_SLOT_CAPACITY`] are unmetered.
    pub(in crate::plane::executor) fn publish_meters(
        shared: &SharedState,
        plan: &RenderPlan,
        frame_count: usize,
    ) {
        for (index, stage) in plan.stages.iter().take(METER_SLOT_CAPACITY).enumerate() {
            let samples = &stage.scratch[..frame_count * stage.channels];
            let mut peak = 0.0f32;
            let mut sum_squares = 0.0f32;
            for sample in samples {
                let magnitude = sample.abs();
                if magnitude > peak {
                    peak = magnitude;
                }
                sum_squares += sample * sample;
            }
            let gain = stage.gain_current.abs();
            let rms = (sum_squares / samples.len().max(1) as f32).sqrt() * gain;
            let slot = &shared.meter_slots[index];
            slot.peak_bits
                .store((peak * gain).to_bits(), Ordering::Relaxed);
            slot.rms_bits.store(rms.to_bits(), Ordering::Relaxed);
        }
        shared
            .meter_generation
            .store(plan.generation, Ordering::Relaxed);
    }

    /// Zero the active plan's meter slots (silence: stopped or fully ramped
    /// out) and stamp the generation so readers see live zeros, not stale
    /// levels from the last audible block.
    pub(in crate::plane::executor) fn publish_silent_meters(
        shared: &SharedState,
        plan: &RenderPlan,
    ) {
        for slot in shared
            .meter_slots
            .iter()
            .take(plan.stages.len().min(METER_SLOT_CAPACITY))
        {
            slot.peak_bits.store(0, Ordering::Relaxed);
            slot.rms_bits.store(0, Ordering::Relaxed);
        }
        shared
            .meter_generation
            .store(plan.generation, Ordering::Relaxed);
    }
}
