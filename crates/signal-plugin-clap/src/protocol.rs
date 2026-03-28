use signal_plugin::{
    AudioBlock, BlockDispatch, BlockPayload, EventPacket, MidiEvent, NoteExpressionEvent,
    NoteExpressionKind, ParameterGestureEvent, ParameterGesturePhase, ParameterModulationEvent,
    ParameterValueEvent, PluginDescriptor, PluginEvent, PluginIoLayout, PluginRenderContext,
    PluginTypeId,
};

use crate::adapter::ClapSharedMemoryHeader;
use crate::clap_sandbox_harness;
use crate::event_translation::{translate_input_events, translate_output_events};
use crate::events::ClapEventPacket;

#[derive(Clone, Debug, PartialEq)]
pub struct ClapBlockProtocol {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: signal_plugin::PluginInstanceId,
    pub io_layout: PluginIoLayout,
    pub event_capacity_bytes: u32,
}

impl ClapBlockProtocol {
    pub fn automation_parameter_id(&self) -> u32 {
        4_096
    }

    pub fn automation_cycle_span_blocks(&self) -> u64 {
        4
    }

    pub fn new(
        plugin_type_id: impl Into<String>,
        instance_id: impl Into<String>,
        io_layout: PluginIoLayout,
        event_capacity_bytes: u32,
    ) -> Self {
        Self {
            plugin_type_id: PluginTypeId(plugin_type_id.into()),
            instance_id: signal_plugin::PluginInstanceId(instance_id.into()),
            io_layout,
            event_capacity_bytes,
        }
    }

    pub fn descriptor(&self) -> PluginDescriptor {
        clap_sandbox_harness::clap_fixture_descriptor(
            self.plugin_type_id.0.as_str(),
            self.io_layout,
        )
    }

    pub fn block_header(
        &self,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
    ) -> ClapSharedMemoryHeader {
        let dispatch = self.block_dispatch(
            processing_epoch,
            block_sequence,
            frame_count,
            self.default_render_context(frame_count),
        );

        ClapSharedMemoryHeader {
            plugin_type_id: self.plugin_type_id.clone(),
            instance_id: dispatch.instance_id,
            block: dispatch.header,
            layout: dispatch.layout,
        }
    }

    pub fn default_render_context(&self, frame_count: u32) -> PluginRenderContext {
        PluginRenderContext {
            sample_rate_hz: 48_000,
            tempo_bpm: 120.0,
            timeline_position_samples: 0,
            playing: true,
            bypassed: false,
            loop_range: None,
            deadline_frames: frame_count,
        }
    }

    pub fn block_dispatch(
        &self,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        render_context: PluginRenderContext,
    ) -> BlockDispatch {
        BlockDispatch::new(
            self.instance_id.clone(),
            processing_epoch,
            block_sequence,
            frame_count,
            self.io_layout,
            render_context,
            self.event_capacity_bytes,
        )
    }

    pub fn test_input_payload(&self, block_sequence: u64, frame_count: u32) -> BlockPayload {
        let channel_count = self.io_layout.audio_channels();
        let mut samples = Vec::with_capacity(channel_count as usize * frame_count as usize);
        for frame_index in 0..frame_count {
            for channel_index in 0..channel_count {
                samples.push(
                    block_sequence as f32
                        + channel_index as f32
                        + ((frame_index % 8) as f32 * 0.125),
                );
            }
        }

        let audio = AudioBlock::new(channel_count, frame_count, samples).expect("test audio");
        let automation_parameter_id = self.automation_parameter_id();
        let automation_cycle_phase = block_sequence % self.automation_cycle_span_blocks();
        let events = EventPacket::new(vec![
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: ((block_sequence as u32) * 2) % frame_count.max(1),
                parameter_id: 100 + block_sequence as u32,
                phase: ParameterGesturePhase::Begin,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: ((block_sequence as u32) * 11) % frame_count.max(1),
                parameter_id: automation_parameter_id,
                phase: if automation_cycle_phase == 0 {
                    ParameterGesturePhase::Begin
                } else {
                    ParameterGesturePhase::End
                },
            }),
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: (block_sequence as u32) % frame_count.max(1),
                parameter_id: 100 + block_sequence as u32,
                normalized_value: 0.25 + (block_sequence as f32 * 0.1),
            }),
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: ((block_sequence as u32) * 10) % frame_count.max(1),
                parameter_id: automation_parameter_id,
                normalized_value: 0.1 + (block_sequence as f32 * 0.05),
            }),
            PluginEvent::ParameterModulation(ParameterModulationEvent {
                offset_frames: ((block_sequence as u32) * 3) % frame_count.max(1),
                parameter_id: 200 + block_sequence as u32,
                amount: 0.05 + (block_sequence as f32 * 0.01),
            }),
            PluginEvent::ParameterModulation(ParameterModulationEvent {
                offset_frames: ((block_sequence as u32) * 12) % frame_count.max(1),
                parameter_id: automation_parameter_id,
                amount: -0.08 + (block_sequence as f32 * 0.02),
            }),
            PluginEvent::Midi(MidiEvent {
                offset_frames: ((block_sequence as u32) * 4) % frame_count.max(1),
                status: 0x90,
                data1: 60 + (block_sequence as u8 % 12),
                data2: 96,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: ((block_sequence as u32) * 5) % frame_count.max(1),
                note_id: -1,
                port_index: 0,
                channel: 0,
                key: 60 + (block_sequence as u8 % 12),
                expression: NoteExpressionKind::Timbre,
                value: 0.35 + (block_sequence as f32 * 0.02),
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: ((block_sequence as u32) * 6) % frame_count.max(1),
                note_id: -1,
                port_index: 0,
                channel: 0,
                key: 60 + (block_sequence as u8 % 12),
                expression: NoteExpressionKind::Tuning,
                value: -0.15 + (block_sequence as f32 * 0.01),
            }),
            PluginEvent::Midi(MidiEvent {
                offset_frames: ((block_sequence as u32) * 7) % frame_count.max(1),
                status: 0xA0,
                data1: 60 + (block_sequence as u8 % 12),
                data2: 48 + (block_sequence as u8 % 32),
            }),
            PluginEvent::Midi(MidiEvent {
                offset_frames: ((block_sequence as u32) * 8) % frame_count.max(1),
                status: 0xB0,
                data1: 1,
                data2: 64 + (block_sequence as u8 % 32),
            }),
        ]);

        BlockPayload::new(audio, events)
    }

    pub fn translate_input_events(&self, packet: &EventPacket) -> ClapEventPacket {
        translate_input_events(packet)
    }

    pub fn translate_output_events(&self, packet: &ClapEventPacket) -> EventPacket {
        translate_output_events(packet)
    }
}
