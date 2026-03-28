#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClapParamValueEvent {
    pub offset_frames: u32,
    pub clap_param_id: u32,
    pub normalized_value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClapParamModEvent {
    pub offset_frames: u32,
    pub clap_param_id: u32,
    pub amount: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapParamGesturePhase {
    Begin,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClapParamGestureEvent {
    pub offset_frames: u32,
    pub clap_param_id: u32,
    pub phase: ClapParamGesturePhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapNoteEventKind {
    NoteOn,
    NoteOff,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClapNoteEvent {
    pub offset_frames: u32,
    pub note_id: i32,
    pub port_index: u16,
    pub channel: u8,
    pub key: u8,
    pub velocity: f64,
    pub kind: ClapNoteEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapNoteExpressionKind {
    Pressure,
    Timbre,
    Tuning,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClapNoteExpressionEvent {
    pub offset_frames: u32,
    pub note_id: i32,
    pub port_index: u16,
    pub channel: u8,
    pub key: u8,
    pub expression: ClapNoteExpressionKind,
    pub value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClapMidiEvent {
    pub offset_frames: u32,
    pub port_index: u16,
    pub data: [u8; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClapEvent {
    ParamValue(ClapParamValueEvent),
    ParamModulation(ClapParamModEvent),
    ParamGesture(ClapParamGestureEvent),
    Note(ClapNoteEvent),
    NoteExpression(ClapNoteExpressionEvent),
    Midi(ClapMidiEvent),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClapEventPacket {
    pub events: Vec<ClapEvent>,
}
