use super::{
    PluginAudioBusDescriptor, PluginAudioBusDirection, PluginDescriptor, PluginFeature,
    PluginFormat, PluginIoLayout, PluginLifecycleContract, PluginParameterDescriptor,
    PluginProcessingContract, PluginStateContract,
};

impl PluginIoLayout {
    /// Returns the maximum of `audio_inputs` and `audio_outputs`, representing the channel
    /// budget needed to hold either direction's audio in a single buffer.
    pub fn audio_channels(self) -> u16 {
        self.audio_inputs.max(self.audio_outputs)
    }

    /// Returns the maximum of `midi_inputs` and `midi_outputs`.
    pub fn midi_ports(self) -> u16 {
        self.midi_inputs.max(self.midi_outputs)
    }

    /// Builds the main-input and main-output [`PluginAudioBusDescriptor`]s implied by this layout.
    pub fn main_audio_buses(self) -> Vec<PluginAudioBusDescriptor> {
        let mut buses = Vec::new();
        if self.audio_inputs > 0 {
            buses.push(PluginAudioBusDescriptor {
                bus_id: "audio:main-in".into(),
                name: "Main Input".into(),
                direction: PluginAudioBusDirection::Input,
                channels: self.audio_inputs,
                is_main: true,
                active: true,
            });
        }
        if self.audio_outputs > 0 {
            buses.push(PluginAudioBusDescriptor {
                bus_id: "audio:main-out".into(),
                name: "Main Output".into(),
                direction: PluginAudioBusDirection::Output,
                channels: self.audio_outputs,
                is_main: true,
                active: true,
            });
        }
        buses
    }
}

impl PluginDescriptor {
    /// Creates a minimal descriptor with all contracts set to conservative defaults.
    pub fn new(
        plugin_id: impl Into<String>,
        vendor: impl Into<String>,
        name: impl Into<String>,
        format: PluginFormat,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            vendor: vendor.into(),
            name: name.into(),
            format,
            version: None,
            features: Vec::new(),
            audio_buses: Vec::new(),
            parameters: Vec::new(),
            state_contract: PluginStateContract {
                supports_snapshot: false,
                supports_reset: false,
                supports_bypass: false,
                exposes_latency: false,
                exposes_tail: false,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 0,
                sample_accurate_automation: false,
                accepts_midi: false,
                accepts_note_events: false,
                supports_note_expression: false,
                produces_midi: false,
                silence_aware: false,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: false,
            },
        }
    }

    /// Sets the version string on this descriptor.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Appends a feature to the descriptor's feature list.
    pub fn with_feature(mut self, feature: PluginFeature) -> Self {
        self.features.push(feature);
        self
    }

    /// Replaces the audio bus list on this descriptor.
    pub fn with_audio_buses(mut self, audio_buses: Vec<PluginAudioBusDescriptor>) -> Self {
        self.audio_buses = audio_buses;
        self
    }

    /// Replaces the parameter list on this descriptor.
    pub fn with_parameters(mut self, parameters: Vec<PluginParameterDescriptor>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Replaces the state contract on this descriptor.
    pub fn with_state_contract(mut self, state_contract: PluginStateContract) -> Self {
        self.state_contract = state_contract;
        self
    }

    /// Replaces the processing contract on this descriptor.
    pub fn with_processing_contract(
        mut self,
        processing_contract: PluginProcessingContract,
    ) -> Self {
        self.processing_contract = processing_contract;
        self
    }

    /// Replaces the lifecycle contract on this descriptor.
    pub fn with_lifecycle_contract(mut self, lifecycle_contract: PluginLifecycleContract) -> Self {
        self.lifecycle_contract = lifecycle_contract;
        self
    }
}
