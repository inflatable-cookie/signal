/// A CLAP parameter value change event at a specific sample offset within a block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClapParamValueEvent {
    /// Sample offset within the block at which this event applies.
    pub offset_frames: u32,
    /// The CLAP parameter ID.
    pub clap_param_id: u32,
    /// New normalised value for the parameter.
    pub normalized_value: f64,
}

/// A CLAP parameter modulation event (additive modulation amount, not an absolute value).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClapParamModEvent {
    /// Sample offset within the block at which this event applies.
    pub offset_frames: u32,
    /// The CLAP parameter ID.
    pub clap_param_id: u32,
    /// Additive modulation amount to apply to the parameter.
    pub amount: f64,
}

/// Phase of a CLAP parameter gesture (begin or end of a user interaction).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapParamGesturePhase {
    /// The gesture is starting.
    Begin,
    /// The gesture is ending.
    End,
}

/// A CLAP parameter gesture event signalling the start or end of a user-driven automation gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClapParamGestureEvent {
    /// Sample offset within the block at which this event applies.
    pub offset_frames: u32,
    /// The CLAP parameter ID.
    pub clap_param_id: u32,
    /// Whether the gesture is beginning or ending.
    pub phase: ClapParamGesturePhase,
}

/// Whether a CLAP note event is a note-on or note-off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapNoteEventKind {
    /// Note-on event.
    NoteOn,
    /// Note-off event.
    NoteOff,
}

/// A CLAP note-on or note-off event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClapNoteEvent {
    /// Sample offset within the block at which this event applies.
    pub offset_frames: u32,
    /// CLAP note identifier, or -1 if unset.
    pub note_id: i32,
    /// Index of the note port this event targets.
    pub port_index: u16,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// MIDI key number (0–127).
    pub key: u8,
    /// Note velocity in the range [0.0, 1.0].
    pub velocity: f64,
    /// Whether this is a note-on or note-off.
    pub kind: ClapNoteEventKind,
}

/// The type of per-note expression carried in a [`ClapNoteExpressionEvent`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapNoteExpressionKind {
    /// Per-note pressure (aftertouch).
    Pressure,
    /// Per-note timbre (brightness/filter).
    Timbre,
    /// Per-note tuning offset in semitones.
    Tuning,
}

/// A CLAP per-note expression event (pressure, timbre, or tuning).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClapNoteExpressionEvent {
    /// Sample offset within the block at which this event applies.
    pub offset_frames: u32,
    /// CLAP note identifier, or -1 if unset.
    pub note_id: i32,
    /// Index of the note port this event targets.
    pub port_index: u16,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// MIDI key number (0–127).
    pub key: u8,
    /// The expression type being set.
    pub expression: ClapNoteExpressionKind,
    /// Expression value in the range [0.0, 1.0].
    pub value: f64,
}

/// A raw 3-byte MIDI event routed to a specific CLAP note port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClapMidiEvent {
    /// Sample offset within the block at which this event applies.
    pub offset_frames: u32,
    /// Index of the note port this event targets.
    pub port_index: u16,
    /// Raw MIDI bytes: [status, data1, data2].
    pub data: [u8; 3],
}

/// Any CLAP event that can be sent to or received from a plugin instance in a block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClapEvent {
    /// A parameter value change.
    ParamValue(ClapParamValueEvent),
    /// A parameter modulation.
    ParamModulation(ClapParamModEvent),
    /// A parameter gesture begin/end.
    ParamGesture(ClapParamGestureEvent),
    /// A note-on or note-off.
    Note(ClapNoteEvent),
    /// A per-note expression change.
    NoteExpression(ClapNoteExpressionEvent),
    /// A raw MIDI message.
    Midi(ClapMidiEvent),
}

/// A batch of [`ClapEvent`]s for a single block.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClapEventPacket {
    /// Events in delivery order.
    pub events: Vec<ClapEvent>,
}
