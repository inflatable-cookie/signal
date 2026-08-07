//! Broker wire types, receipt rendering, and command parsing.

use signal_plugin::PluginParameterDescriptor;

pub(crate) const RUN_BLOCK_COUNT: u64 = 8;
/// Size of the shared-memory region allocated at attach time.
pub(crate) const REGION_BYTES: u32 = 64 * 1024;
/// Spin iterations between `yield_now` calls on the audio thread's wait.
pub(crate) const AUDIO_SPIN_PER_YIELD: u32 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxBrokerState {
    Starting,
    Ready,
    Attached,
    Running,
    TimedOut,
    TeardownComplete,
    Crashed,
    Shutdown,
    PluginLoaded,
    PluginActivated,
    LayoutUnsupported,
    ProcessingStarted,
    ProcessingStopped,
    PluginDeactivated,
    PluginUnloaded,
    ParamSet,
    EditorOpened,
    EditorClosed,
}

impl SandboxBrokerState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Attached => "attached",
            Self::Running => "running",
            Self::TimedOut => "timed_out",
            Self::TeardownComplete => "teardown_complete",
            Self::Crashed => "crashed",
            Self::Shutdown => "shutdown",
            Self::PluginLoaded => "plugin_loaded",
            Self::PluginActivated => "plugin_activated",
            Self::LayoutUnsupported => "layout_unsupported",
            Self::ProcessingStarted => "processing_started",
            Self::ProcessingStopped => "processing_stopped",
            Self::PluginDeactivated => "plugin_deactivated",
            Self::PluginUnloaded => "plugin_unloaded",
            Self::ParamSet => "param_set",
            Self::EditorOpened => "editor_opened",
            Self::EditorClosed => "editor_closed",
        }
    }
}

/// Percent-encode a token value so it survives the whitespace-separated
/// `key=value` wire format (encodes `%`, space, `=`, `:`, `;`, `|`, and
/// control characters).
pub fn encode_wire_token(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'%' | b' ' | b'=' | b':' | b';' | b'|' => {
                encoded.push('%');
                encoded.push_str(&format!("{byte:02X}"));
            }
            byte if byte.is_ascii_control() => {
                encoded.push('%');
                encoded.push_str(&format!("{byte:02X}"));
            }
            byte => encoded.push(byte as char),
        }
    }
    encoded
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerReceipt {
    pub state: SandboxBrokerState,
    pub sandbox_id: String,
    pub instance_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub lease_id: Option<String>,
    pub region_id: Option<String>,
    /// Extra structured `key=value` tokens (parameter inventory, shm
    /// coordinates); values are already wire-encoded.
    pub extra: Vec<(String, String)>,
    pub detail: String,
}

impl SandboxBrokerReceipt {
    pub fn render_line(&self) -> String {
        let mut line = format!(
            "signal-plugin-sandbox state={} sandbox_id={} instance_id={} epoch={} lease_id={} region_id={}",
            self.state.as_str(),
            self.sandbox_id,
            self.instance_id.as_deref().unwrap_or("-"),
            self.processing_epoch
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into()),
            self.lease_id.as_deref().unwrap_or("-"),
            self.region_id.as_deref().unwrap_or("-"),
        );
        for (key, value) in &self.extra {
            line.push(' ');
            line.push_str(key);
            line.push('=');
            line.push_str(value);
        }
        line.push_str(" detail=");
        line.push_str(&self.detail.replace(' ', "_"));
        line
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SandboxBrokerCommand {
    Status,
    Attach,
    Run,
    RunTimeout,
    Teardown,
    Shutdown,
    LoadPlugin {
        library_path: String,
        plugin_id: String,
    },
    ActivatePlugin {
        sample_rate_hz: f64,
        min_frames: u32,
        max_frames: u32,
    },
    StartProcessing,
    StopProcessing,
    DeactivatePlugin,
    UnloadPlugin,
    /// One or more `(parameter_id, normalized 0..1)` writes (g12.023):
    /// `set-param <id> <normalized>` or the batched
    /// `set-params <id:normalized[;id:normalized...]>`.
    SetParameters {
        changes: Vec<(u32, f32)>,
    },
    /// Open the child-owned editor window for the loaded plugin (g13.027):
    /// `open-editor <instance>` — `instance` is the parent's opaque editor
    /// token (window title; echoed in receipts).
    OpenEditor {
        instance: String,
    },
    /// Close the child-owned editor window: `close-editor <instance>`.
    CloseEditor {
        instance: String,
    },
}

impl SandboxBrokerCommand {
    pub(crate) fn parse(line: &str) -> Result<Self, String> {
        let mut tokens = line.split_whitespace();
        let command = tokens.next().unwrap_or_default();
        match command {
            "status" => Ok(Self::Status),
            "attach" => Ok(Self::Attach),
            "run" => Ok(Self::Run),
            "run-timeout" => Ok(Self::RunTimeout),
            "teardown" => Ok(Self::Teardown),
            "shutdown" => Ok(Self::Shutdown),
            "load-plugin" => {
                let library_path = tokens
                    .next()
                    .ok_or_else(|| "load_plugin_missing_library_path".to_string())?;
                let plugin_id = tokens
                    .next()
                    .ok_or_else(|| "load_plugin_missing_plugin_id".to_string())?;
                Ok(Self::LoadPlugin {
                    library_path: library_path.to_string(),
                    plugin_id: plugin_id.to_string(),
                })
            }
            "activate" => {
                let sample_rate_hz = tokens
                    .next()
                    .and_then(|token| token.parse::<f64>().ok())
                    .ok_or_else(|| "activate_missing_sample_rate".to_string())?;
                let min_frames = tokens
                    .next()
                    .and_then(|token| token.parse::<u32>().ok())
                    .ok_or_else(|| "activate_missing_min_frames".to_string())?;
                let max_frames = tokens
                    .next()
                    .and_then(|token| token.parse::<u32>().ok())
                    .ok_or_else(|| "activate_missing_max_frames".to_string())?;
                Ok(Self::ActivatePlugin {
                    sample_rate_hz,
                    min_frames,
                    max_frames,
                })
            }
            "set-param" => {
                let parameter_id = tokens
                    .next()
                    .and_then(|token| token.parse::<u32>().ok())
                    .ok_or_else(|| "set_param_missing_parameter_id".to_string())?;
                let normalized = tokens
                    .next()
                    .and_then(|token| token.parse::<f32>().ok())
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| "set_param_missing_value".to_string())?;
                Ok(Self::SetParameters {
                    changes: vec![(parameter_id, normalized)],
                })
            }
            "set-params" => {
                let blob = tokens
                    .next()
                    .ok_or_else(|| "set_params_missing_changes".to_string())?;
                let mut changes = Vec::new();
                for entry in blob.split(';').filter(|entry| !entry.is_empty()) {
                    let (id, value) = entry
                        .split_once(':')
                        .ok_or_else(|| "set_params_malformed_entry".to_string())?;
                    let parameter_id = id
                        .parse::<u32>()
                        .map_err(|_| "set_params_malformed_entry".to_string())?;
                    let normalized = value
                        .parse::<f32>()
                        .ok()
                        .filter(|value| value.is_finite())
                        .ok_or_else(|| "set_params_malformed_entry".to_string())?;
                    changes.push((parameter_id, normalized));
                }
                if changes.is_empty() {
                    return Err("set_params_missing_changes".to_string());
                }
                Ok(Self::SetParameters { changes })
            }
            "open-editor" => {
                let instance = tokens
                    .next()
                    .ok_or_else(|| "open_editor_missing_instance".to_string())?;
                Ok(Self::OpenEditor {
                    instance: instance.to_string(),
                })
            }
            "close-editor" => {
                let instance = tokens
                    .next()
                    .ok_or_else(|| "close_editor_missing_instance".to_string())?;
                Ok(Self::CloseEditor {
                    instance: instance.to_string(),
                })
            }
            "start-processing" => Ok(Self::StartProcessing),
            "stop-processing" => Ok(Self::StopProcessing),
            "deactivate" => Ok(Self::DeactivatePlugin),
            "unload-plugin" => Ok(Self::UnloadPlugin),
            other => Err(format!("unknown_command:{other}")),
        }
    }
}

pub(crate) fn encode_parameter_inventory(parameters: &[PluginParameterDescriptor]) -> String {
    parameters
        .iter()
        .map(|parameter| {
            let mut flags = String::new();
            if parameter.is_automatable() {
                flags.push('a');
            }
            if parameter.is_bypass() {
                flags.push('b');
            }
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}",
                parameter.parameter_id,
                encode_wire_token(&parameter.name),
                parameter.min_plain,
                parameter.max_plain,
                parameter.default_normalized,
                parameter
                    .unit
                    .as_deref()
                    .map(encode_wire_token)
                    .unwrap_or_default(),
                parameter
                    .step_count
                    .map(|steps| steps.to_string())
                    .unwrap_or_default(),
                flags,
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
