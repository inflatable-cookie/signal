//! Disk-streaming clip sources (chunk mailbox).

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

// ── Disk-streaming clip sources (chunk mailbox) ─────────────────────────────
//
// Long media plays through [`RenderSource::Stream`]: a control-side feeder
// decodes windows of the file into [`StreamChunk`]s and posts them through a
// bounded lock-free mailbox; the executor drains them into a small fixed
// array of held chunks and renders from those with the SAME interpolation
// paths as in-memory `Samples`. Chunks the executor no longer needs travel
// BACK through a retired mailbox and are deallocated control-side — exactly
// the plan-swap ownership discipline, applied per chunk. The executor never
// blocks, never allocates, and never frees: a missing source frame renders
// silence and increments an underrun counter.

/// Capacity of the feeder → executor chunk mailbox per stream handle.
const STREAM_CHUNK_MAILBOX_CAPACITY: usize = 8;
/// Chunks the executor holds per streaming clip while rendering.
pub(crate) const STREAM_HELD_CHUNK_SLOTS: usize = 4;
/// Retired-chunk mailbox capacity: sized so every in-flight chunk (mailbox
/// plus held slots) can retire without saturating while the feeder drains.
const STREAM_RETIRED_MAILBOX_CAPACITY: usize =
    STREAM_CHUNK_MAILBOX_CAPACITY + STREAM_HELD_CHUNK_SLOTS + 2;
/// Held or mailbox chunks starting further than this past the frame the
/// executor currently needs are treated as stale (left over from before a
/// backward seek) and retired. Feeders must keep their read-ahead window
/// comfortably inside this bound or their prefetch gets churned.
pub const STREAM_RETIRE_LOOKAHEAD_FRAMES: u64 = 262_144;

/// One window of decoded media: interleaved stereo f32 at the stream
/// handle's source rate, anchored at `start_frame` on the source clock.
#[derive(Clone, Debug)]
pub struct StreamChunk {
    /// First source frame the chunk covers.
    pub start_frame: u64,
    /// Interleaved stereo frames (length is even; frame count = len / 2).
    pub frames: Arc<[f32]>,
}

impl StreamChunk {
    /// Number of stereo frames in the chunk.
    pub fn frame_count(&self) -> u64 {
        self.frames.len() as u64 / 2
    }

    /// One past the last source frame the chunk covers.
    pub(crate) fn end_frame(&self) -> u64 {
        self.start_frame + self.frame_count()
    }
}

/// Bounded lock-free MPMC queue (Vyukov sequence-stamped ring). Used as the
/// chunk and retired mailboxes inside a stream handle: `try_push`/`try_pop`
/// neither allocate nor free nor block, so both ends are audio-thread safe.
/// (mpsc channels cannot serve here — the handle lives inside clonable plan
/// specs, and `Receiver` is neither clonable nor `Sync`.)
pub(crate) struct ChunkQueueSlot<T> {
    sequence: AtomicUsize,
    value: UnsafeCell<MaybeUninit<T>>,
}

pub(crate) struct ChunkQueue<T> {
    slots: Box<[ChunkQueueSlot<T>]>,
    enqueue_position: AtomicUsize,
    dequeue_position: AtomicUsize,
}

// Safety: access to each slot's value is serialized by its sequence stamp
// (acquire/release): a slot is written only after winning the enqueue CAS
// and read only after winning the dequeue CAS.
unsafe impl<T: Send> Send for ChunkQueue<T> {}
unsafe impl<T: Send> Sync for ChunkQueue<T> {}

impl<T> ChunkQueue<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        let slots = (0..capacity)
            .map(|index| ChunkQueueSlot {
                sequence: AtomicUsize::new(index),
                value: UnsafeCell::new(MaybeUninit::uninit()),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        ChunkQueue {
            slots,
            enqueue_position: AtomicUsize::new(0),
            dequeue_position: AtomicUsize::new(0),
        }
    }

    /// Push without blocking or allocating; returns the value when full.
    pub(crate) fn try_push(&self, value: T) -> Result<(), T> {
        let capacity = self.slots.len();
        let mut position = self.enqueue_position.load(Ordering::Relaxed);
        loop {
            let slot = &self.slots[position % capacity];
            let sequence = slot.sequence.load(Ordering::Acquire);
            if sequence == position {
                match self.enqueue_position.compare_exchange_weak(
                    position,
                    position + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        unsafe { (*slot.value.get()).write(value) };
                        slot.sequence.store(position + 1, Ordering::Release);
                        return Ok(());
                    }
                    Err(current) => position = current,
                }
            } else if sequence < position {
                return Err(value); // Full.
            } else {
                position = self.enqueue_position.load(Ordering::Relaxed);
            }
        }
    }

    /// Pop without blocking or allocating; `None` when empty.
    pub(crate) fn try_pop(&self) -> Option<T> {
        let capacity = self.slots.len();
        let mut position = self.dequeue_position.load(Ordering::Relaxed);
        loop {
            let slot = &self.slots[position % capacity];
            let sequence = slot.sequence.load(Ordering::Acquire);
            if sequence == position + 1 {
                match self.dequeue_position.compare_exchange_weak(
                    position,
                    position + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        let value = unsafe { (*slot.value.get()).assume_init_read() };
                        slot.sequence.store(position + capacity, Ordering::Release);
                        return Some(value);
                    }
                    Err(current) => position = current,
                }
            } else if sequence <= position {
                return None; // Empty.
            } else {
                position = self.dequeue_position.load(Ordering::Relaxed);
            }
        }
    }
}

impl<T> Drop for ChunkQueue<T> {
    fn drop(&mut self) {
        while self.try_pop().is_some() {}
    }
}

/// State shared between a stream handle (executor side) and its feeder.
pub(crate) struct StreamInner {
    pub(crate) source_sample_rate_hz: u32,
    pub(crate) total_frames: u64,
    /// Next source frame the executor needs (its read hint), published per
    /// rendered block. Seeks read as jumps here.
    pub(crate) wanted_frame: AtomicU64,
    /// Output frames rendered as silence because the needed source frame was
    /// not held (feeder behind, or seek not yet served).
    pub(crate) underrun_frames: AtomicU64,
    /// Feeder → executor chunk mailbox.
    pub(crate) chunks: ChunkQueue<StreamChunk>,
    /// Executor → feeder retired-chunk mailbox: chunks the executor no
    /// longer needs, returned for control-side deallocation.
    pub(crate) retired: ChunkQueue<StreamChunk>,
}

/// Executor-side handle to a disk-streamed media source. Arc-shared and
/// pointer-equal (like `RenderSampleBuffer`): create one per streaming
/// asset and reuse it across plan recompiles so specs stay idempotent.
/// Clips reference the handle plus their own window/anchor, exactly as
/// in-memory sample clips reference a shared buffer.
#[derive(Clone)]
pub struct RenderStreamHandle {
    pub(crate) inner: Arc<StreamInner>,
}

impl std::fmt::Debug for RenderStreamHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderStreamHandle")
            .field("source_sample_rate_hz", &self.inner.source_sample_rate_hz)
            .field("total_frames", &self.inner.total_frames)
            .finish_non_exhaustive()
    }
}

impl PartialEq for RenderStreamHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl RenderStreamHandle {
    /// Source sample rate of the streamed media.
    pub fn source_sample_rate_hz(&self) -> u32 {
        self.inner.source_sample_rate_hz
    }

    /// Total source frames in the streamed media.
    pub fn total_frames(&self) -> u64 {
        self.inner.total_frames
    }

    /// Output frames rendered as silence because the needed source frame
    /// was not available (cumulative).
    pub fn underrun_frames(&self) -> u64 {
        self.inner.underrun_frames.load(Ordering::Relaxed)
    }
}

/// Control-side feeder for one stream handle: reads the executor's wanted
/// frame, posts decoded chunks, and reclaims retired chunks for
/// deallocation. Single producer by design — do not share one feeder across
/// threads (the queue tolerates it, but interleaved feeding is meaningless).
pub struct StreamFeeder {
    pub(crate) inner: Arc<StreamInner>,
}

impl StreamFeeder {
    /// Next source frame the executor needs, as last published. Feed chunks
    /// covering `[wanted, wanted + read-ahead)`.
    pub fn wanted_frame(&self) -> u64 {
        self.inner.wanted_frame.load(Ordering::Relaxed)
    }

    /// Cumulative output frames the executor rendered as silence for want
    /// of this stream's data.
    pub fn underrun_frames(&self) -> u64 {
        self.inner.underrun_frames.load(Ordering::Relaxed)
    }

    /// Post a chunk to the executor; returns the chunk when the mailbox is
    /// full (try again after the executor drains).
    pub fn try_send_chunk(&self, chunk: StreamChunk) -> Result<(), StreamChunk> {
        self.inner.chunks.try_push(chunk)
    }

    /// Reclaim chunks the executor has retired; dropping the returned `Vec`
    /// deallocates them here, on the control side.
    pub fn collect_retired(&self) -> Vec<StreamChunk> {
        let mut retired = Vec::new();
        while let Some(chunk) = self.inner.retired.try_pop() {
            retired.push(chunk);
        }
        retired
    }
}

/// Create a connected feeder/handle pair for one streaming asset:
/// interleaved stereo media at `source_sample_rate_hz`, `total_frames`
/// frames long. The handle goes into `RenderSource::Stream` specs; the
/// feeder stays control-side and must be pumped (post chunks toward
/// [`StreamFeeder::wanted_frame`], collect retired) for audio to flow.
pub fn render_stream(
    source_sample_rate_hz: u32,
    total_frames: u64,
) -> (StreamFeeder, RenderStreamHandle) {
    let inner = Arc::new(StreamInner {
        source_sample_rate_hz,
        total_frames,
        wanted_frame: AtomicU64::new(0),
        underrun_frames: AtomicU64::new(0),
        chunks: ChunkQueue::new(STREAM_CHUNK_MAILBOX_CAPACITY),
        retired: ChunkQueue::new(STREAM_RETIRED_MAILBOX_CAPACITY),
    });
    (
        StreamFeeder {
            inner: Arc::clone(&inner),
        },
        RenderStreamHandle { inner },
    )
}
