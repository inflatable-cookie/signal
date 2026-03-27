#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterValueEvent {
    pub offset_frames: u32,
    pub parameter_id: u32,
    pub normalized_value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterModulationEvent {
    pub offset_frames: u32,
    pub parameter_id: u32,
    pub amount: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterGesturePhase {
    Begin,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterGestureEvent {
    pub offset_frames: u32,
    pub parameter_id: u32,
    pub phase: ParameterGesturePhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiEvent {
    pub offset_frames: u32,
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteEventKind {
    NoteOn,
    NoteOff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteExpressionKind {
    Pressure,
    Timbre,
    Tuning,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteEvent {
    pub offset_frames: u32,
    pub note_id: i32,
    pub port_index: u16,
    pub channel: u8,
    pub key: u8,
    pub velocity: f32,
    pub kind: NoteEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteExpressionEvent {
    pub offset_frames: u32,
    pub note_id: i32,
    pub port_index: u16,
    pub channel: u8,
    pub key: u8,
    pub expression: NoteExpressionKind,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PluginEvent {
    ParameterValue(ParameterValueEvent),
    ParameterModulation(ParameterModulationEvent),
    ParameterGesture(ParameterGestureEvent),
    Note(NoteEvent),
    NoteExpression(NoteExpressionEvent),
    Midi(MidiEvent),
}

impl PluginEvent {
    pub(crate) const ENCODED_BYTES: usize = 24;

    pub fn is_parameter_value(self) -> bool {
        matches!(self, Self::ParameterValue(_))
    }

    pub fn is_parameter_modulation(self) -> bool {
        matches!(self, Self::ParameterModulation(_))
    }

    pub fn is_parameter_gesture(self) -> bool {
        matches!(self, Self::ParameterGesture(_))
    }

    pub fn is_note(self) -> bool {
        matches!(self, Self::Note(_))
    }

    pub fn is_note_expression(self) -> bool {
        matches!(self, Self::NoteExpression(_))
    }

    pub fn is_midi(self) -> bool {
        matches!(self, Self::Midi(_))
    }
}
