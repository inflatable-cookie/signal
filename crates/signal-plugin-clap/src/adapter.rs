use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use signal_plugin::{
    BlockDispatch, BlockProcessResult, BlockProcessingHeader, PluginDescriptor, PluginFormat,
    PluginInstanceId, PluginIoLayout, PluginLifecycleContract, PluginProcessingContract,
    PluginSandboxCapabilities, PluginTypeId, SandboxTransport, SharedMemoryLayout,
    SharedMemoryLease,
};

use crate::{clap_sandbox_harness, discovery::discover_clap_plugins_for_roots};

/// A CLAP host extension that the Signal host advertises to plugins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapHostExtension {
    /// The `clap.audio-ports` extension for audio bus negotiation.
    AudioPorts,
    /// The `clap.note-ports` extension for note port negotiation.
    NotePorts,
    /// The `clap.params` extension for parameter access.
    Params,
    /// The `clap.state` extension for state save/restore.
    State,
    /// The `clap.latency` extension for latency reporting.
    Latency,
    /// The `clap.tail` extension for tail-length reporting.
    Tail,
}

impl ClapHostExtension {
    /// Returns the CLAP extension identifier string for this extension.
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

/// Host-side adapter for CLAP plugins. Handles discovery, instantiation, and capability negotiation.
#[derive(Clone, Debug)]
pub struct ClapPluginHostAdapter {
    strict_sandbox_default: bool,
    discovery_catalog: Arc<Mutex<HashMap<String, ClapDiscoveredPluginType>>>,
}

impl Default for ClapPluginHostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
            discovery_catalog: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ClapPluginHostAdapter {
    /// Returns whether strict sandboxing is the default policy for this adapter.
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    /// Returns `true` if the given plugin format is CLAP.
    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Clap)
    }

    /// Returns the minimum set of CLAP host extensions that must be supported.
    pub fn minimum_extension_set(&self) -> &'static [ClapHostExtension] {
        &MINIMUM_CLAP_EXTENSIONS
    }

    /// Builds the sandbox capabilities advertised to CLAP plugins for the given maximum block size.
    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }

    /// Looks up a previously discovered plugin type by its type ID string.
    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<ClapDiscoveredPluginType> {
        if let Some(discovered) = self
            .discovery_catalog
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(plugin_type_id)
            .cloned()
        {
            return Some(discovered);
        }
        clap_sandbox_harness::clap_discovered_plugin_type(plugin_type_id)
    }

    /// Scans the given filesystem roots for CLAP plugins and updates the internal catalog.
    pub fn discover_plugins_for_roots(&self, roots: &[String]) -> Vec<ClapDiscoveredPluginType> {
        let discovered = discover_clap_plugins_for_roots(roots);
        let mut catalog = self
            .discovery_catalog
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        catalog.clear();
        catalog.extend(
            discovered
                .iter()
                .cloned()
                .map(|plugin| (plugin.plugin_type_id.0.clone(), plugin)),
        );
        discovered
    }

    /// Creates a [`ClapInstanceControlSurface`] binding the discovered plugin type to a new instance ID.
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

/// A CLAP plugin type discovered during a scan, including its descriptor and default I/O layout.
#[derive(Clone, Debug, PartialEq)]
pub struct ClapDiscoveredPluginType {
    /// Unique identifier for this plugin type.
    pub plugin_type_id: PluginTypeId,
    /// Path to the CLAP shared library on disk.
    pub library_path: String,
    /// Full plugin descriptor including contracts and metadata.
    pub descriptor: PluginDescriptor,
    /// Default I/O layout reported by the plugin at scan time.
    pub default_io_layout: PluginIoLayout,
}

/// Live control surface for a CLAP plugin instance. Binds a discovered plugin type to a running instance ID and its contracts.
#[derive(Clone, Debug, PartialEq)]
pub struct ClapInstanceControlSurface {
    /// Unique identifier for the plugin type this instance was created from.
    pub plugin_type_id: PluginTypeId,
    /// Unique identifier for this running instance.
    pub instance_id: PluginInstanceId,
    /// Plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Lifecycle contract for this instance.
    pub lifecycle_contract: PluginLifecycleContract,
    /// Processing contract for this instance.
    pub processing_contract: PluginProcessingContract,
}

/// Shared-memory block header written before each audio block dispatch to a CLAP sandbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSharedMemoryHeader {
    /// Plugin type this block targets.
    pub plugin_type_id: PluginTypeId,
    /// Instance this block targets.
    pub instance_id: PluginInstanceId,
    /// Processing header for the block.
    pub block: BlockProcessingHeader,
    /// Shared-memory region layout for the block.
    pub layout: SharedMemoryLayout,
}

/// Parameters agreed upon during plugin prepare, used to size and allocate the shared-memory lease.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapPreparePlan {
    /// Plugin type being prepared.
    pub plugin_type_id: PluginTypeId,
    /// Instance being prepared.
    pub instance_id: PluginInstanceId,
    /// Sample rate negotiated for this session.
    pub sample_rate_hz: u32,
    /// Maximum audio block size in frames.
    pub max_block_frames: u32,
    /// I/O layout for this session.
    pub io_layout: PluginIoLayout,
    /// Allocated shared-memory lease for this session.
    pub lease: SharedMemoryLease,
}

/// Outcome of a brokered block round-trip: the dispatch, the input and output payloads, and the process result.
#[derive(Clone, Debug, PartialEq)]
pub struct BrokeredBlockOutcome {
    /// The block dispatch that was sent.
    pub dispatch: BlockDispatch,
    /// Input payload written into the shared-memory region.
    pub input: signal_plugin::BlockPayload,
    /// Output payload read back from the shared-memory region.
    pub output: signal_plugin::BlockPayload,
    /// Processing result reported by the sandbox.
    pub result: BlockProcessResult,
}
