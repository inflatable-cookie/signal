use rustfft::num_complex::Complex32;

use crate::realtime_preview::{
    RealtimePreviewCallbackTimelineMode, RealtimePreviewIntegrationMode,
    RealtimePreviewStreamingContract,
};

use super::constants::{
    REALTIME_PREVIEW_STREAM_MAX_RATIO, REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES,
};
use super::types::RealtimePreviewStreamState;

impl RealtimePreviewStreamState {
    /// Validated stream configuration.
    pub fn config(&self) -> crate::realtime_preview::RealtimePreviewStreamConfig {
        self.config
    }

    /// Routing contract for this kernel.
    ///
    /// `audio_thread_processing_supported` is derived from the properties the
    /// `g10.040` Batch 40.3 gates prove, not asserted as a constant:
    ///
    /// - the callback allocates nothing (`G1`)
    /// - work per callback is bounded, which holds only because `render`
    ///   rejects ratios outside `[0.25, 3.0]` rather than clamping them (`G2`)
    /// - source consumption follows the ratio with nothing dropped (`G3`, `G4`)
    /// - starvation is reported rather than hidden (`G5`)
    /// - ratio changes land within one analysis hop (`G6`)
    ///
    /// Each is a property of *this* kernel. The shipped
    /// [`crate::RealtimePreviewCallbackState`] keeps reporting `QuantumLocked`
    /// and unsupported, because it is quantum-locked and does drop source.
    ///
    /// The envelope below is what the state can still get wrong at run time,
    /// so it is checked rather than assumed: a configuration outside it means
    /// the gates' proof does not apply and the contract says unsupported.
    pub fn contract(&self) -> RealtimePreviewStreamingContract {
        let within_envelope = (1..=2).contains(&self.config.channel_count)
            && self.config.max_block_frames > 0
            && self.config.analysis_hop > 0
            && self.config.window_size >= self.config.analysis_hop
            // The overlap law is what bounds the maximum ratio, so a geometry
            // that violates it invalidates the frozen range.
            && (self.config.analysis_hop as f64) * REALTIME_PREVIEW_STREAM_MAX_RATIO
                <= 0.75 * self.config.window_size as f64
            && self.working_bytes() <= REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES;

        RealtimePreviewStreamingContract {
            input_latency_frames: self.source_ring_frames,
            output_latency_frames: self.config.window_size,
            ratio_change_alignment_tolerance_frames: self.config.analysis_hop,
            integration_mode: if within_envelope {
                RealtimePreviewIntegrationMode::CallbackSafeStreaming
            } else {
                RealtimePreviewIntegrationMode::AnticipativePreRender
            },
            callback_timeline_mode: if within_envelope {
                RealtimePreviewCallbackTimelineMode::SourceProjected
            } else {
                RealtimePreviewCallbackTimelineMode::QuantumLocked
            },
            audio_thread_processing_supported: within_envelope,
            unsupported_mode: None,
            config: self.config,
        }
    }

    /// Bytes the state holds, against the frozen ceiling.
    pub fn working_bytes(&self) -> usize {
        let sample = std::mem::size_of::<signal_primitives::Sample>();
        let complex = std::mem::size_of::<Complex32>();
        (self.source_ring.len() + self.output_ring.len() + self.normalization_ring.len()) * sample
            + (self.window.len() + self.omega.len()) * sample
            + (self.analysis_buffer.len() + self.synthesis_spectrum.len()) * complex
            + (self.forward_fft_scratch.len() + self.inverse_fft_scratch.len()) * complex
            + (self.previous_phase.len()
                + self.synthesis_phase.len()
                + self.current_magnitudes.len()
                + self.current_phases.len()
                + self.previous_magnitudes.len())
                * sample
            + self.current_peak_bins.capacity() * std::mem::size_of::<usize>()
            + (self.current_energy.len() + self.previous_energy.len()) * std::mem::size_of::<f64>()
    }

    /// Source frames the producer must fill before playback starts, and keep
    /// ahead of the read cursor afterwards.
    pub fn prefill_frames(&self) -> usize {
        self.source_ring_frames
    }

    /// Reported algorithmic latency: one analysis window plus the prefill.
    ///
    /// Constant for a configuration. This is a start-up delay before preview
    /// playback begins rather than a round-trip cost, because preview plays
    /// back a stored asset rather than monitoring a live signal.
    pub fn latency_frames(&self) -> u64 {
        self.config.window_size as u64 + self.source_ring_frames as u64
    }

    /// Absolute source frame index the producer must fill to.
    pub fn source_demand_frame(&self) -> u64 {
        self.next_analysis_frame
            .saturating_add(self.source_ring_frames as u64)
    }

    /// Source frames accepted so far.
    pub fn source_write_frame(&self) -> u64 {
        self.source_write_frame
    }

    /// Whether enough source is buffered for playback to start.
    pub fn ready(&self) -> bool {
        self.source_write_frame >= self.config.window_size as u64
    }
}
