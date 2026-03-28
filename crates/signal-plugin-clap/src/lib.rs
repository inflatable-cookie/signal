//! CLAP plugin adapter surfaces for Signal.

use std::io;

use signal_ipc::{
    PluginIoLayoutPayload, PluginMessageEnvelope, PluginMessageName, PluginMessagePayload,
    SharedMemoryBroker, SharedMemoryLayoutPayload, SharedMemoryRegionPayload,
    SharedMemoryTransportPayload,
};
use signal_plugin::{
    AudioBlock, BlockDispatch, BlockPayload, BlockProcessResult, BlockProcessingHeader,
    EventPacket, MidiEvent, NoteExpressionEvent, NoteExpressionKind, ParameterGestureEvent,
    ParameterGesturePhase, ParameterModulationEvent, ParameterValueEvent, PluginDescriptor,
    PluginEvent, PluginFormat, PluginInstanceId, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginRenderContext, PluginSandboxCapabilities, PluginTypeId,
    SandboxTransport, SharedMemoryLayout, SharedMemoryLease,
};

mod clap_sandbox_harness;
mod event_translation;

pub use clap_sandbox_harness::{
    classify_sandbox_failure, sandbox_failure_event, ClapHarnessError, ClapHarnessResult,
    ClapSandboxFailureClassification, ClapSandboxFailureInput, ClapSandboxFailureStage,
    ClapSandboxLifecycleHarness,
};
pub use event_translation::{
    io_layout_from_payload, io_layout_payload, shared_memory_layout, shared_memory_layout_payload,
    translate_input_events, translate_output_events,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapHostExtension {
    AudioPorts,
    NotePorts,
    Params,
    State,
    Latency,
    Tail,
}

impl ClapHostExtension {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AudioPorts => "audio-ports",
            Self::NotePorts => "note-ports",
            Self::Params => "params",
            Self::State => "state",
            Self::Latency => "latency",
            Self::Tail => "tail",
        }
    }
}

const MINIMUM_CLAP_EXTENSIONS: [ClapHostExtension; 4] = [
    ClapHostExtension::AudioPorts,
    ClapHostExtension::NotePorts,
    ClapHostExtension::Params,
    ClapHostExtension::State,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClapPluginHostAdapter {
    strict_sandbox_default: bool,
}

impl Default for ClapPluginHostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
        }
    }
}

impl ClapPluginHostAdapter {
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Clap)
    }

    pub fn minimum_extension_set(&self) -> &'static [ClapHostExtension] {
        &MINIMUM_CLAP_EXTENSIONS
    }

    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }

    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<ClapDiscoveredPluginType> {
        clap_sandbox_harness::clap_discovered_plugin_type(plugin_type_id)
    }

    pub fn instantiate_plugin(
        &self,
        discovered: &ClapDiscoveredPluginType,
        instance_id: &str,
    ) -> ClapInstanceControlSurface {
        ClapInstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClapDiscoveredPluginType {
    pub plugin_type_id: PluginTypeId,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClapInstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSharedMemoryHeader {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub block: BlockProcessingHeader,
    pub layout: SharedMemoryLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapPreparePlan {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub lease: SharedMemoryLease,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrokeredBlockOutcome {
    pub dispatch: BlockDispatch,
    pub input: BlockPayload,
    pub output: BlockPayload,
    pub result: BlockProcessResult,
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct ClapBlockProtocol {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
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
            instance_id: PluginInstanceId(instance_id.into()),
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

    pub fn prepare_plan(
        &self,
        broker: &SharedMemoryBroker,
        sandbox_id: &str,
        sample_rate_hz: u32,
        max_block_frames: u32,
        processing_epoch: u64,
    ) -> io::Result<ClapPreparePlan> {
        let layout = self.shared_memory_layout(max_block_frames);
        let region = broker.create_region(
            &format!(
                "{sandbox_id}:{}:epoch-{processing_epoch}",
                self.instance_id.0
            ),
            layout.total_bytes(),
        )?;
        let transport = region.metadata().clone();

        Ok(ClapPreparePlan {
            plugin_type_id: self.plugin_type_id.clone(),
            instance_id: self.instance_id.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: self.io_layout,
            lease: SharedMemoryLease::new(
                format!(
                    "{sandbox_id}:{}:epoch-{processing_epoch}",
                    self.instance_id.0
                ),
                processing_epoch,
                layout,
            )
            .with_transport(transport),
        })
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

    pub fn lifecycle_sequence(
        &self,
        broker: &SharedMemoryBroker,
        sandbox_id: &str,
        sample_rate_hz: u32,
        max_block_frames: u32,
        processing_epoch: u64,
    ) -> io::Result<Vec<PluginMessageEnvelope>> {
        let descriptor = self.descriptor();
        let prepare = self.prepare_plan(
            broker,
            sandbox_id,
            sample_rate_hz,
            max_block_frames,
            processing_epoch,
        )?;
        let transport = prepare
            .lease
            .transport()
            .cloned()
            .expect("brokered transport");

        Ok(vec![
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxHandshake,
                format!("{sandbox_id}:handshake"),
                PluginMessagePayload::SandboxHandshakeRequest {
                    sandbox_id: sandbox_id.into(),
                    format: "clap".into(),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxLoadPluginType,
                format!("{sandbox_id}:load"),
                PluginMessagePayload::LoadPluginTypeRequest {
                    sandbox_id: sandbox_id.into(),
                    plugin_type_id: self.plugin_type_id.0.clone(),
                    descriptor: clap_sandbox_harness::descriptor_payload(&descriptor),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxCreateInstance,
                format!("{sandbox_id}:create"),
                PluginMessagePayload::CreateInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    plugin_type_id: self.plugin_type_id.0.clone(),
                    instance_id: self.instance_id.0.clone(),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxPrepareInstance,
                format!("{sandbox_id}:prepare"),
                PluginMessagePayload::PrepareInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                    processing_epoch,
                    shared_memory_lease_id: prepare.lease.lease_id.clone(),
                    shared_memory_transport: transport.clone(),
                    sample_rate_hz,
                    max_block_frames,
                    io_layout: io_layout_payload(prepare.io_layout),
                    shared_memory: shared_memory_layout_payload(prepare.lease.layout),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxActivateInstance,
                format!("{sandbox_id}:activate"),
                PluginMessagePayload::ActivateInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                    processing_epoch,
                },
            ),
        ])
    }

    pub fn heartbeat_request(
        &self,
        sandbox_id: &str,
        processing_epoch: Option<u64>,
    ) -> PluginMessageEnvelope {
        PluginMessageEnvelope::command(
            PluginMessageName::SandboxHeartbeat,
            format!("{sandbox_id}:heartbeat"),
            PluginMessagePayload::HeartbeatRequest {
                sandbox_id: sandbox_id.into(),
                instance_id: Some(self.instance_id.0.clone()),
                processing_epoch,
            },
        )
    }

    pub fn teardown_sequence(
        &self,
        sandbox_id: &str,
        processing_epoch: u64,
    ) -> Vec<PluginMessageEnvelope> {
        vec![
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxDeactivateInstance,
                format!("{sandbox_id}:deactivate"),
                PluginMessagePayload::DeactivateInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxResetInstance,
                format!("{sandbox_id}:reset"),
                PluginMessagePayload::ResetInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                    processing_epoch,
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxDestroyInstance,
                format!("{sandbox_id}:destroy"),
                PluginMessagePayload::DestroyInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                },
            ),
        ]
    }

    pub fn write_block_dispatch(
        &self,
        broker: &SharedMemoryBroker,
        transport: &SharedMemoryTransportPayload,
        dispatch: &BlockDispatch,
    ) -> io::Result<()> {
        self.write_block_payload(
            broker,
            transport,
            dispatch,
            &BlockPayload::new(
                AudioBlock::new(
                    dispatch.header.channel_count,
                    dispatch.header.frame_count,
                    vec![
                        0.0;
                        dispatch.header.channel_count as usize
                            * dispatch.header.frame_count as usize
                    ],
                )
                .expect("silence block"),
                EventPacket::new(Vec::new()),
            ),
        )
    }

    pub fn write_block_payload(
        &self,
        broker: &SharedMemoryBroker,
        transport: &SharedMemoryTransportPayload,
        dispatch: &BlockDispatch,
        payload: &BlockPayload,
    ) -> io::Result<()> {
        let mut region = broker.attach_region(transport)?;
        dispatch
            .write_to_shared_memory(region.as_mut_slice())
            .map_err(io::Error::other)?;
        dispatch
            .write_input_payload(region.as_mut_slice(), payload)
            .map_err(io::Error::other)?;
        BlockProcessResult::ready_for(dispatch)
            .write_to_shared_memory(dispatch.layout, region.as_mut_slice())
            .map_err(io::Error::other)?;
        region.flush()
    }

    pub fn read_block_result(
        &self,
        broker: &SharedMemoryBroker,
        transport: &SharedMemoryTransportPayload,
        frame_count: u32,
    ) -> io::Result<BlockProcessResult> {
        let region = broker.attach_region(transport)?;
        let layout = self.shared_memory_layout(frame_count);
        BlockProcessResult::read_from_shared_memory(layout, region.as_slice())
            .map_err(io::Error::other)
    }

    pub fn read_block_outcome(
        &self,
        broker: &SharedMemoryBroker,
        transport: &SharedMemoryTransportPayload,
        dispatch: &BlockDispatch,
    ) -> io::Result<BrokeredBlockOutcome> {
        let region = broker.attach_region(transport)?;
        let input = dispatch
            .read_input_payload(region.as_slice())
            .map_err(io::Error::other)?;
        let output = dispatch
            .read_output_payload(region.as_slice())
            .map_err(io::Error::other)?;
        let result =
            BlockProcessResult::read_from_shared_memory(dispatch.layout, region.as_slice())
                .map_err(io::Error::other)?;

        Ok(BrokeredBlockOutcome {
            dispatch: dispatch.clone(),
            input,
            output,
            result,
        })
    }

    fn shared_memory_layout(&self, max_block_frames: u32) -> SharedMemoryLayout {
        let audio_bytes = self.io_layout.audio_channels() as u32
            * max_block_frames
            * core::mem::size_of::<f32>() as u32;
        SharedMemoryLayout::single_block(audio_bytes, self.event_capacity_bytes)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
