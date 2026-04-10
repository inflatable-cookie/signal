use super::introspection::{
    read_vst3_bundle_snapshot, Vst3FactoryClass, Vst3FactoryClassRole, Vst3ModuleMetadata,
};
use super::*;
use std::io;

impl Vst3HostAdapter {
    pub fn instantiate_plugin(
        &self,
        discovered: &Vst3DiscoveredPluginType,
        instance_id: &str,
    ) -> io::Result<Vst3InstanceControlSurface> {
        let (_, factory_classes, component, controller_name) =
            validate_discovered_bundle_contract(discovered)?;

        Ok(Vst3InstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            class_id: discovered.class_id.clone(),
            controller_class_id: discovered.controller_class_id.clone(),
            category: discovered.category.clone(),
            component_name: component.name.clone(),
            controller_name,
            factory_export_count: factory_classes.len(),
            module_root: discovered.module_root.clone(),
            default_io_layout: discovered.default_io_layout,
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        })
    }

    pub fn load_state_snapshot(
        &self,
        instance: &Vst3InstanceControlSurface,
        bytes: &[u8],
    ) -> io::Result<Vst3StateSnapshot> {
        if !instance.descriptor.state_contract.supports_snapshot {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "VST3 instance does not support state snapshots",
            ));
        }
        if bytes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VST3 state snapshot payload cannot be empty",
            ));
        }
        validate_instance_bundle_contract(instance)?;
        Ok(Vst3StateSnapshot {
            instance_id: instance.instance_id.clone(),
            bytes: bytes.to_vec(),
            digest: digest_bytes(bytes),
            summary: format!(
                "instance={} state_loaded={} digest={}",
                instance.instance_id.0,
                bytes.len(),
                digest_bytes(bytes),
            ),
        })
    }

    pub fn store_state_snapshot(
        &self,
        instance: &Vst3InstanceControlSurface,
    ) -> io::Result<Vst3StateSnapshot> {
        if !instance.descriptor.state_contract.supports_snapshot {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "VST3 instance does not support state snapshots",
            ));
        }
        validate_instance_bundle_contract(instance)?;
        let bytes = format!(
            "plugin_type={}\ninstance_id={}\ncomponent={}\ncontroller={}\nfactory_exports={}\n",
            instance.plugin_type_id.0,
            instance.instance_id.0,
            instance.component_name,
            instance.controller_name.as_deref().unwrap_or("none"),
            instance.factory_export_count,
        )
        .into_bytes();
        let digest = digest_bytes(&bytes);
        Ok(Vst3StateSnapshot {
            instance_id: instance.instance_id.clone(),
            bytes,
            digest: digest.clone(),
            summary: format!(
                "instance={} state_stored={} digest={}",
                instance.instance_id.0, instance.factory_export_count, digest,
            ),
        })
    }

    pub fn activate_instance(
        &self,
        instance: &Vst3InstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
        loaded_state: Option<&Vst3StateSnapshot>,
    ) -> io::Result<Vst3ActivationRecord> {
        if !instance.lifecycle_contract.supports_activate {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "VST3 instance does not support activation",
            ));
        }
        validate_instance_bundle_contract(instance)?;
        Ok(Vst3ActivationRecord {
            instance_id: instance.instance_id.clone(),
            component_name: instance.component_name.clone(),
            controller_name: instance.controller_name.clone(),
            sample_rate_hz,
            max_block_frames,
            state_bytes: loaded_state.map(|state| state.bytes.len()).unwrap_or(0),
            summary: format!(
                "instance={} component={} controller={} sample_rate={} max_block_frames={} state_bytes={}",
                instance.instance_id.0,
                instance.component_name,
                instance.controller_name.as_deref().unwrap_or("none"),
                sample_rate_hz,
                max_block_frames,
                loaded_state.map(|state| state.bytes.len()).unwrap_or(0),
            ),
        })
    }

    pub fn teardown_instance(
        &self,
        instance: &Vst3InstanceControlSurface,
        final_state: Option<&Vst3StateSnapshot>,
    ) -> io::Result<Vst3TeardownRecord> {
        validate_instance_bundle_contract(instance)?;
        Ok(Vst3TeardownRecord {
            instance_id: instance.instance_id.clone(),
            flushed_state_bytes: final_state.map(|state| state.bytes.len()).unwrap_or(0),
            summary: format!(
                "instance={} component={} flushed_state_bytes={}",
                instance.instance_id.0,
                instance.component_name,
                final_state.map(|state| state.bytes.len()).unwrap_or(0),
            ),
        })
    }

    pub fn prepare_session(
        &self,
        instance: &Vst3InstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> Vst3ProcessSessionPlan {
        Vst3ProcessSessionPlan {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            class_id: instance.class_id.clone(),
            controller_class_id: instance.controller_class_id.clone(),
            category: instance.category.clone(),
            component_name: instance.component_name.clone(),
            controller_name: instance.controller_name.clone(),
            factory_export_count: instance.factory_export_count,
            sample_rate_hz,
            max_block_frames,
            io_layout: instance.default_io_layout,
            transport: SandboxTransport::SharedMemory,
            module_root: instance.module_root.clone(),
            summary: format!(
                "plugin_type={} class={} controller={} component_name={} controller_name={} factory_exports={} sample_rate={} max_block_frames={} module_root={}",
                instance.plugin_type_id.0,
                instance.class_id,
                instance.controller_class_id.as_deref().unwrap_or("none"),
                instance.component_name,
                instance.controller_name.as_deref().unwrap_or("none"),
                instance.factory_export_count,
                sample_rate_hz,
                max_block_frames,
                instance.module_root,
            ),
        }
    }

    pub fn execute_block(
        &self,
        instance: &Vst3InstanceControlSurface,
        session: &Vst3ProcessSessionPlan,
        state: Option<&Vst3StateSnapshot>,
        block_sequence: u64,
        block_frames: u32,
        parameter_event_count: u16,
        midi_event_count: u16,
    ) -> io::Result<Vst3BlockProcessingRecord> {
        validate_instance_bundle_contract(instance)?;
        if block_frames == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "VST3 block execution requires non-zero block frames",
            ));
        }
        if block_frames > session.max_block_frames {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "VST3 block execution exceeded prepared max block frames: {} > {}",
                    block_frames, session.max_block_frames
                ),
            ));
        }
        if session.instance_id != instance.instance_id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VST3 block execution session instance id drifted from instantiated control surface",
            ));
        }

        let state_digest = state.map(|snapshot| snapshot.digest.clone());
        let parameter_signature = digest_bytes(
            format!(
                "instance={} block_sequence={} block_frames={} parameter_events={} midi_events={}",
                instance.instance_id.0,
                block_sequence,
                block_frames,
                parameter_event_count,
                midi_event_count,
            )
            .as_bytes(),
        );
        let next_state_digest = digest_bytes(
            format!(
                "previous_state={} parameter_signature={} completion={} component={}",
                state_digest.as_deref().unwrap_or("none"),
                parameter_signature,
                "completed",
                instance.component_name,
            )
            .as_bytes(),
        );
        let parameter_application_order = format!(
            "block{}:parameters({})->midi({})",
            block_sequence, parameter_event_count, midi_event_count
        );
        let event_packet_order = format!(
            "block{}[param_packets={},midi_packets={},apply=parameters_then_midi]",
            block_sequence, parameter_event_count, midi_event_count
        );
        let automation_delta = format!(
            "block{}:delta[param={},midi={},baseline={}]",
            block_sequence,
            parameter_event_count,
            midi_event_count,
            state_digest.as_deref().unwrap_or("none"),
        );
        let state_transition = format!(
            "state_transition=applied parameter_signature={} previous_state={} next_state={}",
            parameter_signature,
            state_digest.as_deref().unwrap_or("none"),
            next_state_digest,
        );
        Ok(Vst3BlockProcessingRecord {
            instance_id: instance.instance_id.clone(),
            block_sequence,
            block_frames,
            audio_input_channels: session.io_layout.audio_inputs,
            audio_output_channels: session.io_layout.audio_outputs,
            midi_input_ports: session.io_layout.midi_inputs,
            midi_output_ports: session.io_layout.midi_outputs,
            parameter_event_count,
            midi_event_count,
            completion_status: "completed",
            state_digest: state_digest.clone(),
            parameter_signature: parameter_signature.clone(),
            parameter_application_order: parameter_application_order.clone(),
            event_packet_order: event_packet_order.clone(),
            automation_delta: automation_delta.clone(),
            state_transition: state_transition.clone(),
            next_state_digest: next_state_digest.clone(),
            summary: format!(
                "instance={} component={} block_sequence={} block_frames={} audio_inputs={} audio_outputs={} midi_inputs={} midi_outputs={} parameter_events={} midi_events={} completion={} state_digest={} parameter_signature={} parameter_application_order={} event_packet_order={} automation_delta={} next_state_digest={} {}",
                instance.instance_id.0,
                instance.component_name,
                block_sequence,
                block_frames,
                session.io_layout.audio_inputs,
                session.io_layout.audio_outputs,
                session.io_layout.midi_inputs,
                session.io_layout.midi_outputs,
                parameter_event_count,
                midi_event_count,
                "completed",
                state_digest.as_deref().unwrap_or("none"),
                parameter_signature,
                parameter_application_order,
                event_packet_order,
                automation_delta,
                next_state_digest,
                state_transition,
            ),
        })
    }
}

fn validate_discovered_bundle_contract(
    discovered: &Vst3DiscoveredPluginType,
) -> io::Result<(
    Vst3ModuleMetadata,
    Vec<Vst3FactoryClass>,
    Vst3FactoryClass,
    Option<String>,
)> {
    let bundle_root = std::path::Path::new(&discovered.module_root);
    let snapshot = read_vst3_bundle_snapshot(bundle_root, inferred_platform(bundle_root))?;
    let factory_classes = snapshot.factory_classes;
    let module_metadata = snapshot
        .plugins
        .into_iter()
        .find(|metadata| {
            metadata.plugin_type_id == discovered.plugin_type_id.0
                && metadata.class_id == discovered.class_id
                && metadata.controller_class_id == discovered.controller_class_id
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "VST3 instantiation metadata no longer matches discovered plugin identity",
            )
        })?;

    let component = factory_classes
        .iter()
        .find(|class| {
            class.role == Vst3FactoryClassRole::Component
                && class.class_id == discovered.class_id
                && class.category == discovered.category
                && class.name == discovered.descriptor.name
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "VST3 component class missing from factory manifest during instantiation",
            )
        })?
        .clone();

    let controller_name = match discovered.controller_class_id.as_deref() {
        Some(controller_class_id) => Some(
            factory_classes
                .iter()
                .find(|class| {
                    class.role == Vst3FactoryClassRole::Controller
                        && class.class_id == controller_class_id
                })
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "VST3 controller class missing from factory manifest during instantiation",
                    )
                })?
                .name
                .clone(),
        ),
        None => None,
    };

    Ok((module_metadata, factory_classes, component, controller_name))
}

fn validate_instance_bundle_contract(instance: &Vst3InstanceControlSurface) -> io::Result<()> {
    let discovered = Vst3DiscoveredPluginType {
        plugin_type_id: instance.plugin_type_id.clone(),
        class_id: instance.class_id.clone(),
        controller_class_id: instance.controller_class_id.clone(),
        category: instance.category.clone(),
        module_root: instance.module_root.clone(),
        descriptor: instance.descriptor.clone(),
        default_io_layout: instance.default_io_layout,
    };
    let _ = validate_discovered_bundle_contract(&discovered)?;
    Ok(())
}

fn inferred_platform(bundle_root: &std::path::Path) -> Vst3HostPlatform {
    let contents = bundle_root.join("Contents");
    if contents.join("MacOS").exists() {
        Vst3HostPlatform::MacOs
    } else if contents.join("x86_64-linux").exists() || contents.join("aarch64-linux").exists() {
        Vst3HostPlatform::Linux
    } else if contents.join("x86_64-win").exists() || contents.join("arm64-win").exists() {
        Vst3HostPlatform::Windows
    } else {
        Vst3HostPlatform::MacOs
    }
}

fn digest_bytes(bytes: &[u8]) -> String {
    let checksum = bytes.iter().fold(0u64, |acc, byte| {
        acc.wrapping_mul(131).wrapping_add(*byte as u64)
    });
    format!("{checksum:016x}")
}
