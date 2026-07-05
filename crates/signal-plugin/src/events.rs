/// An absolute parameter value set at a specific frame offset within the block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterValueEvent {
    /// Frame offset within the current block at which the change takes effect.
    pub offset_frames: u32,
    /// Identifies the target parameter.
    pub parameter_id: u32,
    /// New value in the \[0.0, 1.0\] normalized range.
    pub normalized_value: f32,
}

/// A transient modulation offset applied on top of the current parameter value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterModulationEvent {
    /// Frame offset within the current block at which modulation is applied.
    pub offset_frames: u32,
    /// Identifies the target parameter.
    pub parameter_id: u32,
    /// Modulation amount in the normalized range; added to the current value.
    pub amount: f32,
}

/// Marks the start or end of a user gesture on a parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterGesturePhase {
    /// The gesture has started (e.g. mouse-down on a knob).
    Begin,
    /// The gesture has ended (e.g. mouse-up).
    End,
}

/// Signals that a user gesture on a parameter has begun or ended.
///
/// Hosts use gesture events to bracket automation recording and to suppress
/// conflicting automation playback during manual control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterGestureEvent {
    /// Frame offset within the current block at which the gesture boundary occurs.
    pub offset_frames: u32,
    /// Identifies the parameter being touched.
    pub parameter_id: u32,
    /// Whether this marks the beginning or end of the gesture.
    pub phase: ParameterGesturePhase,
}

/// A raw three-byte MIDI 1.0 message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiEvent {
    /// Frame offset within the current block at which the message is delivered.
    pub offset_frames: u32,
    /// MIDI status byte (includes channel in the low nibble for channel messages).
    pub status: u8,
    /// First data byte (e.g. note number, controller number).
    pub data1: u8,
    /// Second data byte (e.g. velocity, controller value).
    pub data2: u8,
}

/// Whether a note event represents a note-on or note-off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteEventKind {
    /// Note began sounding.
    NoteOn,
    /// Note was released.
    NoteOff,
}

/// The dimension of a per-note expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteExpressionKind {
    /// Per-note aftertouch / pressure.
    Pressure,
    /// Spectral brightness or sound character (CLAP/MPE "slide").
    Timbre,
    /// Pitch deviation in cents relative to the note's base pitch.
    Tuning,
}

/// A note-on or note-off event targeting a specific note by ID, channel, and key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteEvent {
    /// Frame offset within the current block at which the note event occurs.
    pub offset_frames: u32,
    /// Per-note identifier used to distinguish simultaneous notes on the same key.
    /// Negative values indicate no specific note ID.
    pub note_id: i32,
    /// Index of the MIDI/note port this event targets.
    pub port_index: u16,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// MIDI key number (0–127).
    pub key: u8,
    /// Velocity in the \[0.0, 1.0\] range.
    pub velocity: f32,
    /// Whether this is a note-on or note-off.
    pub kind: NoteEventKind,
}

/// A per-note expression update targeting a note by ID, channel, and key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteExpressionEvent {
    /// Frame offset within the current block at which the expression change occurs.
    pub offset_frames: u32,
    /// Per-note identifier; negative values match any note on the given channel/key.
    pub note_id: i32,
    /// Index of the MIDI/note port this event targets.
    pub port_index: u16,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// MIDI key number (0–127).
    pub key: u8,
    /// Which expression dimension this event carries.
    pub expression: NoteExpressionKind,
    /// Expression value in the \[0.0, 1.0\] range (interpretation depends on `expression`).
    pub value: f32,
}

/// A continuous-controller change carrying the model's 32-bit normalized
/// value (g12.034 follow-up). Unlike [`MidiEvent`], the value is NOT
/// quantized to 7 bits: each hosting backend downconverts (or maps to a
/// parameter, e.g. VST3 `IMidiMapping`) at its own format boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlChangeEvent {
    /// Frame offset within the current block at which the change applies.
    pub offset_frames: u32,
    /// Index of the MIDI/note port this event targets.
    pub port_index: u16,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// MIDI controller number (0–127).
    pub controller: u8,
    /// Controller value in the \[0.0, 1.0\] normalized range.
    pub value: f32,
}

/// All event types that can appear in a plugin event packet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PluginEvent {
    /// Absolute parameter value change.
    ParameterValue(ParameterValueEvent),
    /// Transient modulation offset for a parameter.
    ParameterModulation(ParameterModulationEvent),
    /// Start or end of a user gesture on a parameter.
    ParameterGesture(ParameterGestureEvent),
    /// Note-on or note-off.
    Note(NoteEvent),
    /// Per-note expression (pressure, timbre, tuning).
    NoteExpression(NoteExpressionEvent),
    /// Raw MIDI 1.0 message.
    Midi(MidiEvent),
    /// Continuous-controller change with a 32-bit normalized value
    /// (downconverted per format at the hosting boundary).
    ControlChange(ControlChangeEvent),
}

impl PluginEvent {
    pub(crate) const ENCODED_BYTES: usize = 24;

    /// Returns `true` if this event is a [`ParameterValue`](Self::ParameterValue) variant.
    pub fn is_parameter_value(self) -> bool {
        matches!(self, Self::ParameterValue(_))
    }

    /// Returns `true` if this event is a [`ParameterModulation`](Self::ParameterModulation) variant.
    pub fn is_parameter_modulation(self) -> bool {
        matches!(self, Self::ParameterModulation(_))
    }

    /// Returns `true` if this event is a [`ParameterGesture`](Self::ParameterGesture) variant.
    pub fn is_parameter_gesture(self) -> bool {
        matches!(self, Self::ParameterGesture(_))
    }

    /// Returns `true` if this event is a [`Note`](Self::Note) variant.
    pub fn is_note(self) -> bool {
        matches!(self, Self::Note(_))
    }

    /// Returns `true` if this event is a [`NoteExpression`](Self::NoteExpression) variant.
    pub fn is_note_expression(self) -> bool {
        matches!(self, Self::NoteExpression(_))
    }

    /// Returns `true` if this event is a [`Midi`](Self::Midi) variant.
    pub fn is_midi(self) -> bool {
        matches!(self, Self::Midi(_))
    }

    /// Returns `true` if this event is a [`ControlChange`](Self::ControlChange) variant.
    pub fn is_control_change(self) -> bool {
        matches!(self, Self::ControlChange(_))
    }
}
