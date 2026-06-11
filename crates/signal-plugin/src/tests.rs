// Tests for signal-plugin
#[allow(clippy::module_inception)]
mod tests {
    use crate::{
        AudioBlock, BlockDispatch, BlockPayload, BlockProcessResult, CompletionSlot, CompletionState, EventPacket, LoopRange,
        MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent, NoteExpressionKind,
        ParameterAutomationSummary, ParameterGestureEvent, ParameterGesturePhase,
        ParameterModulationEvent, ParameterValueEvent, PluginDescriptor, PluginEvent,
        PluginFaultKind, PluginFaultSeverity, PluginFormat, PluginInstanceId, PluginIoLayout,
        PluginLifecycleState, PluginParameterDomain, PluginParameterFlags, PluginReadiness,
        PluginRenderContext, PluginSandboxCapabilities, PluginSandboxError, PluginSandboxErrorKind,
        RestartEscalationPolicy, RestartEscalationState, SandboxStateMachine, SandboxTransport,
        SandboxWatchdogPolicy,
        SandboxWatchdogState, SharedMemoryLayout, SharedMemoryLease, WatchdogOutcome,
        WatchdogTriggerReason,
    };
    use signal_ipc::{SharedMemoryTransportKind, SharedMemoryTransportPayload};

    fn test_render_context() -> PluginRenderContext {
        PluginRenderContext {
            sample_rate_hz: 48_000,
            tempo_bpm: 120.0,
            timeline_position_samples: 0,
            playing: true,
            bypassed: false,
            loop_range: Some(LoopRange {
                start_samples: 0,
                end_samples: 96_000,
            }),
            deadline_frames: 512,
        }
    }

    fn test_payload(dispatch: &BlockDispatch) -> BlockPayload {
        let sample_count =
            dispatch.header.channel_count as usize * dispatch.header.frame_count as usize;
        let audio = AudioBlock::new(
            dispatch.header.channel_count,
            dispatch.header.frame_count,
            (0..sample_count).map(|index| index as f32).collect(),
        )
        .expect("audio block");
        let events = EventPacket::new(vec![
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 32,
                parameter_id: 7,
                normalized_value: 0.5,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 40,
                parameter_id: 7,
                phase: ParameterGesturePhase::Begin,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 48,
                parameter_id: 7,
                phase: ParameterGesturePhase::End,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 56,
                note_id: 7,
                port_index: 0,
                channel: 0,
                key: 60,
                expression: NoteExpressionKind::Pressure,
                value: 0.6,
            }),
            PluginEvent::Midi(MidiEvent {
                offset_frames: 64,
                status: 0x90,
                data1: 60,
                data2: 96,
            }),
        ]);
        BlockPayload::new(audio, events)
    }

    mod block_io;
    mod events;
    mod shared_memory;
    mod watchdog;
}
