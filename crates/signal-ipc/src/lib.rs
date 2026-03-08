//! Shared runtime control and message model for Signal.

use std::{
    fs::{self, File, OpenOptions},
    io,
    path::PathBuf,
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use memmap2::{MmapMut, MmapOptions};

static PROCESS_WIDE_REGION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDomain {
    Engine,
    Graph,
    Hardware,
    Plugin,
    Diagnostics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    Command,
    Event,
    Response,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMessage {
    pub domain: RuntimeDomain,
    pub kind: MessageKind,
    pub name: String,
    pub correlation_id: Option<CorrelationId>,
}

impl RuntimeMessage {
    pub fn command(domain: RuntimeDomain, name: impl Into<String>) -> Self {
        Self {
            domain,
            kind: MessageKind::Command,
            name: name.into(),
            correlation_id: None,
        }
    }

    pub fn response(
        domain: RuntimeDomain,
        name: impl Into<String>,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            domain,
            kind: MessageKind::Response,
            name: name.into(),
            correlation_id: Some(correlation_id),
        }
    }

    pub fn event(
        domain: RuntimeDomain,
        name: impl Into<String>,
        correlation_id: Option<CorrelationId>,
    ) -> Self {
        Self {
            domain,
            kind: MessageKind::Event,
            name: name.into(),
            correlation_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginMessageName {
    SandboxHandshake,
    SandboxLoadPluginType,
    SandboxCreateInstance,
    SandboxPrepareInstance,
    SandboxActivateInstance,
    SandboxHeartbeat,
    SandboxDeactivateInstance,
    SandboxResetInstance,
    SandboxDestroyInstance,
    SandboxFailure,
}

impl PluginMessageName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SandboxHandshake => "sandbox.handshake",
            Self::SandboxLoadPluginType => "sandbox.loadPluginType",
            Self::SandboxCreateInstance => "sandbox.createInstance",
            Self::SandboxPrepareInstance => "sandbox.prepareInstance",
            Self::SandboxActivateInstance => "sandbox.activateInstance",
            Self::SandboxHeartbeat => "sandbox.heartbeat",
            Self::SandboxDeactivateInstance => "sandbox.deactivateInstance",
            Self::SandboxResetInstance => "sandbox.resetInstance",
            Self::SandboxDestroyInstance => "sandbox.destroyInstance",
            Self::SandboxFailure => "sandbox.failure",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDescriptorPayload {
    pub plugin_id: String,
    pub vendor: String,
    pub name: String,
    pub format: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginIoLayoutPayload {
    pub audio_inputs: u16,
    pub audio_outputs: u16,
    pub midi_inputs: u16,
    pub midi_outputs: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryRegionPayload {
    pub offset_bytes: u32,
    pub size_bytes: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedMemoryTransportKind {
    MappedFile,
}

impl SharedMemoryTransportKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MappedFile => "mappedFile",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedMemoryTransportPayload {
    pub region_id: String,
    pub transport_kind: SharedMemoryTransportKind,
    pub backing_path: String,
    pub total_bytes: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryLayoutPayload {
    pub audio_input: SharedMemoryRegionPayload,
    pub audio_output: SharedMemoryRegionPayload,
    pub event_input: SharedMemoryRegionPayload,
    pub event_output: SharedMemoryRegionPayload,
    pub render_context: SharedMemoryRegionPayload,
    pub completion: SharedMemoryRegionPayload,
}

impl SharedMemoryLayoutPayload {
    pub fn total_bytes(self) -> u32 {
        self.completion.offset_bytes + self.completion.size_bytes
    }
}

pub struct MappedSharedMemoryRegion {
    metadata: SharedMemoryTransportPayload,
    file: File,
    map: MmapMut,
}

impl core::fmt::Debug for MappedSharedMemoryRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MappedSharedMemoryRegion")
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl MappedSharedMemoryRegion {
    pub fn metadata(&self) -> &SharedMemoryTransportPayload {
        &self.metadata
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.map
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.map
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.map.flush()
    }

    pub fn total_bytes(&self) -> u32 {
        self.metadata.total_bytes
    }

    pub fn file_len(&self) -> io::Result<u64> {
        self.file.metadata().map(|metadata| metadata.len())
    }
}

#[derive(Debug)]
pub struct SharedMemoryBroker {
    root: PathBuf,
    next_region: AtomicU64,
}

impl Default for SharedMemoryBroker {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("signal-shared-memory"))
    }
}

impl SharedMemoryBroker {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            next_region: AtomicU64::new(1),
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn create_region(
        &self,
        lease_id: &str,
        total_bytes: u32,
    ) -> io::Result<MappedSharedMemoryRegion> {
        fs::create_dir_all(&self.root)?;

        let sequence = self.next_region.fetch_add(1, Ordering::Relaxed);
        let process_wide_sequence = PROCESS_WIDE_REGION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let region_id = format!(
            "region-{}-{timestamp}-{sequence}-{process_wide_sequence}",
            process::id()
        );
        let filename = format!("{}-{region_id}.signal-shm", sanitize_identifier(lease_id));
        let path = self.root.join(filename);
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&path)?;
        file.set_len(total_bytes as u64)?;
        let mut map = map_file(&file, total_bytes as usize)?;
        map.fill(0);

        Ok(MappedSharedMemoryRegion {
            metadata: SharedMemoryTransportPayload {
                region_id,
                transport_kind: SharedMemoryTransportKind::MappedFile,
                backing_path: path.display().to_string(),
                total_bytes,
            },
            file,
            map,
        })
    }

    pub fn attach_region(
        &self,
        transport: &SharedMemoryTransportPayload,
    ) -> io::Result<MappedSharedMemoryRegion> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&transport.backing_path)?;
        let file_len = file.metadata()?.len();
        if file_len < transport.total_bytes as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "shared-memory region {} expected {} bytes but file only has {}",
                    transport.region_id, transport.total_bytes, file_len
                ),
            ));
        }

        let map = map_file(&file, transport.total_bytes as usize)?;
        Ok(MappedSharedMemoryRegion {
            metadata: transport.clone(),
            file,
            map,
        })
    }

    pub fn destroy_region(&self, transport: &SharedMemoryTransportPayload) -> io::Result<()> {
        match fs::remove_file(&transport.backing_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

fn map_file(file: &File, len: usize) -> io::Result<MmapMut> {
    // SAFETY: the file is explicitly sized by the broker before mapping, and
    // callers pass the exact byte length they expect to use.
    unsafe { MmapOptions::new().len(len).map_mut(file) }
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginMessagePayload {
    SandboxHandshakeRequest {
        sandbox_id: String,
        format: String,
    },
    SandboxHandshakeResponse {
        sandbox_id: String,
        protocol_version: u32,
        supports_state: bool,
        supports_midi: bool,
        max_block_frames: u32,
    },
    LoadPluginTypeRequest {
        sandbox_id: String,
        plugin_type_id: String,
        descriptor: PluginDescriptorPayload,
    },
    LoadPluginTypeResponse {
        plugin_type_id: String,
    },
    CreateInstanceRequest {
        sandbox_id: String,
        plugin_type_id: String,
        instance_id: String,
    },
    CreateInstanceResponse {
        instance_id: String,
    },
    PrepareInstanceRequest {
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
        shared_memory_lease_id: String,
        shared_memory_transport: SharedMemoryTransportPayload,
        sample_rate_hz: u32,
        max_block_frames: u32,
        io_layout: PluginIoLayoutPayload,
        shared_memory: SharedMemoryLayoutPayload,
    },
    PrepareInstanceResponse {
        instance_id: String,
        processing_epoch: u64,
        shared_memory_lease_id: String,
        shared_memory_transport: SharedMemoryTransportPayload,
        shared_memory_bytes: u32,
    },
    ActivateInstanceRequest {
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
    },
    ActivateInstanceResponse {
        instance_id: String,
        processing_epoch: u64,
    },
    HeartbeatRequest {
        sandbox_id: String,
        instance_id: Option<String>,
        processing_epoch: Option<u64>,
    },
    HeartbeatResponse {
        sandbox_id: String,
        instance_id: Option<String>,
        processing_epoch: Option<u64>,
        active: bool,
    },
    DeactivateInstanceRequest {
        sandbox_id: String,
        instance_id: String,
    },
    DeactivateInstanceResponse {
        instance_id: String,
    },
    ResetInstanceRequest {
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
    },
    ResetInstanceResponse {
        instance_id: String,
        processing_epoch: u64,
    },
    DestroyInstanceRequest {
        sandbox_id: String,
        instance_id: String,
    },
    DestroyInstanceResponse {
        instance_id: String,
    },
    SandboxFailure {
        sandbox_id: String,
        instance_id: Option<String>,
        stage: String,
        error_kind: String,
        detail: String,
        processing_epoch: Option<u64>,
        shared_memory_lease_id: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginMessageEnvelope {
    pub message: RuntimeMessage,
    pub payload: PluginMessagePayload,
}

impl PluginMessageEnvelope {
    pub fn command(
        name: PluginMessageName,
        correlation_id: impl Into<String>,
        payload: PluginMessagePayload,
    ) -> Self {
        Self {
            message: RuntimeMessage {
                domain: RuntimeDomain::Plugin,
                kind: MessageKind::Command,
                name: name.as_str().into(),
                correlation_id: Some(CorrelationId::new(correlation_id)),
            },
            payload,
        }
    }

    pub fn response(
        name: PluginMessageName,
        correlation_id: CorrelationId,
        payload: PluginMessagePayload,
    ) -> Self {
        Self {
            message: RuntimeMessage::response(RuntimeDomain::Plugin, name.as_str(), correlation_id),
            payload,
        }
    }

    pub fn event(
        name: PluginMessageName,
        correlation_id: Option<CorrelationId>,
        payload: PluginMessagePayload,
    ) -> Self {
        Self {
            message: RuntimeMessage::event(RuntimeDomain::Plugin, name.as_str(), correlation_id),
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PluginMessageEnvelope, PluginMessageName, PluginMessagePayload, RuntimeDomain,
        SharedMemoryBroker, SharedMemoryTransportKind, SharedMemoryTransportPayload,
    };
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
            "signal-ipc-tests-{}-{name}-{timestamp}",
            process::id()
        ))
    }

    #[test]
    fn plugin_command_envelope_uses_plugin_domain() {
        let envelope = PluginMessageEnvelope::command(
            PluginMessageName::SandboxHandshake,
            "cid-1",
            PluginMessagePayload::SandboxHandshakeRequest {
                sandbox_id: "sandbox-a".into(),
                format: "clap".into(),
            },
        );

        assert_eq!(envelope.message.domain, RuntimeDomain::Plugin);
        assert_eq!(envelope.message.name, "sandbox.handshake");
    }

    #[test]
    fn plugin_response_envelope_preserves_correlation() {
        let request = PluginMessageEnvelope::command(
            PluginMessageName::SandboxActivateInstance,
            "cid-activate-1",
            PluginMessagePayload::ActivateInstanceRequest {
                sandbox_id: "sandbox-a".into(),
                instance_id: "instance-a".into(),
                processing_epoch: 1,
            },
        );
        let response = PluginMessageEnvelope::response(
            PluginMessageName::SandboxActivateInstance,
            request
                .message
                .correlation_id
                .clone()
                .expect("request correlation"),
            PluginMessagePayload::ActivateInstanceResponse {
                instance_id: "instance-a".into(),
                processing_epoch: 1,
            },
        );

        assert_eq!(
            response.message.correlation_id,
            request.message.correlation_id
        );
        assert_eq!(response.message.name, "sandbox.activateInstance");
    }

    #[test]
    fn heartbeat_envelope_uses_heartbeat_message_name() {
        let envelope = PluginMessageEnvelope::command(
            PluginMessageName::SandboxHeartbeat,
            "cid-heartbeat-1",
            PluginMessagePayload::HeartbeatRequest {
                sandbox_id: "sandbox-a".into(),
                instance_id: Some("instance-a".into()),
                processing_epoch: Some(7),
            },
        );

        assert_eq!(envelope.message.name, "sandbox.heartbeat");
        assert_eq!(envelope.message.domain, RuntimeDomain::Plugin);
    }

    #[test]
    fn shared_memory_broker_round_trips_bytes_across_attachment() {
        let root = test_broker_root("roundtrip");
        let broker = SharedMemoryBroker::new(&root);
        let mut region = broker
            .create_region("lease-roundtrip", 256)
            .expect("create brokered region");

        region.as_mut_slice()[0..4].copy_from_slice(&[1, 2, 3, 4]);
        region.flush().expect("flush brokered region");

        let transport = region.metadata().clone();
        let attached = broker
            .attach_region(&transport)
            .expect("attach brokered region");

        assert_eq!(&attached.as_slice()[0..4], &[1, 2, 3, 4]);
        assert_eq!(attached.file_len().expect("file len"), 256);
        assert_eq!(
            transport.transport_kind,
            SharedMemoryTransportKind::MappedFile
        );

        broker.destroy_region(&transport).expect("destroy region");
        assert!(!std::path::Path::new(&transport.backing_path).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_payload_can_carry_transport_metadata() {
        let payload = PluginMessagePayload::PrepareInstanceRequest {
            sandbox_id: "sandbox-a".into(),
            instance_id: "instance-a".into(),
            processing_epoch: 2,
            shared_memory_lease_id: "lease-a".into(),
            shared_memory_transport: SharedMemoryTransportPayload {
                region_id: "region-a".into(),
                transport_kind: SharedMemoryTransportKind::MappedFile,
                backing_path: "/tmp/region-a.signal-shm".into(),
                total_bytes: 1024,
            },
            sample_rate_hz: 48_000,
            max_block_frames: 512,
            io_layout: super::PluginIoLayoutPayload {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            shared_memory: super::SharedMemoryLayoutPayload {
                audio_input: super::SharedMemoryRegionPayload {
                    offset_bytes: 0,
                    size_bytes: 256,
                },
                audio_output: super::SharedMemoryRegionPayload {
                    offset_bytes: 256,
                    size_bytes: 256,
                },
                event_input: super::SharedMemoryRegionPayload {
                    offset_bytes: 512,
                    size_bytes: 64,
                },
                event_output: super::SharedMemoryRegionPayload {
                    offset_bytes: 576,
                    size_bytes: 64,
                },
                render_context: super::SharedMemoryRegionPayload {
                    offset_bytes: 640,
                    size_bytes: 256,
                },
                completion: super::SharedMemoryRegionPayload {
                    offset_bytes: 896,
                    size_bytes: 64,
                },
            },
        };

        match payload {
            PluginMessagePayload::PrepareInstanceRequest {
                shared_memory_transport,
                ..
            } => {
                assert_eq!(shared_memory_transport.region_id, "region-a");
                assert_eq!(shared_memory_transport.total_bytes, 1024);
            }
            other => panic!("expected prepare payload, got {other:?}"),
        }
    }
}
