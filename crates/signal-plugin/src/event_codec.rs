use crate::{
    MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent, NoteExpressionKind,
    ParameterGestureEvent, ParameterGesturePhase, ParameterModulationEvent, ParameterValueEvent,
    PluginEvent,
};

const ENCODED_BYTES: usize = 24;

/// Encodes `event` into the first 24 bytes of `bytes` in the shared-memory wire format.
///
/// Returns an error if `bytes` is shorter than `PluginEvent::ENCODED_BYTES`.
pub fn write_event_to_slice(event: &PluginEvent, bytes: &mut [u8]) -> Result<(), &'static str> {
    if bytes.len() < PluginEvent::ENCODED_BYTES {
        return Err("event region entry is too small for encoded event");
    }

    bytes[..ENCODED_BYTES].fill(0);
    match event {
        PluginEvent::ParameterValue(event) => {
            bytes[0] = 1;
            bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
            bytes[8..12].copy_from_slice(&event.parameter_id.to_le_bytes());
            bytes[12..16].copy_from_slice(&event.normalized_value.to_le_bytes());
        }
        PluginEvent::ParameterModulation(event) => {
            bytes[0] = 2;
            bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
            bytes[8..12].copy_from_slice(&event.parameter_id.to_le_bytes());
            bytes[12..16].copy_from_slice(&event.amount.to_le_bytes());
        }
        PluginEvent::ParameterGesture(event) => {
            bytes[0] = 3;
            bytes[1] = match event.phase {
                ParameterGesturePhase::Begin => 1,
                ParameterGesturePhase::End => 2,
            };
            bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
            bytes[8..12].copy_from_slice(&event.parameter_id.to_le_bytes());
        }
        PluginEvent::Note(event) => {
            bytes[0] = 4;
            bytes[1] = match event.kind {
                NoteEventKind::NoteOn => 1,
                NoteEventKind::NoteOff => 2,
            };
            bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
            bytes[8..12].copy_from_slice(&event.note_id.to_le_bytes());
            bytes[12..14].copy_from_slice(&event.port_index.to_le_bytes());
            bytes[14] = event.channel;
            bytes[15] = event.key;
            bytes[16..20].copy_from_slice(&event.velocity.to_le_bytes());
        }
        PluginEvent::NoteExpression(event) => {
            bytes[0] = 5;
            bytes[1] = match event.expression {
                NoteExpressionKind::Pressure => 1,
                NoteExpressionKind::Timbre => 2,
                NoteExpressionKind::Tuning => 3,
            };
            bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
            bytes[8..12].copy_from_slice(&event.note_id.to_le_bytes());
            bytes[12..14].copy_from_slice(&event.port_index.to_le_bytes());
            bytes[14] = event.channel;
            bytes[15] = event.key;
            bytes[16..20].copy_from_slice(&event.value.to_le_bytes());
        }
        PluginEvent::Midi(event) => {
            bytes[0] = 6;
            bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
            bytes[8] = event.status;
            bytes[9] = event.data1;
            bytes[10] = event.data2;
        }
    }
    Ok(())
}

/// Decodes a [`PluginEvent`] from the first 24 bytes of `bytes`.
///
/// Returns an error if `bytes` is too short or contains an unrecognised type tag.
pub fn read_event_from_slice(bytes: &[u8]) -> Result<PluginEvent, &'static str> {
    if bytes.len() < PluginEvent::ENCODED_BYTES {
        return Err("event region entry is too small for encoded event");
    }

    match bytes[0] {
        1 => Ok(PluginEvent::ParameterValue(ParameterValueEvent {
            offset_frames: u32::from_le_bytes(
                bytes[4..8]
                    .try_into()
                    .map_err(|_| "parameter event offset decode")?,
            ),
            parameter_id: u32::from_le_bytes(
                bytes[8..12]
                    .try_into()
                    .map_err(|_| "parameter event id decode")?,
            ),
            normalized_value: f32::from_le_bytes(
                bytes[12..16]
                    .try_into()
                    .map_err(|_| "parameter event value decode")?,
            ),
        })),
        2 => Ok(PluginEvent::ParameterModulation(ParameterModulationEvent {
            offset_frames: u32::from_le_bytes(
                bytes[4..8]
                    .try_into()
                    .map_err(|_| "parameter modulation offset decode")?,
            ),
            parameter_id: u32::from_le_bytes(
                bytes[8..12]
                    .try_into()
                    .map_err(|_| "parameter modulation id decode")?,
            ),
            amount: f32::from_le_bytes(
                bytes[12..16]
                    .try_into()
                    .map_err(|_| "parameter modulation amount decode")?,
            ),
        })),
        3 => Ok(PluginEvent::ParameterGesture(ParameterGestureEvent {
            offset_frames: u32::from_le_bytes(
                bytes[4..8]
                    .try_into()
                    .map_err(|_| "parameter gesture offset decode")?,
            ),
            parameter_id: u32::from_le_bytes(
                bytes[8..12]
                    .try_into()
                    .map_err(|_| "parameter gesture id decode")?,
            ),
            phase: match bytes[1] {
                1 => ParameterGesturePhase::Begin,
                2 => ParameterGesturePhase::End,
                _ => return Err("unknown parameter gesture phase"),
            },
        })),
        4 => Ok(PluginEvent::Note(NoteEvent {
            offset_frames: u32::from_le_bytes(
                bytes[4..8]
                    .try_into()
                    .map_err(|_| "note event offset decode")?,
            ),
            note_id: i32::from_le_bytes(
                bytes[8..12]
                    .try_into()
                    .map_err(|_| "note event id decode")?,
            ),
            port_index: u16::from_le_bytes(
                bytes[12..14]
                    .try_into()
                    .map_err(|_| "note event port decode")?,
            ),
            channel: bytes[14],
            key: bytes[15],
            velocity: f32::from_le_bytes(
                bytes[16..20]
                    .try_into()
                    .map_err(|_| "note event velocity decode")?,
            ),
            kind: match bytes[1] {
                1 => NoteEventKind::NoteOn,
                2 => NoteEventKind::NoteOff,
                _ => return Err("unknown note event kind"),
            },
        })),
        5 => Ok(PluginEvent::NoteExpression(NoteExpressionEvent {
            offset_frames: u32::from_le_bytes(
                bytes[4..8]
                    .try_into()
                    .map_err(|_| "note expression offset decode")?,
            ),
            note_id: i32::from_le_bytes(
                bytes[8..12]
                    .try_into()
                    .map_err(|_| "note expression id decode")?,
            ),
            port_index: u16::from_le_bytes(
                bytes[12..14]
                    .try_into()
                    .map_err(|_| "note expression port decode")?,
            ),
            channel: bytes[14],
            key: bytes[15],
            value: f32::from_le_bytes(
                bytes[16..20]
                    .try_into()
                    .map_err(|_| "note expression value decode")?,
            ),
            expression: match bytes[1] {
                1 => NoteExpressionKind::Pressure,
                2 => NoteExpressionKind::Timbre,
                3 => NoteExpressionKind::Tuning,
                _ => return Err("unknown note expression kind"),
            },
        })),
        6 => Ok(PluginEvent::Midi(MidiEvent {
            offset_frames: u32::from_le_bytes(
                bytes[4..8]
                    .try_into()
                    .map_err(|_| "midi event offset decode")?,
            ),
            status: bytes[8],
            data1: bytes[9],
            data2: bytes[10],
        })),
        _ => Err("unknown plugin event type"),
    }
}
