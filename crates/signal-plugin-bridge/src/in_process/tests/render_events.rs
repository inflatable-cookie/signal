//! Render-event conversion boundary tests.

use super::prelude::*;

#[test]
fn expressive_render_events_retain_values_at_the_neutral_boundary() {
    let events = [
        RenderBlockPluginEvent {
            offset_frames: 3,
            channel: 2,
            kind: RenderPluginEventKind::PitchBend { value: 0.0 },
        },
        RenderBlockPluginEvent {
            offset_frames: 5,
            channel: 2,
            kind: RenderPluginEventKind::ChannelPressure { value: 0.5 },
        },
        RenderBlockPluginEvent {
            offset_frames: 7,
            channel: 2,
            kind: RenderPluginEventKind::NoteExpression {
                key: 64,
                expression: RenderNoteExpressionKind::Tuning,
                value: 37.5,
            },
        },
    ];
    let mut scratch = Vec::with_capacity(EVENT_SCRATCH_CAPACITY);
    convert_block_events(&events, &mut scratch);

    assert_eq!(
        scratch[0],
        PluginEvent::Midi(MidiEvent {
            offset_frames: 3,
            status: 0xE2,
            data1: 0,
            data2: 64,
        })
    );
    assert_eq!(
        scratch[1],
        PluginEvent::Midi(MidiEvent {
            offset_frames: 5,
            status: 0xD2,
            data1: 64,
            data2: 0,
        })
    );
    assert_eq!(
        scratch[2],
        PluginEvent::NoteExpression(NoteExpressionEvent {
            offset_frames: 7,
            note_id: -1,
            port_index: 0,
            channel: 2,
            key: 64,
            expression: NoteExpressionKind::Tuning,
            value: 37.5,
        })
    );
}
