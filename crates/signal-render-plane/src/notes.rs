//! Built-in note sources (stateless instrument clips).

use std::sync::Arc;

// ── Note sources (built-in instrument, stateless) ───────────────────────────

/// Attack ramp length for note voices, in seconds (linear 0 → 1).
pub(crate) const NOTE_ATTACK_SECONDS: f64 = 0.003;
/// Release tail length after a note's end, in seconds (linear 1 → 0).
pub(crate) const NOTE_RELEASE_SECONDS: f64 = 0.040;
/// Most notes rendered simultaneously per block per clip. The overlap scan
/// walks notes in start order, so when more than this many notes sound in
/// one block the EARLIEST-STARTED ones render and the rest are skipped —
/// deterministic, and counted per block (a block-granular approximation of
/// true simultaneity).
pub const NOTE_POLYPHONY_LIMIT: usize = 32;

/// Explicit per-note pitch override (loophole g12.034 rider): what
/// frequency actually sounds, kept separate from the note's degree (row)
/// identity. `None` on the note derives the frequency from the degree via
/// the tuning default (12-EDO today).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderPitchIntent {
    /// Absolute frequency in hertz.
    FrequencyHz(f64),
    /// Offset in cents from the degree's tuning-derived frequency.
    CentsOffset(f64),
}

/// One note event: clip-relative timing on the stream clock, degree (row)
/// identity + optional pitch intent, normalized velocity. Velocity is the
/// voice's sustain amplitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderNote {
    /// First frame of the note, relative to the owning clip's window start.
    pub start_frame: u64,
    /// Note length in frames (the release tail extends past this).
    pub duration_frames: u64,
    /// Degree (row/scale-step) identity. Under the 12-EDO default tuning a
    /// degree IS the MIDI pitch (69 = A4 = 440 Hz).
    pub degree: i32,
    /// `None` = derive the frequency from `degree` via the 12-EDO default;
    /// `Some` = explicit override.
    pub pitch_intent: Option<RenderPitchIntent>,
    /// Normalized velocity in `0..=1`, applied as the voice amplitude.
    pub velocity: f32,
}

impl RenderNote {
    /// The 12-EDO default frequency for a degree:
    /// `440 · 2^((degree − 69) / 12)`.
    fn degree_default_frequency_hz(degree: i32) -> f64 {
        440.0 * f64::powf(2.0, (f64::from(degree) - 69.0) / 12.0)
    }

    /// Oscillator frequency in hertz, derived from `(degree, pitch_intent)`:
    /// no intent means the degree's 12-EDO default — bit-identical to the
    /// pre-widening `440 · 2^((pitch − 69) / 12)` path.
    pub fn frequency_hz(&self) -> f64 {
        match self.pitch_intent {
            None => Self::degree_default_frequency_hz(self.degree),
            Some(RenderPitchIntent::FrequencyHz(hz)) => hz,
            Some(RenderPitchIntent::CentsOffset(cents)) => {
                Self::degree_default_frequency_hz(self.degree) * f64::powf(2.0, cents / 1200.0)
            }
        }
    }
}

/// Shared immutable note list for one clip, sorted by `start_frame`
/// (compile validates and rejects unsorted buffers).
///
/// Equality is pointer-based (like [`RenderSampleBuffer`]): hosts cache one
/// buffer per clip content so recompiled specs stay idempotent.
#[derive(Clone, Debug)]
pub struct RenderNoteBuffer {
    /// Notes sorted by `start_frame`.
    pub notes: Arc<[RenderNote]>,
}

impl PartialEq for RenderNoteBuffer {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.notes, &other.notes)
    }
}
