//! Convolution reverb as a [`PluginBlockProcessor`] — promotes the reverb
//! that downstreams (Jetstream) run as a CPU post-process on the executor's
//! output into a real render-plane stage.
//!
//! Wraps one partitioned FFT convolver per stereo channel
//! ([`signal_dsp_spectral::StreamingConvolver`] — RT-safe, preallocated) and
//! mixes `dry · x + wet · conv(x)` in place on the stage scratch. The wet
//! path carries the streaming convolver's fixed latency (one partition
//! block); the dry path is left un-delayed, matching how send-style game
//! reverbs behave. Report the latency via
//! [`latency_frames`](PluginBlockProcessor::latency_frames) and let the host
//! decide whether to compensate.
//!
//! Stereo stages only; other channel counts bypass. `process` takes state
//! through a `try_lock` (contention only from control-thread `reset`; one
//! bypassed block is the correct degradation).

use std::sync::Mutex;

use signal_dsp_spectral::StreamingConvolver;

use crate::PluginBlockProcessor;

struct ReverbState {
    left: StreamingConvolver,
    right: StreamingConvolver,
    /// Deinterleave scratch, sized at construction (`max_frames`).
    left_buf: Vec<f32>,
    right_buf: Vec<f32>,
}

/// Stereo convolution reverb processor for a `Sum` stage.
pub struct ConvolutionReverbProcessor {
    state: Mutex<ReverbState>,
    wet: f32,
    dry: f32,
    latency: u32,
    max_frames: usize,
}

impl ConvolutionReverbProcessor {
    /// Build a reverb over per-channel impulse responses. `partition_block`
    /// sets the internal FFT partition (and therefore the wet-path latency);
    /// `max_frames` is the largest block `process` will ever see (allocation
    /// happens here, never on the audio thread).
    pub fn new(
        left_ir: &[f32],
        right_ir: &[f32],
        wet: f32,
        dry: f32,
        partition_block: usize,
        max_frames: usize,
    ) -> Self {
        let left = StreamingConvolver::new(left_ir, partition_block);
        let right = StreamingConvolver::new(right_ir, partition_block);
        let latency = left.latency() as u32;
        Self {
            state: Mutex::new(ReverbState {
                left,
                right,
                left_buf: vec![0.0; max_frames],
                right_buf: vec![0.0; max_frames],
            }),
            wet,
            dry,
            latency,
            max_frames,
        }
    }

    /// Clear convolver histories (control thread).
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.left.reset();
            state.right.reset();
        }
    }
}

impl PluginBlockProcessor for ConvolutionReverbProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        if channels != 2 || frame_count > self.max_frames {
            return false;
        }
        let Ok(mut state) = self.state.try_lock() else {
            return false;
        };
        let state = &mut *state;

        for frame in 0..frame_count {
            state.left_buf[frame] = scratch[frame * 2];
            state.right_buf[frame] = scratch[frame * 2 + 1];
        }
        state.left.process_in_place(&mut state.left_buf[..frame_count]);
        state.right.process_in_place(&mut state.right_buf[..frame_count]);
        for frame in 0..frame_count {
            let dry_l = scratch[frame * 2];
            let dry_r = scratch[frame * 2 + 1];
            scratch[frame * 2] = self.dry * dry_l + self.wet * state.left_buf[frame];
            scratch[frame * 2 + 1] = self.dry * dry_r + self.wet * state.right_buf[frame];
        }
        true
    }

    fn latency_frames(&self) -> u32 {
        self.latency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wet_path_reproduces_ir_at_reported_latency() {
        // IR = unit impulse: the wet path is identity delayed by the
        // convolver's latency. Feed an impulse, expect it back `latency`
        // frames later at wet gain.
        let reverb = ConvolutionReverbProcessor::new(&[1.0], &[1.0], 1.0, 0.0, 16, 128);
        let latency = reverb.latency_frames() as usize;

        let mut scratch = vec![0.0f32; 128 * 2];
        scratch[0] = 1.0; // left impulse, frame 0
        scratch[1] = 0.5; // right impulse
        assert!(reverb.process(&mut scratch, 128, 2));

        assert!((scratch[latency * 2] - 1.0).abs() < 1e-4, "left at latency");
        assert!((scratch[latency * 2 + 1] - 0.5).abs() < 1e-4, "right at latency");
        // Nothing before the latency point.
        for frame in 0..latency {
            assert!(scratch[frame * 2].abs() < 1e-5, "pre-latency frame {frame}");
        }
    }

    #[test]
    fn dry_path_is_undelayed_and_mix_sums() {
        let reverb = ConvolutionReverbProcessor::new(&[1.0], &[1.0], 0.5, 0.5, 16, 64);
        let mut scratch = vec![0.0f32; 64 * 2];
        scratch[0] = 1.0;
        assert!(reverb.process(&mut scratch, 64, 2));
        // Dry half arrives at frame 0.
        assert!((scratch[0] - 0.5).abs() < 1e-4, "dry now, got {}", scratch[0]);
        // Wet half arrives at latency.
        let latency = reverb.latency_frames() as usize;
        assert!((scratch[latency * 2] - 0.5).abs() < 1e-4, "wet later");
    }

    #[test]
    fn tail_rings_across_blocks() {
        // Two-tap IR spreads energy into the following block.
        let ir = [0.0f32; 24]
            .iter()
            .copied()
            .chain([0.8f32])
            .collect::<Vec<_>>();
        let reverb = ConvolutionReverbProcessor::new(&ir, &ir, 1.0, 0.0, 16, 32);
        let mut first = vec![0.0f32; 32 * 2];
        first[0] = 1.0;
        assert!(reverb.process(&mut first, 32, 2));
        let mut second = vec![0.0f32; 32 * 2];
        assert!(reverb.process(&mut second, 32, 2));
        // Single 0.8 tap at IR offset 24 + latency 16 = frame 40 -> lands in
        // the SECOND block (frame 8 there), proving state crosses blocks.
        let first_energy: f32 = first.iter().map(|s| s.abs()).sum();
        assert!(first_energy < 1e-4, "first block should be silent, got {first_energy}");
        assert!((second[8 * 2] - 0.8).abs() < 1e-3, "tap at second-block frame 8: {}", second[8 * 2]);
    }

    #[test]
    fn non_stereo_and_oversize_blocks_bypass() {
        let reverb = ConvolutionReverbProcessor::new(&[1.0], &[1.0], 1.0, 0.0, 16, 32);
        let mut mono = vec![0.3f32; 16];
        assert!(!reverb.process(&mut mono, 16, 1));
        assert!(mono.iter().all(|&s| (s - 0.3).abs() < 1e-6));
        let mut big = vec![0.0f32; 64 * 2];
        assert!(!reverb.process(&mut big, 64, 2));
    }
}
