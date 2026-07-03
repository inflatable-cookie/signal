//! Shared-memory audio block layout for out-of-process plugin processing.
//!
//! One region per hosted plugin instance, created by the sandbox child at
//! activate time through the existing lease machinery. Both sides map the
//! same file and speak a sequence-stamped request/response protocol:
//!
//! ```text
//! header (64 bytes):
//!   [0..4)   request_seq  (AtomicU32) — parent bumps after writing input
//!   [4..8)   response_seq (AtomicU32) — child sets = request_seq after
//!            writing output
//!   [8..12)  frame_count  (AtomicU32) — frames in the current request
//!   [12..16) channels     (AtomicU32) — interleaved channel count (2 in v1)
//!   [16..20) flags        (AtomicU32) — reserved, zero
//!   [20..64) reserved
//! input  samples: max_frames × channels interleaved f32
//! output samples: max_frames × channels interleaved f32
//! ```
//!
//! The parent writes input, publishes `request_seq` (release), and
//! bounded-spin-waits for `response_seq` to catch up (acquire). The child
//! spin/yield-waits on `request_seq`, processes, writes output, and publishes
//! `response_seq` (release). Neither side allocates or blocks on the other
//! beyond its wait budget.

use std::sync::atomic::{AtomicU32, Ordering};

/// Size of the block header preceding the sample areas.
pub const PLUGIN_AUDIO_BLOCK_HEADER_BYTES: usize = 64;

const REQUEST_SEQ_OFFSET: usize = 0;
const RESPONSE_SEQ_OFFSET: usize = 4;
const FRAME_COUNT_OFFSET: usize = 8;
const CHANNELS_OFFSET: usize = 12;

/// Fixed layout of one plugin audio block region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginAudioBlockLayout {
    /// Largest block the region can carry.
    pub max_frames: u32,
    /// Interleaved channel count of both sample areas.
    pub channels: u32,
}

impl PluginAudioBlockLayout {
    /// Total region size in bytes for this layout.
    pub fn region_bytes(&self) -> u32 {
        let samples = self.max_frames as usize * self.channels as usize * 4;
        (PLUGIN_AUDIO_BLOCK_HEADER_BYTES + 2 * samples) as u32
    }

    /// Byte offset of the input sample area.
    pub fn input_offset(&self) -> usize {
        PLUGIN_AUDIO_BLOCK_HEADER_BYTES
    }

    /// Byte offset of the output sample area.
    pub fn output_offset(&self) -> usize {
        PLUGIN_AUDIO_BLOCK_HEADER_BYTES + self.max_frames as usize * self.channels as usize * 4
    }
}

/// A live view over a mapped plugin audio block region.
///
/// Holds a raw base pointer into the caller's mapping; the caller must keep
/// the mapping alive (and unmoved) for the view's lifetime, and at most one
/// side may write each area per the protocol above.
#[derive(Debug)]
pub struct PluginAudioBlockView {
    base: *mut u8,
    layout: PluginAudioBlockLayout,
}

// Safety: the view is a typed window over shared memory whose concurrent
// access is serialized by the request/response sequence stamps; the raw
// pointer itself is safe to move across threads.
unsafe impl Send for PluginAudioBlockView {}
unsafe impl Sync for PluginAudioBlockView {}

impl PluginAudioBlockView {
    /// Wrap a mapped region.
    ///
    /// # Safety
    ///
    /// `base` must point at a live mapping of at least
    /// [`PluginAudioBlockLayout::region_bytes`] bytes that outlives the view,
    /// and both processes must agree on `layout`.
    pub unsafe fn new(base: *mut u8, layout: PluginAudioBlockLayout) -> Self {
        debug_assert!(!base.is_null());
        Self { base, layout }
    }

    /// The region layout this view was built with.
    pub fn layout(&self) -> PluginAudioBlockLayout {
        self.layout
    }

    fn atomic_at(&self, offset: usize) -> &AtomicU32 {
        // Safety: offset is within the 64-byte header, 4-aligned, and the
        // mapping outlives the view per the `new` contract.
        unsafe { &*(self.base.add(offset).cast::<AtomicU32>()) }
    }

    /// Request sequence stamp (parent-published).
    pub fn request_seq(&self) -> &AtomicU32 {
        self.atomic_at(REQUEST_SEQ_OFFSET)
    }

    /// Response sequence stamp (child-published).
    pub fn response_seq(&self) -> &AtomicU32 {
        self.atomic_at(RESPONSE_SEQ_OFFSET)
    }

    /// Frame count of the current request.
    pub fn frame_count(&self) -> &AtomicU32 {
        self.atomic_at(FRAME_COUNT_OFFSET)
    }

    /// Channel count both sample areas are interleaved at.
    pub fn channels(&self) -> &AtomicU32 {
        self.atomic_at(CHANNELS_OFFSET)
    }

    /// Copy `samples` into the input area (parent side).
    ///
    /// # Safety
    ///
    /// Caller must own the parent role of the protocol: the child must not
    /// be reading the input area (i.e. the previous request has completed or
    /// timed out and a new `request_seq` has not yet been published).
    pub unsafe fn write_input(&self, samples: &[f32]) {
        // Safety: per the method contract; offsets stay inside the region.
        unsafe {
            let capacity = self.layout.max_frames as usize * self.layout.channels as usize;
            let count = samples.len().min(capacity);
            let dest = self.base.add(self.layout.input_offset()).cast::<f32>();
            std::ptr::copy_nonoverlapping(samples.as_ptr(), dest, count);
        }
    }

    /// Copy the first `samples.len()` output samples out (parent side).
    ///
    /// # Safety
    ///
    /// Caller must have observed `response_seq == request_seq` (acquire) for
    /// the request whose output it reads.
    pub unsafe fn read_output(&self, samples: &mut [f32]) {
        // Safety: per the method contract; offsets stay inside the region.
        unsafe {
            let capacity = self.layout.max_frames as usize * self.layout.channels as usize;
            let count = samples.len().min(capacity);
            let source = self.base.add(self.layout.output_offset()).cast::<f32>();
            std::ptr::copy_nonoverlapping(source, samples.as_mut_ptr(), count);
        }
    }

    /// Copy the first `samples.len()` input samples out (child side).
    ///
    /// # Safety
    ///
    /// Caller must have observed a fresh `request_seq` (acquire) before
    /// reading the request's input.
    pub unsafe fn read_input(&self, samples: &mut [f32]) {
        // Safety: per the method contract; offsets stay inside the region.
        unsafe {
            let capacity = self.layout.max_frames as usize * self.layout.channels as usize;
            let count = samples.len().min(capacity);
            let source = self.base.add(self.layout.input_offset()).cast::<f32>();
            std::ptr::copy_nonoverlapping(source, samples.as_mut_ptr(), count);
        }
    }

    /// Copy `samples` into the output area (child side).
    ///
    /// # Safety
    ///
    /// Caller must own the child role and publish `response_seq` only after
    /// this write (release ordering).
    pub unsafe fn write_output(&self, samples: &[f32]) {
        // Safety: per the method contract; offsets stay inside the region.
        unsafe {
            let capacity = self.layout.max_frames as usize * self.layout.channels as usize;
            let count = samples.len().min(capacity);
            let dest = self.base.add(self.layout.output_offset()).cast::<f32>();
            std::ptr::copy_nonoverlapping(samples.as_ptr(), dest, count);
        }
    }

    /// Initialize the header for a fresh region (child side, before any
    /// request): zero stamps, publish the layout's channel count.
    pub fn initialize(&self) {
        self.request_seq().store(0, Ordering::Relaxed);
        self.response_seq().store(0, Ordering::Relaxed);
        self.frame_count().store(0, Ordering::Relaxed);
        self.channels()
            .store(self.layout.channels, Ordering::Relaxed);
        self.atomic_at(16).store(0, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_sizes_and_offsets_are_consistent() {
        let layout = PluginAudioBlockLayout {
            max_frames: 4096,
            channels: 2,
        };
        assert_eq!(layout.input_offset(), 64);
        assert_eq!(layout.output_offset(), 64 + 4096 * 2 * 4);
        assert_eq!(layout.region_bytes(), 64 + 2 * 4096 * 2 * 4);
    }

    #[test]
    fn view_round_trips_header_and_samples() {
        let layout = PluginAudioBlockLayout {
            max_frames: 8,
            channels: 2,
        };
        let mut backing = vec![0u8; layout.region_bytes() as usize];
        let view = unsafe { PluginAudioBlockView::new(backing.as_mut_ptr(), layout) };
        view.initialize();
        assert_eq!(view.channels().load(Ordering::Relaxed), 2);

        let input: Vec<f32> = (0..16).map(|value| value as f32).collect();
        unsafe { view.write_input(&input) };
        let mut echoed = vec![0.0f32; 16];
        unsafe { view.read_input(&mut echoed) };
        assert_eq!(echoed, input);

        let output: Vec<f32> = input.iter().map(|value| value * 0.5).collect();
        unsafe { view.write_output(&output) };
        let mut read = vec![0.0f32; 16];
        unsafe { view.read_output(&mut read) };
        assert_eq!(read, output);

        view.frame_count().store(8, Ordering::Relaxed);
        view.request_seq().store(3, Ordering::Release);
        view.response_seq().store(3, Ordering::Release);
        assert_eq!(view.request_seq().load(Ordering::Acquire), 3);
        assert_eq!(view.response_seq().load(Ordering::Acquire), 3);
    }
}
