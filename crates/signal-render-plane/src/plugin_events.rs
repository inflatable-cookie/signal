//! Plugin event delivery vocabulary and state-chase helpers.

use std::sync::Arc;

// ── Plugin event delivery (g12.034 follow-up) ───────────────────────────────
//
// Stages carrying a plugin processor may also carry a compiled event stream
// (notes + MIDI performance data from the consumer's model). The executor
// slices the
// stream per block and hands the slice to the processor with intra-block
// sample offsets; the processing backend converts to each plugin format's
// native event lists AT THAT BOUNDARY (the model value stays 32-bit float
// 0..1 here — no 7-bit MIDI quantization inside the engine).

/// What one compiled plugin event does. Values stay normalized floats;
/// downconversion to MIDI 1.0 bytes (where a format needs it) happens in
/// the plugin backend, never here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderPluginEventKind {
    /// Note starts sounding.
    NoteOn {
        /// MIDI key number (0–127).
        key: u8,
        /// Velocity in `0..=1`.
        velocity: f32,
    },
    /// Note released.
    NoteOff {
        /// MIDI key number (0–127).
        key: u8,
    },
    /// Continuous-controller change.
    ControlChange {
        /// MIDI controller number (0–127).
        controller: u8,
        /// Controller value in `0..=1` (32-bit; quantized only at the
        /// plugin-format boundary).
        value: f32,
    },
    /// Channel pitch bend, before any instrument-specific bend range.
    PitchBend {
        /// Bipolar wheel position in `-1..=1`; zero is centre.
        value: f32,
    },
    /// Channel pressure (channel aftertouch).
    ChannelPressure {
        /// Pressure in `0..=1`.
        value: f32,
    },
    /// Per-note expression targeting the active note on `channel`/`key`.
    NoteExpression {
        /// MIDI key number (0-127).
        key: u8,
        /// Expression dimension.
        expression: RenderNoteExpressionKind,
        /// Pressure/timbre use `0..=1`; tuning uses cents.
        value: f32,
    },
    /// Start a voice-bank slot playing a preloaded sound (voice-bank
    /// processors only; plugin-format bridges cannot represent this and drop
    /// it). Slot allocation/stealing policy belongs to the sender.
    VoiceStart {
        /// Bank slot to (re)start.
        voice: u16,
        /// Index into the bank's preloaded sound table.
        sound: u16,
        /// Linear gain applied to the voice.
        gain: f32,
    },
    /// Stop a voice-bank slot immediately.
    VoiceStop {
        /// Bank slot to silence.
        voice: u16,
    },
    /// Update one continuous parameter on a voice-bank slot (e.g. select a
    /// new HRIR as a source moves, or retarget gain).
    VoiceParam {
        /// Bank slot addressed.
        voice: u16,
        /// Which parameter `value` carries.
        param: RenderVoiceParam,
        /// Parameter payload (see [`RenderVoiceParam`] for units).
        value: f32,
    },
}

/// Continuous per-voice parameters for voice-bank processors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderVoiceParam {
    /// Index into the bank's HRIR table (value is a non-negative integer
    /// carried in f32; exact for any real table size). The bank crossfades
    /// to the newly selected response.
    HrirIndex,
    /// Linear voice gain (`0..` — typically `0..=1`).
    Gain,
    /// Occlusion low-pass cutoff in Hz. Values at/above ~20 kHz disable the
    /// filter (fully unoccluded).
    OcclusionCutoffHz,
}

/// Format-neutral per-note expression dimensions supported by the render plane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderNoteExpressionKind {
    /// Per-note pressure / polyphonic aftertouch.
    Pressure,
    /// Per-note spectral brightness or sound character.
    Timbre,
    /// Per-note pitch deviation in cents.
    Tuning,
}

/// Event families one live plugin processor backend can deliver to its
/// native format. This describes host transport support, not the plugin's
/// catalog-declared input capability.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderPluginEventSupport {
    /// Note-on and note-off delivery.
    pub notes: bool,
    /// Continuous-controller delivery.
    pub control_change: bool,
    /// Channel pitch-bend delivery.
    pub pitch_bend: bool,
    /// Channel-pressure delivery.
    pub channel_pressure: bool,
    /// Per-note pressure, timbre, and tuning delivery.
    pub note_expression: bool,
}

/// One compiled plugin event on the ABSOLUTE stream clock (the consumer
/// projects ticks to samples at compile; clip windows are already applied).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPluginEvent {
    /// Absolute stream-clock frame the event fires at.
    pub frame: u64,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// What the event does.
    pub kind: RenderPluginEventKind,
}

/// Shared immutable event stream for one processor stage, sorted by `frame`
/// (compile validates and rejects unsorted buffers).
///
/// Equality is pointer-based (like `RenderNoteBuffer`): hosts cache one
/// buffer per content hash so recompiled specs stay idempotent.
#[derive(Clone, Debug)]
pub struct RenderPluginEventBuffer {
    /// Events sorted by `frame`.
    pub events: Arc<[RenderPluginEvent]>,
}

impl PartialEq for RenderPluginEventBuffer {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.events, &other.events)
    }
}

/// One plugin event scheduled within the CURRENT block: the absolute frame
/// resolved to an offset from the block (or loop-segment) start. This is
/// the slice element handed to `PluginBlockProcessor::process_with_events`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBlockPluginEvent {
    /// Frame offset within the current block at which the event fires.
    pub offset_frames: u32,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// What the event does.
    pub kind: RenderPluginEventKind,
}

/// Per-stage, per-block plugin event capacity. The executor's event scratch
/// is preallocated at this size (audio thread never allocates); events past
/// the cap in one block are dropped, earliest-first wins.
pub const PLUGIN_EVENTS_PER_BLOCK_CAPACITY: usize = 1024;

/// Inline event capacity of one `RenderCommand`-borne live-event push.
/// The command must stay `Send` + fixed-size (no allocation once the
/// controller has copied the caller's slice), so batches larger than this
/// chunk across multiple commands control-side.
pub const LIVE_EVENT_PUSH_CAPACITY: usize = 32;

/// Per-stage live-event ring capacity (g13.018). The ring is preallocated
/// at install for stages with `RenderStageSpec::accepts_live_events` and
/// drains fully every rendered block; events pushed past the capacity
/// between blocks are dropped and counted
/// (`RenderPlaneController::live_event_drop_count`).
pub const LIVE_EVENT_RING_CAPACITY: usize = 256;

/// Reconcile plugin note, controller, and expression state across a non-
/// contiguous transport jump. Fixed bounded state and the caller's
/// preallocated scratch keep this audio-thread safe. Note IDs are not yet
/// distinct, so note expression is tracked by channel/key.
pub(crate) fn append_plugin_state_chase(
    events: &[RenderPluginEvent],
    from_frame: u64,
    to_frame: u64,
    offset_frames: u32,
    scratch: &mut Vec<RenderBlockPluginEvent>,
) {
    let mut from_active = [0u128; 16];
    let mut to_active = [0u128; 16];
    let mut to_velocity = [[0.0f32; 128]; 16];
    let mut cc_seen = [0u128; 16];
    let mut cc_values = [[0.0f32; 128]; 16];
    let mut bend_seen = 0u16;
    let mut bend_values = [0.0f32; 16];
    let mut pressure_seen = 0u16;
    let mut pressure_values = [0.0f32; 16];
    let mut expression_seen = [[0u128; 16]; 3];
    let mut expression_values = [[[0.0f32; 128]; 16]; 3];
    let scan_end = from_frame.max(to_frame);
    for event in events.iter().take_while(|event| event.frame < scan_end) {
        let channel = usize::from(event.channel.min(15));
        match event.kind {
            RenderPluginEventKind::NoteOn { key, velocity } => {
                let bit = 1u128 << key;
                if event.frame < from_frame {
                    from_active[channel] |= bit;
                }
                if event.frame < to_frame {
                    to_active[channel] |= bit;
                    to_velocity[channel][usize::from(key)] = velocity;
                }
            }
            RenderPluginEventKind::NoteOff { key } => {
                let bit = 1u128 << key;
                if event.frame < from_frame {
                    from_active[channel] &= !bit;
                }
                if event.frame < to_frame {
                    to_active[channel] &= !bit;
                }
            }
            RenderPluginEventKind::ControlChange { controller, value }
                if event.frame < to_frame =>
            {
                cc_seen[channel] |= 1u128 << controller;
                cc_values[channel][usize::from(controller)] = value;
            }
            RenderPluginEventKind::PitchBend { value } if event.frame < to_frame => {
                bend_seen |= 1u16 << channel;
                bend_values[channel] = value;
            }
            RenderPluginEventKind::ChannelPressure { value } if event.frame < to_frame => {
                pressure_seen |= 1u16 << channel;
                pressure_values[channel] = value;
            }
            RenderPluginEventKind::NoteExpression {
                key,
                expression,
                value,
            } if event.frame < to_frame => {
                let index = match expression {
                    RenderNoteExpressionKind::Pressure => 0,
                    RenderNoteExpressionKind::Timbre => 1,
                    RenderNoteExpressionKind::Tuning => 2,
                };
                expression_seen[index][channel] |= 1u128 << key;
                expression_values[index][channel][usize::from(key)] = value;
            }
            _ => {}
        }
    }
    for (channel, active) in from_active.into_iter().enumerate() {
        for key in 0..128u8 {
            if active & (1u128 << key) == 0 {
                continue;
            }
            if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                return;
            }
            scratch.push(RenderBlockPluginEvent {
                offset_frames,
                channel: channel as u8,
                kind: RenderPluginEventKind::NoteOff { key },
            });
        }
    }
    // Channel-wide state must precede destination attacks.
    for channel in 0..16 {
        for controller in 0..128u8 {
            if cc_seen[channel] & (1u128 << controller) == 0 {
                continue;
            }
            if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                return;
            }
            scratch.push(RenderBlockPluginEvent {
                offset_frames,
                channel: channel as u8,
                kind: RenderPluginEventKind::ControlChange {
                    controller,
                    value: cc_values[channel][usize::from(controller)],
                },
            });
        }
        for (seen, kind) in [
            (
                bend_seen,
                RenderPluginEventKind::PitchBend {
                    value: bend_values[channel],
                },
            ),
            (
                pressure_seen,
                RenderPluginEventKind::ChannelPressure {
                    value: pressure_values[channel],
                },
            ),
        ] {
            if seen & (1u16 << channel) == 0 {
                continue;
            }
            if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                return;
            }
            scratch.push(RenderBlockPluginEvent {
                offset_frames,
                channel: channel as u8,
                kind,
            });
        }
    }
    for (channel, active) in to_active.into_iter().enumerate() {
        for key in 0..128u8 {
            if active & (1u128 << key) == 0 {
                continue;
            }
            if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                return;
            }
            scratch.push(RenderBlockPluginEvent {
                offset_frames,
                channel: channel as u8,
                kind: RenderPluginEventKind::NoteOn {
                    key,
                    velocity: to_velocity[channel][usize::from(key)],
                },
            });
        }
    }
    // Per-note state follows attacks so formats that require an active note
    // can address the freshly chased voice.
    for (index, expression) in [
        RenderNoteExpressionKind::Pressure,
        RenderNoteExpressionKind::Timbre,
        RenderNoteExpressionKind::Tuning,
    ]
    .into_iter()
    .enumerate()
    {
        for channel in 0..16 {
            let active = to_active[channel] & expression_seen[index][channel];
            for key in 0..128u8 {
                if active & (1u128 << key) == 0 {
                    continue;
                }
                if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                    return;
                }
                scratch.push(RenderBlockPluginEvent {
                    offset_frames,
                    channel: channel as u8,
                    kind: RenderPluginEventKind::NoteExpression {
                        key,
                        expression,
                        value: expression_values[index][channel][usize::from(key)],
                    },
                });
            }
        }
    }
}
