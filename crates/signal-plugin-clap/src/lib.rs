//! CLAP plugin adapter surfaces for Signal.

use std::io;

use signal_ipc::{
    PluginIoLayoutPayload, PluginMessageEnvelope, PluginMessageName, PluginMessagePayload,
    SharedMemoryBroker, SharedMemoryLayoutPayload, SharedMemoryRegionPayload,
    SharedMemoryTransportPayload,
};
use signal_plugin::{
    AudioBlock, BlockDispatch, BlockPayload, BlockProcessResult, BlockProcessingHeader,
    EventPacket, MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent, NoteExpressionKind,
    ParameterGestureEvent, ParameterGesturePhase, ParameterModulationEvent, ParameterValueEvent,
    PluginDescriptor, PluginEvent, PluginFormat, PluginInstanceId, PluginIoLayout,
    PluginLifecycleContract, PluginProcessingContract, PluginRenderContext,
    PluginSandboxCapabilities, PluginTypeId, SandboxTransport, SharedMemoryLayout,
    SharedMemoryLease, SharedMemoryRegion,
};

mod clap_sandbox_harness;

pub use clap_sandbox_harness::{
    classify_sandbox_failure, sandbox_failure_event, ClapSandboxFailureClassification,
    ClapSandboxFailureStage, ClapSandboxLifecycleHarness,
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

fn translate_input_events(packet: &EventPacket) -> ClapEventPacket {
    let events = packet
        .events
        .iter()
        .map(|event| match event {
            PluginEvent::ParameterValue(event) => ClapEvent::ParamValue(ClapParamValueEvent {
                offset_frames: event.offset_frames,
                clap_param_id: event.parameter_id,
                normalized_value: event.normalized_value as f64,
            }),
            PluginEvent::ParameterModulation(event) => {
                ClapEvent::ParamModulation(ClapParamModEvent {
                    offset_frames: event.offset_frames,
                    clap_param_id: event.parameter_id,
                    amount: event.amount as f64,
                })
            }
            PluginEvent::ParameterGesture(event) => {
                ClapEvent::ParamGesture(ClapParamGestureEvent {
                    offset_frames: event.offset_frames,
                    clap_param_id: event.parameter_id,
                    phase: match event.phase {
                        ParameterGesturePhase::Begin => ClapParamGesturePhase::Begin,
                        ParameterGesturePhase::End => ClapParamGesturePhase::End,
                    },
                })
            }
            PluginEvent::Note(event) => ClapEvent::Note(ClapNoteEvent {
                offset_frames: event.offset_frames,
                note_id: event.note_id,
                port_index: event.port_index,
                channel: event.channel,
                key: event.key,
                velocity: event.velocity as f64,
                kind: match event.kind {
                    NoteEventKind::NoteOn => ClapNoteEventKind::NoteOn,
                    NoteEventKind::NoteOff => ClapNoteEventKind::NoteOff,
                },
            }),
            PluginEvent::NoteExpression(event) => {
                ClapEvent::NoteExpression(ClapNoteExpressionEvent {
                    offset_frames: event.offset_frames,
                    note_id: event.note_id,
                    port_index: event.port_index,
                    channel: event.channel,
                    key: event.key,
                    expression: clap_note_expression_kind(event.expression),
                    value: event.value as f64,
                })
            }
            PluginEvent::Midi(event) => {
                if let Some(note_event) = midi_to_clap_note(*event) {
                    ClapEvent::Note(note_event)
                } else if let Some(note_expression) = midi_to_clap_note_expression(*event) {
                    ClapEvent::NoteExpression(note_expression)
                } else {
                    ClapEvent::Midi(ClapMidiEvent {
                        offset_frames: event.offset_frames,
                        port_index: 0,
                        data: [event.status, event.data1, event.data2],
                    })
                }
            }
        })
        .collect();
    ClapEventPacket { events }
}

fn translate_output_events(packet: &ClapEventPacket) -> EventPacket {
    let events = packet
        .events
        .iter()
        .map(|event| match event {
            ClapEvent::ParamValue(event) => PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: event.offset_frames,
                parameter_id: event.clap_param_id,
                normalized_value: event.normalized_value as f32,
            }),
            ClapEvent::ParamModulation(event) => {
                PluginEvent::ParameterModulation(ParameterModulationEvent {
                    offset_frames: event.offset_frames,
                    parameter_id: event.clap_param_id,
                    amount: event.amount as f32,
                })
            }
            ClapEvent::ParamGesture(event) => {
                PluginEvent::ParameterGesture(ParameterGestureEvent {
                    offset_frames: event.offset_frames,
                    parameter_id: event.clap_param_id,
                    phase: match event.phase {
                        ClapParamGesturePhase::Begin => ParameterGesturePhase::Begin,
                        ClapParamGesturePhase::End => ParameterGesturePhase::End,
                    },
                })
            }
            ClapEvent::Note(event) => PluginEvent::Note(NoteEvent {
                offset_frames: event.offset_frames,
                note_id: event.note_id,
                port_index: event.port_index,
                channel: event.channel,
                key: event.key,
                velocity: event.velocity as f32,
                kind: match event.kind {
                    ClapNoteEventKind::NoteOn => NoteEventKind::NoteOn,
                    ClapNoteEventKind::NoteOff => NoteEventKind::NoteOff,
                },
            }),
            ClapEvent::NoteExpression(event) => PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: event.offset_frames,
                note_id: event.note_id,
                port_index: event.port_index,
                channel: event.channel,
                key: event.key,
                expression: plugin_note_expression_kind(event.expression),
                value: event.value as f32,
            }),
            ClapEvent::Midi(event) => PluginEvent::Midi(MidiEvent {
                offset_frames: event.offset_frames,
                status: event.data[0],
                data1: event.data[1],
                data2: event.data[2],
            }),
        })
        .collect();
    EventPacket::new(events)
}

fn clap_note_expression_kind(kind: NoteExpressionKind) -> ClapNoteExpressionKind {
    match kind {
        NoteExpressionKind::Pressure => ClapNoteExpressionKind::Pressure,
        NoteExpressionKind::Timbre => ClapNoteExpressionKind::Timbre,
        NoteExpressionKind::Tuning => ClapNoteExpressionKind::Tuning,
    }
}

fn plugin_note_expression_kind(kind: ClapNoteExpressionKind) -> NoteExpressionKind {
    match kind {
        ClapNoteExpressionKind::Pressure => NoteExpressionKind::Pressure,
        ClapNoteExpressionKind::Timbre => NoteExpressionKind::Timbre,
        ClapNoteExpressionKind::Tuning => NoteExpressionKind::Tuning,
    }
}

fn midi_to_clap_note(event: MidiEvent) -> Option<ClapNoteEvent> {
    let status = event.status & 0xF0;
    let channel = event.status & 0x0F;
    match status {
        0x90 if event.data2 > 0 => Some(ClapNoteEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel,
            key: event.data1,
            velocity: f64::from(event.data2) / 127.0,
            kind: ClapNoteEventKind::NoteOn,
        }),
        0x80 | 0x90 => Some(ClapNoteEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel,
            key: event.data1,
            velocity: f64::from(event.data2) / 127.0,
            kind: ClapNoteEventKind::NoteOff,
        }),
        _ => None,
    }
}

fn midi_to_clap_note_expression(event: MidiEvent) -> Option<ClapNoteExpressionEvent> {
    let status = event.status & 0xF0;
    let channel = event.status & 0x0F;
    match status {
        0xA0 => Some(ClapNoteExpressionEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel,
            key: event.data1,
            expression: ClapNoteExpressionKind::Pressure,
            value: f64::from(event.data2) / 127.0,
        }),
        _ => None,
    }
}

fn io_layout_payload(io_layout: PluginIoLayout) -> PluginIoLayoutPayload {
    PluginIoLayoutPayload {
        audio_inputs: io_layout.audio_inputs,
        audio_outputs: io_layout.audio_outputs,
        midi_inputs: io_layout.midi_inputs,
        midi_outputs: io_layout.midi_outputs,
    }
}

fn io_layout_from_payload(payload: PluginIoLayoutPayload) -> PluginIoLayout {
    PluginIoLayout {
        audio_inputs: payload.audio_inputs,
        audio_outputs: payload.audio_outputs,
        midi_inputs: payload.midi_inputs,
        midi_outputs: payload.midi_outputs,
    }
}

fn shared_memory_layout_payload(layout: SharedMemoryLayout) -> SharedMemoryLayoutPayload {
    SharedMemoryLayoutPayload {
        audio_input: shared_region_payload(layout.audio_input),
        audio_output: shared_region_payload(layout.audio_output),
        event_input: shared_region_payload(layout.event_input),
        event_output: shared_region_payload(layout.event_output),
        render_context: shared_region_payload(layout.render_context),
        completion: shared_region_payload(layout.completion),
    }
}

fn shared_memory_layout(payload: SharedMemoryLayoutPayload) -> SharedMemoryLayout {
    SharedMemoryLayout {
        audio_input: shared_region(payload.audio_input),
        audio_output: shared_region(payload.audio_output),
        event_input: shared_region(payload.event_input),
        event_output: shared_region(payload.event_output),
        render_context: shared_region(payload.render_context),
        completion: shared_region(payload.completion),
    }
}

fn shared_region_payload(region: SharedMemoryRegion) -> SharedMemoryRegionPayload {
    SharedMemoryRegionPayload {
        offset_bytes: region.offset_bytes,
        size_bytes: region.size_bytes,
    }
}

fn shared_region(payload: SharedMemoryRegionPayload) -> SharedMemoryRegion {
    SharedMemoryRegion {
        offset_bytes: payload.offset_bytes,
        size_bytes: payload.size_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_sandbox_failure, sandbox_failure_event, ClapBlockProtocol, ClapEvent,
        ClapHostExtension, ClapNoteExpressionEvent, ClapNoteExpressionKind, ClapParamGestureEvent,
        ClapParamGesturePhase, ClapPluginHostAdapter, ClapSandboxFailureStage,
        ClapSandboxLifecycleHarness,
    };
    use signal_ipc::{
        PluginDescriptorPayload, PluginMessageName, PluginMessagePayload, SharedMemoryBroker,
        SharedMemoryTransportKind,
    };
    use signal_plugin::{CompletionState, EventPacket, PluginFormat, PluginIoLayout};
    use std::{
        fs,
        path::PathBuf,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn test_broker_root(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "signal-plugin-clap-tests-{}-{name}-{timestamp}",
            process::id()
        ))
    }

    #[test]
    fn clap_adapter_reports_supported_format_and_extensions() {
        let adapter = ClapPluginHostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Clap));
        assert!(adapter
            .minimum_extension_set()
            .iter()
            .any(|extension| *extension == ClapHostExtension::Params));
        assert_eq!(adapter.minimum_extension_set()[0].as_str(), "audio-ports");
    }

    #[test]
    fn clap_adapter_discovers_concrete_plugin_type_metadata() {
        let adapter = ClapPluginHostAdapter::default();
        let discovered = adapter
            .discover_plugin_type("plugin:clap:sandbox")
            .expect("discovered sandbox plugin");

        assert_eq!(discovered.plugin_type_id.0, "plugin:clap:sandbox");
        assert_eq!(discovered.descriptor.plugin_id, "plugin:clap:sandbox");
        assert_eq!(discovered.descriptor.name, "Signal Sandbox CLAP Plugin");
        assert_eq!(discovered.descriptor.format, PluginFormat::Clap);
        assert_eq!(discovered.default_io_layout.audio_inputs, 2);
        assert_eq!(discovered.default_io_layout.audio_outputs, 2);
        assert_eq!(discovered.default_io_layout.midi_inputs, 1);
        assert_eq!(discovered.default_io_layout.midi_outputs, 1);
    }

    #[test]
    fn clap_protocol_descriptor_projects_plugin_neutral_contract_surface() {
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-a",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            2048,
        );

        let descriptor = protocol.descriptor();
        assert_eq!(descriptor.plugin_id, "plugin:clap:test");
        assert_eq!(descriptor.version.as_deref(), Some("0.1.0"));
        assert_eq!(descriptor.audio_buses.len(), 2);
        assert_eq!(descriptor.parameters.len(), 2);
        assert!(descriptor.processing_contract.sample_accurate_automation);
        assert!(descriptor.processing_contract.accepts_midi);
        assert!(descriptor.state_contract.supports_snapshot);
        assert!(descriptor.lifecycle_contract.supports_reset_while_active);
    }

    #[test]
    fn clap_lifecycle_sequence_builds_prepare_and_activate_requests() {
        let root = test_broker_root("sequence");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-a",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            2048,
        );

        let messages = protocol
            .lifecycle_sequence(&broker, "sandbox-a", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        assert_eq!(messages.len(), 5);
        assert_eq!(
            messages[0].message.name,
            PluginMessageName::SandboxHandshake.as_str()
        );

        match &messages[3].payload {
            PluginMessagePayload::PrepareInstanceRequest {
                processing_epoch,
                shared_memory_lease_id,
                shared_memory_transport,
                sample_rate_hz,
                max_block_frames,
                shared_memory,
                ..
            } => {
                assert_eq!(*processing_epoch, 1);
                assert_eq!(*sample_rate_hz, 48_000);
                assert_eq!(*max_block_frames, 512);
                assert!(shared_memory_lease_id.contains("sandbox-a"));
                assert_eq!(
                    shared_memory_transport.transport_kind,
                    SharedMemoryTransportKind::MappedFile
                );
                assert!(std::path::Path::new(&shared_memory_transport.backing_path).exists());
                assert!(shared_memory.total_bytes() > 0);
            }
            other => panic!("expected prepare request, got {other:?}"),
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_lifecycle_harness_accepts_full_control_sequence() {
        let root = test_broker_root("accept");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-b",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let messages = protocol
            .lifecycle_sequence(&broker, "sandbox-b", 48_000, 512, 1)
            .expect("build lifecycle sequence");

        let responses = messages
            .into_iter()
            .map(|message| harness.handle(message).expect("accepted request"))
            .collect::<Vec<_>>();

        assert_eq!(responses.len(), 5);
        assert_eq!(
            responses.last().expect("last response").message.name,
            PluginMessageName::SandboxActivateInstance.as_str()
        );
        match &responses.last().expect("last response").payload {
            PluginMessagePayload::ActivateInstanceResponse { instance_state, .. } => {
                assert_eq!(instance_state.lifecycle_state, "Active");
                assert_eq!(instance_state.readiness_state, "Ready");
                assert!(instance_state.active);
                assert!(instance_state.processing.is_some());
            }
            other => panic!("expected activate response, got {other:?}"),
        }
        assert_eq!(
            harness
                .lease()
                .expect("prepared lease")
                .invalidated_epochs()
                .len(),
            0
        );
        assert!(harness
            .lease()
            .and_then(|lease| lease.transport())
            .is_some());
        harness
            .teardown_active_transport()
            .expect("teardown transport");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_lifecycle_harness_rejects_unknown_plugin_type_requests() {
        let root = test_broker_root("unknown-plugin");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-unknown",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let mut messages = protocol
            .lifecycle_sequence(&broker, "sandbox-unknown", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        if let Some(load) = messages.get_mut(1) {
            load.payload = PluginMessagePayload::LoadPluginTypeRequest {
                sandbox_id: "sandbox-unknown".into(),
                plugin_type_id: "plugin:vst:missing".into(),
                descriptor: PluginDescriptorPayload {
                    plugin_id: "plugin:vst:missing".into(),
                    vendor: "Signal".into(),
                    name: "Missing Plugin".into(),
                    format: "clap".into(),
                },
            };
        }

        harness
            .handle(messages.remove(0))
            .expect("accepted handshake");

        match harness
            .handle(messages.remove(0))
            .expect_err("missing plugin failure")
            .payload
        {
            PluginMessagePayload::SandboxFailure {
                error_kind,
                detail,
                fault,
                ..
            } => {
                assert_eq!(error_kind, "unsupported");
                assert!(detail.contains("not available in the local catalog"));
                assert_eq!(fault.kind, "unsupportedCapability");
                assert_eq!(fault.severity, "warning");
            }
            other => panic!("expected sandbox failure, got {other:?}"),
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_lifecycle_harness_can_invalidate_active_epoch() {
        let root = test_broker_root("invalidate");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-invalidate",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let messages = protocol
            .lifecycle_sequence(&broker, "sandbox-invalidate", 48_000, 512, 3)
            .expect("build lifecycle sequence");

        for message in messages {
            harness.handle(message).expect("accepted request");
        }

        let (completion_invalidated, lease_invalidated) = harness.invalidate_active_epoch(3);
        assert!(completion_invalidated);
        assert!(lease_invalidated);
        assert_eq!(
            harness
                .lease()
                .expect("prepared lease")
                .invalidated_epochs(),
            &[3]
        );

        harness
            .teardown_active_transport()
            .expect("teardown transport");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_lifecycle_harness_rejects_prepare_requests_above_contract_limit() {
        let root = test_broker_root("prepare-limit");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-prepare-limit",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let mut messages = protocol
            .lifecycle_sequence(&broker, "sandbox-prepare-limit", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        if let Some(prepare) = messages.get_mut(3) {
            let original_payload = prepare.payload.clone();
            match original_payload {
                PluginMessagePayload::PrepareInstanceRequest {
                    sandbox_id,
                    instance_id,
                    processing_epoch,
                    shared_memory_lease_id,
                    shared_memory_transport,
                    sample_rate_hz,
                    io_layout,
                    shared_memory,
                    ..
                } => {
                    prepare.payload = PluginMessagePayload::PrepareInstanceRequest {
                        sandbox_id,
                        instance_id,
                        processing_epoch,
                        shared_memory_lease_id,
                        shared_memory_transport,
                        sample_rate_hz,
                        max_block_frames: 8_192,
                        io_layout,
                        shared_memory,
                    };
                }
                other => panic!("expected prepare request, got {other:?}"),
            }
        }
        let mut messages = messages.into_iter();
        harness
            .handle(messages.next().expect("handshake request"))
            .expect("accepted handshake");
        harness
            .handle(messages.next().expect("load request"))
            .expect("accepted load");
        harness
            .handle(messages.next().expect("create request"))
            .expect("accepted create");

        match harness.handle(messages.next().expect("prepare request")) {
            Ok(_) => panic!("expected prepare failure"),
            Err(failure) => match failure.payload {
                PluginMessagePayload::SandboxFailure {
                    error_kind,
                    detail,
                    processing_epoch,
                    shared_memory_lease_id,
                    fault,
                    ..
                } => {
                    assert_eq!(error_kind, "resourceUnavailable");
                    assert!(detail.contains("exceeds discovered CLAP processing contract"));
                    assert_eq!(processing_epoch, Some(1));
                    assert!(shared_memory_lease_id.is_some());
                    assert_eq!(fault.kind, "resourceUnavailable");
                    assert_eq!(fault.severity, "recoverable");
                }
                other => panic!("expected sandbox failure, got {other:?}"),
            },
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_lifecycle_harness_emits_failure_and_invalidates_epoch() {
        let root = test_broker_root("failure");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-fault",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let mut messages = protocol
            .lifecycle_sequence(&broker, "sandbox-fault", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        if let Some(activate) = messages.last_mut() {
            activate.payload = PluginMessagePayload::ActivateInstanceRequest {
                sandbox_id: "sandbox-fault".into(),
                instance_id: "instance-fault".into(),
                processing_epoch: 9,
            };
        }

        let mut last_failure = None;
        for message in messages {
            match harness.handle(message) {
                Ok(_) => {}
                Err(failure) => {
                    last_failure = Some(failure);
                    break;
                }
            }
        }

        match last_failure.expect("failure envelope").payload {
            PluginMessagePayload::SandboxFailure {
                error_kind,
                fault,
                processing_epoch,
                ..
            } => {
                assert_eq!(error_kind, "protocolViolation");
                assert_eq!(fault.kind, "protocolViolation");
                assert_eq!(fault.severity, "critical");
                assert_eq!(processing_epoch, Some(9));
            }
            other => panic!("expected sandbox failure envelope, got {other:?}"),
        }
        assert!(!harness.lease().expect("lease").is_epoch_valid(9));
        harness
            .teardown_active_transport()
            .expect("teardown transport");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_shared_memory_header_scales_with_channel_count() {
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-c",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let header = protocol.block_header(1, 2, 512);
        assert_eq!(header.block.channel_count, 2);
        assert!(header.layout.audio_input.size_bytes > 0);
    }

    #[test]
    fn clap_event_translation_upgrades_note_and_modulation_semantics() {
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-translate",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            1024,
        );
        let payload = protocol.test_input_payload(3, 512);

        let clap_events = protocol.translate_input_events(&payload.events);
        assert_eq!(clap_events.events.len(), 11);
        assert!(matches!(clap_events.events[0], ClapEvent::ParamGesture(_)));
        assert!(matches!(clap_events.events[1], ClapEvent::ParamGesture(_)));
        assert!(matches!(clap_events.events[2], ClapEvent::ParamValue(_)));
        assert!(matches!(clap_events.events[3], ClapEvent::ParamValue(_)));
        assert!(matches!(
            clap_events.events[4],
            ClapEvent::ParamModulation(_)
        ));
        assert!(matches!(
            clap_events.events[5],
            ClapEvent::ParamModulation(_)
        ));
        assert!(matches!(clap_events.events[6], ClapEvent::Note(_)));
        assert!(matches!(
            clap_events.events[7],
            ClapEvent::NoteExpression(ClapNoteExpressionEvent {
                expression: ClapNoteExpressionKind::Timbre,
                ..
            })
        ));
        assert!(matches!(
            clap_events.events[8],
            ClapEvent::NoteExpression(ClapNoteExpressionEvent {
                expression: ClapNoteExpressionKind::Tuning,
                ..
            })
        ));
        assert!(matches!(
            clap_events.events[9],
            ClapEvent::NoteExpression(ClapNoteExpressionEvent {
                expression: ClapNoteExpressionKind::Pressure,
                ..
            })
        ));
        assert!(matches!(clap_events.events[10], ClapEvent::Midi(_)));
        assert!(matches!(
            clap_events.events[1],
            ClapEvent::ParamGesture(ClapParamGestureEvent {
                phase: ClapParamGesturePhase::End,
                ..
            })
        ));

        let round_tripped = protocol.translate_output_events(&clap_events);
        let summary = round_tripped.summary();
        assert_eq!(summary.parameter_value_events, 2);
        assert_eq!(summary.parameter_gesture_events, 2);
        assert_eq!(summary.parameter_modulation_events, 2);
        assert_eq!(summary.note_events, 1);
        assert_eq!(summary.note_expression_events, 3);
        assert_eq!(summary.midi_events, 1);
        let automation =
            round_tripped.parameter_automation_summary(protocol.automation_parameter_id());
        assert_eq!(automation.value_events, 1);
        assert_eq!(automation.modulation_events, 1);
        assert_eq!(automation.gesture_begin_events, 0);
        assert_eq!(automation.gesture_end_events, 1);
        assert_eq!(automation.first_value, Some(0.25));
        assert_eq!(automation.last_value, Some(0.25));
        assert_eq!(automation.last_modulation, Some(-0.02));
    }

    #[test]
    fn clap_harness_processes_brokered_block_and_heartbeat() {
        let root = test_broker_root("block");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-block",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let messages = protocol
            .lifecycle_sequence(&broker, "sandbox-block", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        let responses = messages
            .into_iter()
            .map(|message| harness.handle(message).expect("accepted request"))
            .collect::<Vec<_>>();
        let transport = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::PrepareInstanceResponse {
                    shared_memory_transport,
                    ..
                } => Some(shared_memory_transport.clone()),
                _ => None,
            })
            .expect("prepare transport");

        let heartbeat = harness
            .handle(protocol.heartbeat_request("sandbox-block", Some(1)))
            .expect("heartbeat response");
        match heartbeat.payload {
            PluginMessagePayload::HeartbeatResponse { active, .. } => assert!(active),
            other => panic!("expected heartbeat response, got {other:?}"),
        }
        assert_eq!(harness.heartbeat_count(), 1);

        let dispatch = protocol.block_dispatch(1, 4, 512, protocol.default_render_context(512));
        let payload = protocol.test_input_payload(4, 512);
        protocol
            .write_block_payload(&broker, &transport, &dispatch, &payload)
            .expect("write block payload");
        let result = harness.process_pending_block().expect("process block");
        assert_eq!(result.slot.state, CompletionState::Completed);
        let expected_output_events =
            protocol.translate_output_events(&protocol.translate_input_events(&payload.events));
        assert_eq!(
            result.generated_event_bytes,
            expected_output_events.encoded_bytes()
        );

        let stored_outcome = protocol
            .read_block_outcome(&broker, &transport, &dispatch)
            .expect("read block outcome");
        assert_eq!(stored_outcome.result.slot.state, CompletionState::Completed);
        assert_eq!(stored_outcome.input, payload);
        assert_eq!(stored_outcome.output.audio, stored_outcome.input.audio);
        assert_eq!(stored_outcome.output.events, expected_output_events);

        harness
            .teardown_active_transport()
            .expect("teardown transport");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_harness_round_trips_multi_block_payload_sequence() {
        let root = test_broker_root("multi-block");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-multi-block",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let messages = protocol
            .lifecycle_sequence(&broker, "sandbox-multi-block", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        let responses = messages
            .into_iter()
            .map(|message| harness.handle(message).expect("accepted request"))
            .collect::<Vec<_>>();
        let transport = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::PrepareInstanceResponse {
                    shared_memory_transport,
                    ..
                } => Some(shared_memory_transport.clone()),
                _ => None,
            })
            .expect("prepare transport");

        let mut aggregated_output_events = EventPacket::new(Vec::new());
        for block_sequence in 0..4 {
            let dispatch = protocol.block_dispatch(
                1,
                block_sequence,
                512,
                protocol.default_render_context(512),
            );
            let payload = protocol.test_input_payload(block_sequence, 512);
            protocol
                .write_block_payload(&broker, &transport, &dispatch, &payload)
                .expect("write block payload");
            let result = harness.process_pending_block().expect("process block");
            assert_eq!(result.slot.block_sequence, block_sequence);
            assert_eq!(result.slot.state, CompletionState::Completed);

            let outcome = protocol
                .read_block_outcome(&broker, &transport, &dispatch)
                .expect("read block outcome");
            assert_eq!(outcome.input, payload);
            let expected_output_events =
                protocol.translate_output_events(&protocol.translate_input_events(&payload.events));
            assert_eq!(outcome.output.audio, outcome.input.audio);
            assert_eq!(outcome.output.events, expected_output_events);
            assert_eq!(
                outcome.output.audio.first_sample(),
                Some(block_sequence as f32)
            );
            assert_eq!(outcome.output.events.event_count(), 11);
            aggregated_output_events
                .events
                .extend(outcome.output.events.events.iter().copied());
        }

        let automation = aggregated_output_events
            .parameter_automation_summary(protocol.automation_parameter_id());
        assert_eq!(automation.value_events, 4);
        assert_eq!(automation.modulation_events, 4);
        assert_eq!(automation.gesture_begin_events, 1);
        assert_eq!(automation.gesture_end_events, 3);
        assert_eq!(automation.first_value, Some(0.1));
        assert_eq!(automation.last_value, Some(0.25));
        assert_eq!(automation.last_modulation, Some(-0.02));

        harness
            .teardown_active_transport()
            .expect("teardown transport");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_harness_marks_deadline_miss_in_completion_region() {
        let root = test_broker_root("timeout");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-timeout",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let messages = protocol
            .lifecycle_sequence(&broker, "sandbox-timeout", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        let responses = messages
            .into_iter()
            .map(|message| harness.handle(message).expect("accepted request"))
            .collect::<Vec<_>>();
        let transport = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::PrepareInstanceResponse {
                    shared_memory_transport,
                    ..
                } => Some(shared_memory_transport.clone()),
                _ => None,
            })
            .expect("prepare transport");

        let dispatch = protocol.block_dispatch(1, 5, 512, protocol.default_render_context(512));
        protocol
            .write_block_dispatch(&broker, &transport, &dispatch)
            .expect("write block dispatch");
        let result = harness.mark_deadline_miss().expect("mark deadline miss");
        assert_eq!(result.slot.state, CompletionState::TimedOut);
        assert!(result.fallback_applied);

        let stored_result = protocol
            .read_block_result(&broker, &transport, 512)
            .expect("read block result");
        assert_eq!(stored_result.slot.state, CompletionState::TimedOut);

        harness
            .teardown_active_transport()
            .expect("teardown transport");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_harness_accepts_deactivate_reset_and_destroy_sequence() {
        let root = test_broker_root("teardown");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-teardown",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let messages = protocol
            .lifecycle_sequence(&broker, "sandbox-teardown", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        for message in messages {
            harness.handle(message).expect("accepted request");
        }

        let teardown_responses = protocol
            .teardown_sequence("sandbox-teardown", 2)
            .into_iter()
            .map(|message| harness.handle(message).expect("accepted teardown request"))
            .collect::<Vec<_>>();

        assert_eq!(
            teardown_responses[0].message.name,
            PluginMessageName::SandboxDeactivateInstance.as_str()
        );
        assert_eq!(
            teardown_responses[1].message.name,
            PluginMessageName::SandboxResetInstance.as_str()
        );
        assert_eq!(
            teardown_responses[2].message.name,
            PluginMessageName::SandboxDestroyInstance.as_str()
        );
        assert!(harness.lease().is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sandbox_failure_event_exposes_timeout_kind() {
        let failure = sandbox_failure_event(
            "sandbox-a",
            Some("instance-a".into()),
            "processBlock",
            "timeout",
            "sandbox exceeded block deadline",
            Some(3),
            Some("lease-a".into()),
            None,
        );

        match failure.payload {
            PluginMessagePayload::SandboxFailure { error_kind, .. } => {
                assert_eq!(error_kind, "timeout");
            }
            other => panic!("expected sandbox failure, got {other:?}"),
        }
    }

    #[test]
    fn classify_sandbox_failure_maps_process_attach_errors() {
        let failure = sandbox_failure_event(
            "sandbox-a",
            Some("instance-a".into()),
            "processBlock",
            "resourceUnavailable",
            "failed to attach shared-memory region: stale mapping",
            Some(3),
            Some("lease-a".into()),
            None,
        );

        let classification = classify_sandbox_failure(&failure).expect("classification");
        assert_eq!(classification.stage, ClapSandboxFailureStage::ProcessAttach);
        assert_eq!(classification.operation, "processBlock");
        assert_eq!(classification.lease_id.as_deref(), Some("lease-a"));
    }
}
