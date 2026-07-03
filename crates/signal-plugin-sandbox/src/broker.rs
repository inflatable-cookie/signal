//! Out-of-process sandbox broker.
//!
//! This binary is the plugin-hosting child process: it exercises the
//! plumbing that out-of-process hosting needs — child-process spawn, a
//! line-oriented stdio control protocol, file-backed shared-memory block
//! transport — and REAL plugin instance hosting (CLAP per g11.012, VST3 per
//! g11.031): `load-plugin` selects the format by library path extension
//! (`.clap` → `signal-plugin-clap`'s hosting FFI, `.vst3` →
//! `signal-plugin-vst3`'s COM FFI; second argument = format-native load
//! key), `activate` activates the instance and leases a shared-memory audio
//! block region, and `start-processing` spawns the child's audio thread,
//! which spin/yield-waits on the region's request stamp and runs the
//! plugin's `process()` for every block the parent posts.
//!
//! Wire format: one receipt per line,
//! `signal-plugin-sandbox state=<token> sandbox_id=... instance_id=... epoch=... lease_id=... region_id=... [key=value ...] detail=...`
//! with the legacy states `starting`, `ready`, `attached`, `running`,
//! `timed_out`, `crashed`, `teardown_complete`, `shutdown` plus the plugin
//! lifecycle states `plugin_loaded`, `plugin_activated`,
//! `layout_unsupported`, `processing_started`, `processing_stopped`,
//! `plugin_deactivated`, `plugin_unloaded`. Extra `key=value` tokens carry
//! structured payloads (parameter inventory, shm coordinates); values are
//! percent-encoded where they may contain spaces or separators.
//!
//! Command arguments are whitespace-separated; library paths containing
//! whitespace are not supported by the v1 wire format (the shared-memory
//! root and standard plugin dirs avoid them).

use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use signal_ipc::{
    MappedSharedMemoryRegion, PluginAudioBlockLayout, PluginAudioBlockView, SharedMemoryBroker,
};
use signal_plugin::PluginParameterDescriptor;
use signal_plugin_clap::{ClapHostedInstance, ClapProcessSession};
use signal_plugin_vst3::{Vst3HostedInstance, Vst3ProcessSession};

/// Number of shared-memory block round-trips performed by the `run` command.
const RUN_BLOCK_COUNT: u64 = 8;
/// Size of the shared-memory region allocated at attach time.
const REGION_BYTES: u32 = 64 * 1024;
/// Spin iterations between `yield_now` calls on the audio thread's wait.
const AUDIO_SPIN_PER_YIELD: u32 = 256;

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
enum SandboxBrokerCommand {
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
}

impl SandboxBrokerCommand {
    fn parse(line: &str) -> Result<Self, String> {
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
            "start-processing" => Ok(Self::StartProcessing),
            "stop-processing" => Ok(Self::StopProcessing),
            "deactivate" => Ok(Self::DeactivatePlugin),
            "unload-plugin" => Ok(Self::UnloadPlugin),
            other => Err(format!("unknown_command:{other}")),
        }
    }
}

struct AttachedRegion {
    region: MappedSharedMemoryRegion,
    lease_id: String,
    processed_blocks: u64,
}

/// The child audio thread: owns the process session and the shm view while
/// processing runs.
struct AudioThread {
    stop: Arc<AtomicBool>,
    join: JoinHandle<()>,
}

/// Shared-memory audio bridge for an activated plugin instance.
struct ActivatedAudio {
    region: MappedSharedMemoryRegion,
    layout: PluginAudioBlockLayout,
    thread: Option<AudioThread>,
}

/// Format-selected hosted instance (g11.031): the broker infers the plugin
/// format from the library path extension (`.clap` / `.vst3`), keeping the
/// stdio wire format unchanged. `load-plugin`'s second argument is the
/// format-native load key: the raw plugin id for CLAP, the component class
/// CID hex for VST3.
enum HostedPluginInstance {
    Clap(ClapHostedInstance),
    Vst3(Vst3HostedInstance),
}

/// Format-selected raw process session for the child audio thread.
enum HostedProcessSession {
    Clap(ClapProcessSession),
    Vst3(Vst3ProcessSession),
}

impl HostedPluginInstance {
    /// Load `load_key` from `library_path`, selecting the format by path
    /// extension. Unknown extensions fail with a stable token.
    fn load(library_path: &str, load_key: &str) -> Result<Self, String> {
        let path = std::path::Path::new(library_path);
        match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("clap") => ClapHostedInstance::load(path, load_key)
                .map(Self::Clap)
                .map_err(|error| error.token),
            Some("vst3") => Vst3HostedInstance::load(path, load_key)
                .map(Self::Vst3)
                .map_err(|error| error.token),
            _ => Err("unsupported_library_extension".to_string()),
        }
    }

    fn parameters(&self) -> Vec<PluginParameterDescriptor> {
        match self {
            Self::Clap(instance) => instance.parameters().to_vec(),
            Self::Vst3(instance) => instance.parameters().to_vec(),
        }
    }

    /// Main-bus channel counts (input, output).
    fn main_ports(&self) -> (u16, u16) {
        match self {
            Self::Clap(instance) => {
                let layout = instance.port_layout();
                (layout.main_input_channels, layout.main_output_channels)
            }
            Self::Vst3(instance) => {
                let layout = instance.port_layout();
                (layout.main_input_channels, layout.main_output_channels)
            }
        }
    }

    /// Phase-1 gate: stereo main in + stereo main out effects only.
    fn is_stereo_effect(&self) -> bool {
        match self {
            Self::Clap(instance) => instance.port_layout().is_stereo_effect(),
            Self::Vst3(instance) => instance.port_layout().is_stereo_effect(),
        }
    }

    fn activate(
        &mut self,
        sample_rate_hz: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<(), String> {
        match self {
            Self::Clap(instance) => instance
                .activate(sample_rate_hz, min_frames, max_frames)
                .map_err(|error| error.token),
            Self::Vst3(instance) => instance
                .activate(sample_rate_hz, min_frames, max_frames)
                .map_err(|error| error.token),
        }
    }

    fn deactivate(&mut self) -> Result<(), String> {
        match self {
            Self::Clap(instance) => instance.deactivate().map_err(|error| error.token),
            Self::Vst3(instance) => instance.deactivate().map_err(|error| error.token),
        }
    }

    fn process_session(&self) -> Result<HostedProcessSession, String> {
        match self {
            Self::Clap(instance) => instance
                .process_session()
                .map(HostedProcessSession::Clap)
                .map_err(|error| error.token),
            Self::Vst3(instance) => instance
                .process_session()
                .map(HostedProcessSession::Vst3)
                .map_err(|error| error.token),
        }
    }
}

impl HostedProcessSession {
    fn start(&mut self) -> Result<(), String> {
        match self {
            Self::Clap(session) => session.start().map_err(|error| error.token),
            Self::Vst3(session) => session.start().map_err(|error| error.token),
        }
    }

    fn stop(&mut self) {
        match self {
            Self::Clap(session) => session.stop(),
            Self::Vst3(session) => session.stop(),
        }
    }

    fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        match self {
            Self::Clap(session) => session.process_interleaved_stereo(input, output, frame_count),
            Self::Vst3(session) => session.process_interleaved_stereo(input, output, frame_count),
        }
    }
}

/// A loaded (and possibly activated) hosted plugin instance.
struct LoadedPlugin {
    instance: HostedPluginInstance,
    plugin_id: String,
    audio: Option<ActivatedAudio>,
}

pub struct SandboxBrokerProcess {
    broker: SharedMemoryBroker,
    sandbox_id: String,
    instance_id: String,
    processing_epoch: u64,
    attached: Option<AttachedRegion>,
    plugin: Option<LoadedPlugin>,
    last_state: SandboxBrokerState,
}

impl Default for SandboxBrokerProcess {
    fn default() -> Self {
        Self {
            broker: SharedMemoryBroker::default(),
            sandbox_id: "plugin-sandbox-broker".into(),
            instance_id: "instance:sandbox:shm".into(),
            processing_epoch: 1,
            attached: None,
            plugin: None,
            last_state: SandboxBrokerState::Starting,
        }
    }
}

impl SandboxBrokerProcess {
    pub fn startup_receipts(&mut self) -> [SandboxBrokerReceipt; 2] {
        self.last_state = SandboxBrokerState::Ready;
        [
            self.receipt(SandboxBrokerState::Starting, "broker_boot"),
            self.receipt(SandboxBrokerState::Ready, "awaiting_commands"),
        ]
    }

    pub fn serve<R: BufRead, W: Write>(&mut self, input: R, mut output: W) -> io::Result<()> {
        for receipt in self.startup_receipts() {
            writeln!(output, "{}", receipt.render_line())?;
        }
        // Receipts must reach the parent promptly: it reads with bounded
        // timeouts, so a buffered writer would read as a stall.
        output.flush()?;

        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match SandboxBrokerCommand::parse(&line) {
                Ok(SandboxBrokerCommand::Status) => {
                    let receipt = self.receipt(self.last_state, "status");
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::Attach) => {
                    let receipt = self.attach();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::Run) => {
                    for receipt in self.run() {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::RunTimeout) => {
                    for receipt in self.run_timeout() {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::LoadPlugin {
                    library_path,
                    plugin_id,
                }) => {
                    let receipt = self.load_plugin(&library_path, &plugin_id);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::ActivatePlugin {
                    sample_rate_hz,
                    min_frames,
                    max_frames,
                }) => {
                    let receipt = self.activate_plugin(sample_rate_hz, min_frames, max_frames);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::StartProcessing) => {
                    let receipt = self.start_processing();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::StopProcessing) => {
                    let receipt = self.stop_processing();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::DeactivatePlugin) => {
                    let receipt = self.deactivate_plugin();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::UnloadPlugin) => {
                    let receipt = self.unload_plugin();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::Teardown) => {
                    let receipt = self.teardown();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::Shutdown) => {
                    let receipt = self.shutdown_receipt();
                    writeln!(output, "{}", receipt.render_line())?;
                    output.flush()?;
                    return Ok(());
                }
                Err(error) => {
                    self.last_state = SandboxBrokerState::Crashed;
                    let receipt = self.receipt(SandboxBrokerState::Crashed, &error);
                    writeln!(output, "{}", receipt.render_line())?;
                }
            }
            output.flush()?;
        }

        let receipt = self.shutdown_receipt();
        writeln!(output, "{}", receipt.render_line())?;
        output.flush()?;
        Ok(())
    }

    fn receipt(&self, state: SandboxBrokerState, detail: &str) -> SandboxBrokerReceipt {
        let attached = self.attached.as_ref();
        SandboxBrokerReceipt {
            state,
            sandbox_id: self.sandbox_id.clone(),
            instance_id: attached.map(|_| self.instance_id.clone()),
            processing_epoch: attached.map(|_| self.processing_epoch),
            lease_id: attached.map(|region| region.lease_id.clone()),
            region_id: attached.map(|region| region.region.metadata().region_id.clone()),
            extra: Vec::new(),
            detail: detail.to_string(),
        }
    }

    fn crashed_receipt(&mut self, detail: &str) -> SandboxBrokerReceipt {
        self.last_state = SandboxBrokerState::Crashed;
        self.receipt(SandboxBrokerState::Crashed, detail)
    }

    // ── Plugin lifecycle (g11.012 batch 12.1) ──────────────────────────────

    /// Load the plugin library (format inferred from the path extension:
    /// `.clap` dlopens through the CLAP hosting FFI, `.vst3` through the
    /// VST3 COM FFI), create+initialize the instance, and enumerate its
    /// parameter inventory (returned in the receipt's `params=` token).
    /// `plugin_id` is the format-native load key (CLAP plugin id / VST3
    /// component class CID hex).
    fn load_plugin(&mut self, library_path: &str, plugin_id: &str) -> SandboxBrokerReceipt {
        if self.plugin.is_some() {
            return self.crashed_receipt("plugin_already_loaded");
        }
        let instance = match HostedPluginInstance::load(library_path, plugin_id) {
            Ok(instance) => instance,
            Err(token) => {
                return self.crashed_receipt(&format!("load_plugin:{token}"));
            }
        };
        let parameters = instance.parameters();
        let (main_inputs, main_outputs) = instance.main_ports();
        self.plugin = Some(LoadedPlugin {
            instance,
            plugin_id: plugin_id.to_string(),
            audio: None,
        });
        self.last_state = SandboxBrokerState::PluginLoaded;
        let mut receipt = self.receipt(
            SandboxBrokerState::PluginLoaded,
            &format!(
                "plugin_loaded|plugin_id={plugin_id}|param_count={}|main_ports={main_inputs}x{main_outputs}",
                parameters.len(),
            ),
        );
        receipt
            .extra
            .push(("params".into(), encode_parameter_inventory(&parameters)));
        receipt
    }

    /// Activate the loaded instance and lease the shared-memory audio block
    /// region. Phase 1 hosts stereo-in/stereo-out effects only; any other
    /// main-port layout is rejected with a typed `layout_unsupported`
    /// receipt (the parent compiles the chain as passthrough).
    fn activate_plugin(
        &mut self,
        sample_rate_hz: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> SandboxBrokerReceipt {
        let Some(plugin) = self.plugin.as_mut() else {
            return self.crashed_receipt("missing_loaded_plugin");
        };
        if plugin.audio.is_some() {
            return self.crashed_receipt("plugin_already_activated");
        }
        if sample_rate_hz <= 0.0 || max_frames == 0 || min_frames > max_frames {
            return self.crashed_receipt("activate_invalid_configuration");
        }
        let (main_inputs, main_outputs) = plugin.instance.main_ports();
        if !plugin.instance.is_stereo_effect() {
            self.last_state = SandboxBrokerState::LayoutUnsupported;
            return self.receipt(
                SandboxBrokerState::LayoutUnsupported,
                &format!(
                    "unsupported_port_layout|main_ports={main_inputs}x{main_outputs}|supported=2x2",
                ),
            );
        }
        if let Err(token) = plugin
            .instance
            .activate(sample_rate_hz, min_frames, max_frames)
        {
            return self.crashed_receipt(&format!("activate:{token}"));
        }
        let block_layout = PluginAudioBlockLayout {
            max_frames,
            channels: 2,
        };
        let lease_id = format!("plugin-audio:{}", self.sandbox_id);
        let region = match self
            .broker
            .create_region(&lease_id, block_layout.region_bytes())
        {
            Ok(mut region) => {
                // Safety: the region is freshly created at the layout's
                // exact size and stays alive while the view is used below.
                let view = unsafe {
                    PluginAudioBlockView::new(region.as_mut_slice().as_mut_ptr(), block_layout)
                };
                view.initialize();
                region
            }
            Err(error) => {
                let _ = plugin.instance.deactivate();
                return self.crashed_receipt(&format!("shm_create:{}", error.detail()));
            }
        };
        let metadata = region.metadata().clone();
        plugin.audio = Some(ActivatedAudio {
            region,
            layout: block_layout,
            thread: None,
        });
        self.last_state = SandboxBrokerState::PluginActivated;
        let mut receipt = self.receipt(
            SandboxBrokerState::PluginActivated,
            &format!(
                "plugin_activated|sample_rate={sample_rate_hz}|max_frames={max_frames}|shm_bytes={}",
                block_layout.region_bytes(),
            ),
        );
        receipt.lease_id = Some(lease_id);
        receipt.region_id = Some(metadata.region_id.clone());
        receipt
            .extra
            .push(("shm_path".into(), encode_wire_token(&metadata.backing_path)));
        receipt
            .extra
            .push(("shm_bytes".into(), metadata.total_bytes.to_string()));
        receipt
            .extra
            .push(("max_frames".into(), max_frames.to_string()));
        receipt.extra.push(("channels".into(), "2".into()));
        receipt
    }

    /// Spawn the audio thread: `start_processing` runs there (CLAP audio
    /// thread contract), then the thread spin/yield-waits on the request
    /// stamp and processes every posted block. No allocation in the loop —
    /// all buffers preallocate here.
    fn start_processing(&mut self) -> SandboxBrokerReceipt {
        let Some(plugin) = self.plugin.as_mut() else {
            return self.crashed_receipt("missing_loaded_plugin");
        };
        let Some(audio) = plugin.audio.as_mut() else {
            return self.crashed_receipt("plugin_not_activated");
        };
        if audio.thread.is_some() {
            return self.crashed_receipt("already_processing");
        }
        let mut session = match plugin.instance.process_session() {
            Ok(session) => session,
            Err(token) => {
                return self.crashed_receipt(&format!("process_session:{token}"));
            }
        };
        let layout = audio.layout;
        // Safety: the mapped region lives in `ActivatedAudio` until the
        // thread is stopped and joined (stop/deactivate/teardown all join
        // before dropping the region).
        let view =
            unsafe { PluginAudioBlockView::new(audio.region.as_mut_slice().as_mut_ptr(), layout) };
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let join = match std::thread::Builder::new()
            .name("sandbox-plugin-audio".into())
            .spawn(move || {
                let max_samples = layout.max_frames as usize * layout.channels as usize;
                let mut input = vec![0.0f32; max_samples];
                let mut output = vec![0.0f32; max_samples];
                if session.start().is_err() {
                    return;
                }
                let mut handled = view.response_seq().load(Ordering::Acquire);
                let mut spins = 0u32;
                while !thread_stop.load(Ordering::Relaxed) {
                    let request = view.request_seq().load(Ordering::Acquire);
                    if request == handled {
                        spins += 1;
                        if spins >= AUDIO_SPIN_PER_YIELD {
                            spins = 0;
                            std::thread::yield_now();
                        } else {
                            std::hint::spin_loop();
                        }
                        continue;
                    }
                    spins = 0;
                    let frames = (view.frame_count().load(Ordering::Relaxed) as usize)
                        .min(layout.max_frames as usize);
                    let samples = frames * layout.channels as usize;
                    // Safety: request/response stamping serializes access to
                    // the sample areas between the two processes.
                    unsafe { view.read_input(&mut input[..samples]) };
                    session.process_interleaved_stereo(
                        &input[..samples],
                        &mut output[..samples],
                        frames,
                    );
                    unsafe { view.write_output(&output[..samples]) };
                    view.response_seq().store(request, Ordering::Release);
                    handled = request;
                }
                session.stop();
            }) {
            Ok(join) => join,
            Err(error) => {
                return self.crashed_receipt(&format!("audio_thread_spawn:{error}"));
            }
        };
        audio.thread = Some(AudioThread { stop, join });
        self.last_state = SandboxBrokerState::ProcessingStarted;
        self.receipt(SandboxBrokerState::ProcessingStarted, "processing_started")
    }

    fn stop_audio_thread(plugin: &mut LoadedPlugin) {
        if let Some(audio) = plugin.audio.as_mut() {
            if let Some(thread) = audio.thread.take() {
                thread.stop.store(true, Ordering::Relaxed);
                let _ = thread.join.join();
            }
        }
    }

    fn stop_processing(&mut self) -> SandboxBrokerReceipt {
        let Some(plugin) = self.plugin.as_mut() else {
            return self.crashed_receipt("missing_loaded_plugin");
        };
        if plugin
            .audio
            .as_ref()
            .and_then(|audio| audio.thread.as_ref())
            .is_none()
        {
            return self.crashed_receipt("not_processing");
        }
        Self::stop_audio_thread(plugin);
        self.last_state = SandboxBrokerState::ProcessingStopped;
        self.receipt(SandboxBrokerState::ProcessingStopped, "processing_stopped")
    }

    /// Deactivate the instance and destroy its audio block region. Stops the
    /// audio thread first when it is still running.
    fn deactivate_plugin(&mut self) -> SandboxBrokerReceipt {
        let Some(plugin) = self.plugin.as_mut() else {
            return self.crashed_receipt("missing_loaded_plugin");
        };
        if plugin.audio.is_none() {
            return self.crashed_receipt("plugin_not_activated");
        }
        Self::stop_audio_thread(plugin);
        let audio = plugin.audio.take().expect("audio checked above");
        let metadata = audio.region.metadata().clone();
        drop(audio.region);
        let destroy_result = self.broker.destroy_region(&metadata);
        let plugin = self.plugin.as_mut().expect("plugin checked above");
        if let Err(token) = plugin.instance.deactivate() {
            return self.crashed_receipt(&format!("deactivate:{token}"));
        }
        if let Err(error) = destroy_result {
            return self.crashed_receipt(&format!("shm_destroy:{}", error.detail()));
        }
        self.last_state = SandboxBrokerState::PluginDeactivated;
        self.receipt(
            SandboxBrokerState::PluginDeactivated,
            "plugin_deactivated|shm_destroyed",
        )
    }

    /// Full plugin teardown: stop processing, deactivate, destroy the
    /// instance and close the library.
    fn unload_plugin(&mut self) -> SandboxBrokerReceipt {
        let Some(mut plugin) = self.plugin.take() else {
            return self.crashed_receipt("missing_loaded_plugin");
        };
        Self::stop_audio_thread(&mut plugin);
        let mut detail = format!("plugin_unloaded|plugin_id={}", plugin.plugin_id);
        if let Some(audio) = plugin.audio.take() {
            let metadata = audio.region.metadata().clone();
            drop(audio.region);
            let _ = plugin.instance.deactivate();
            match self.broker.destroy_region(&metadata) {
                Ok(()) => detail.push_str("|shm_destroyed"),
                Err(error) => detail.push_str(&format!("|shm_destroy_failed:{}", error.detail())),
            }
        }
        drop(plugin);
        self.last_state = SandboxBrokerState::PluginUnloaded;
        self.receipt(SandboxBrokerState::PluginUnloaded, &detail)
    }

    // ── Legacy transport exercise commands ─────────────────────────────────

    fn attach(&mut self) -> SandboxBrokerReceipt {
        if self.attached.is_some() {
            return self.crashed_receipt("already_attached");
        }
        let lease_id = format!("lease:{}", self.sandbox_id);
        match self.broker.create_region(&lease_id, REGION_BYTES) {
            Ok(region) => {
                self.attached = Some(AttachedRegion {
                    region,
                    lease_id,
                    processed_blocks: 0,
                });
                self.last_state = SandboxBrokerState::Attached;
                self.receipt(
                    SandboxBrokerState::Attached,
                    &format!("lease_attached|shm_bytes={REGION_BYTES}"),
                )
            }
            Err(error) => self.crashed_receipt(&format!("shm_create:{}", error.detail())),
        }
    }

    /// Performs verified shared-memory block round-trips: for each block a
    /// deterministic pattern is written into the mapped region, flushed, and
    /// read back. This exercises the transport only — no plugin processing.
    fn run(&mut self) -> Vec<SandboxBrokerReceipt> {
        if self.attached.is_none() {
            return vec![self.crashed_receipt("missing_attached_session")];
        }

        let mut receipts = Vec::new();
        for block_sequence in 0..RUN_BLOCK_COUNT {
            match self.roundtrip_block(block_sequence) {
                Ok(checksum) => {
                    self.last_state = SandboxBrokerState::Running;
                    receipts.push(self.receipt(
                        SandboxBrokerState::Running,
                        &format!(
                            "shm_block_roundtrip|block_sequence={block_sequence}|checksum={checksum}"
                        ),
                    ));
                }
                Err(detail) => {
                    receipts.push(self.crashed_receipt(&detail));
                    return receipts;
                }
            }
        }
        let total = self
            .attached
            .as_ref()
            .map(|attached| attached.processed_blocks)
            .unwrap_or(0);
        self.last_state = SandboxBrokerState::Attached;
        receipts.push(self.receipt(
            SandboxBrokerState::Attached,
            &format!("execution_complete|processed_blocks={RUN_BLOCK_COUNT}|total_blocks={total}"),
        ));
        receipts
    }

    fn roundtrip_block(&mut self, block_sequence: u64) -> Result<u64, String> {
        let Some(attached) = self.attached.as_mut() else {
            return Err("missing_attached_session".into());
        };
        let bytes = attached.region.as_mut_slice();
        for (index, slot) in bytes.iter_mut().enumerate() {
            *slot = (index as u64)
                .wrapping_mul(31)
                .wrapping_add(block_sequence.wrapping_mul(17)) as u8;
        }
        attached
            .region
            .flush()
            .map_err(|error| format!("shm_flush:{error}"))?;
        let mut checksum = 0u64;
        for (index, slot) in attached.region.as_slice().iter().enumerate() {
            let expected = (index as u64)
                .wrapping_mul(31)
                .wrapping_add(block_sequence.wrapping_mul(17)) as u8;
            if *slot != expected {
                return Err(format!(
                    "shm_verify_mismatch|block_sequence={block_sequence}|offset={index}"
                ));
            }
            checksum = checksum
                .wrapping_mul(1099511628211)
                .wrapping_add(*slot as u64);
        }
        attached.processed_blocks = attached.processed_blocks.saturating_add(1);
        Ok(checksum)
    }

    /// Exercises the bounded deadline-miss path: reports a recoverable
    /// timeout, then re-attaches without losing the shared-memory lease.
    fn run_timeout(&mut self) -> Vec<SandboxBrokerReceipt> {
        if self.attached.is_none() {
            return vec![self.crashed_receipt("missing_attached_session")];
        }
        self.last_state = SandboxBrokerState::Attached;
        vec![
            self.receipt(
                SandboxBrokerState::TimedOut,
                "execution_interrupted|timeout=recoverable|lease=retained",
            ),
            self.receipt(
                SandboxBrokerState::Attached,
                "reattached_after_timeout|processed_blocks=0",
            ),
        ]
    }

    fn teardown(&mut self) -> SandboxBrokerReceipt {
        // Plugin teardown path folds into the transport teardown so a
        // parent-side `teardown` always leaves the child clean.
        if self.plugin.is_some() {
            let _ = self.unload_plugin();
        }
        let Some(attached) = self.attached.take() else {
            self.last_state = SandboxBrokerState::TeardownComplete;
            return self.receipt(SandboxBrokerState::TeardownComplete, "teardown_noop");
        };
        let transport = attached.region.metadata().clone();
        let processed_blocks = attached.processed_blocks;
        let instance_id = self.instance_id.clone();
        let processing_epoch = self.processing_epoch;
        drop(attached.region);
        let detail = match self.broker.destroy_region(&transport) {
            Ok(()) => format!(
                "lease_cleanup_ok|region_destroyed|processed_blocks_total={processed_blocks}"
            ),
            Err(error) => {
                self.last_state = SandboxBrokerState::Crashed;
                return SandboxBrokerReceipt {
                    state: SandboxBrokerState::Crashed,
                    sandbox_id: self.sandbox_id.clone(),
                    instance_id: Some(instance_id),
                    processing_epoch: Some(processing_epoch),
                    lease_id: None,
                    region_id: Some(transport.region_id.clone()),
                    extra: Vec::new(),
                    detail: format!("shm_destroy:{}", error.detail()),
                };
            }
        };
        self.last_state = SandboxBrokerState::TeardownComplete;
        SandboxBrokerReceipt {
            state: SandboxBrokerState::TeardownComplete,
            sandbox_id: self.sandbox_id.clone(),
            instance_id: Some(instance_id),
            processing_epoch: Some(processing_epoch),
            lease_id: None,
            region_id: None,
            extra: Vec::new(),
            detail,
        }
    }

    fn shutdown_receipt(&mut self) -> SandboxBrokerReceipt {
        if self.plugin.is_some() {
            let _ = self.unload_plugin();
        }
        if self.attached.is_some() {
            let _ = self.teardown();
        }
        self.last_state = SandboxBrokerState::Shutdown;
        SandboxBrokerReceipt {
            state: SandboxBrokerState::Shutdown,
            sandbox_id: self.sandbox_id.clone(),
            instance_id: None,
            processing_epoch: None,
            lease_id: None,
            region_id: None,
            extra: Vec::new(),
            detail: "broker_shutdown".into(),
        }
    }
}

/// Encode the parameter inventory as
/// `id:name:min:max:default;...` with wire-encoded names.
fn encode_parameter_inventory(parameters: &[PluginParameterDescriptor]) -> String {
    parameters
        .iter()
        .map(|parameter| {
            format!(
                "{}:{}:{}:{}:{}",
                parameter.parameter_id,
                encode_wire_token(&parameter.name),
                parameter.min_plain,
                parameter.max_plain,
                parameter.default_normalized,
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn serve_lines(commands: &str) -> Vec<String> {
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();
        broker
            .serve(Cursor::new(commands.to_string()), &mut output)
            .expect("broker serve should succeed");
        String::from_utf8(output)
            .expect("broker output should be utf-8")
            .lines()
            .map(|line| line.to_string())
            .collect()
    }

    #[test]
    fn broker_reports_startup_and_shutdown() {
        let lines = serve_lines("shutdown\n");
        assert!(lines[0].contains("state=starting"));
        assert!(lines[1].contains("state=ready"));
        assert!(lines[2].contains("state=shutdown"));
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn broker_attach_run_teardown_roundtrips_shared_memory() {
        let lines = serve_lines("attach\nrun\nteardown\nshutdown\n");
        assert!(lines[2].contains("state=attached"));
        assert!(lines[2].contains("lease_id=lease:plugin-sandbox-broker"));
        assert!(lines[2].contains("shm_bytes=65536"));
        let running = lines
            .iter()
            .filter(|line| line.contains("state=running"))
            .count();
        assert_eq!(running, 8);
        assert!(lines
            .iter()
            .any(|line| line.contains("execution_complete|processed_blocks=8")));
        assert!(lines
            .iter()
            .any(|line| line.contains("state=teardown_complete")
                && line.contains("lease_cleanup_ok")));
    }

    #[test]
    fn broker_timeout_path_reports_recoverable_interrupt_and_reattaches() {
        let lines = serve_lines("attach\nrun-timeout\nteardown\nshutdown\n");
        assert!(lines
            .iter()
            .any(|line| line.contains("state=timed_out") && line.contains("timeout=recoverable")));
        assert!(lines
            .iter()
            .any(|line| line.contains("reattached_after_timeout")));
    }

    #[test]
    fn broker_rejects_run_without_attach_and_unknown_commands() {
        let lines = serve_lines("run\nbogus\nshutdown\n");
        assert!(lines.iter().any(
            |line| line.contains("state=crashed") && line.contains("missing_attached_session")
        ));
        assert!(lines
            .iter()
            .any(|line| line.contains("unknown_command:bogus")));
    }

    #[test]
    fn broker_rejects_plugin_commands_without_a_loaded_plugin() {
        let lines = serve_lines(
            "activate 48000 1 256\nstart-processing\nstop-processing\ndeactivate\nunload-plugin\nshutdown\n",
        );
        let crashed = lines
            .iter()
            .filter(|line| line.contains("state=crashed") && line.contains("missing_loaded_plugin"))
            .count();
        assert_eq!(crashed, 5);
    }

    #[test]
    fn broker_rejects_malformed_plugin_commands() {
        let lines = serve_lines("load-plugin /tmp/only-path\nactivate 48000\nshutdown\n");
        assert!(lines
            .iter()
            .any(|line| line.contains("load_plugin_missing_plugin_id")));
        assert!(lines
            .iter()
            .any(|line| line.contains("activate_missing_min_frames")));
    }

    #[test]
    fn broker_rejects_load_of_missing_library_with_typed_detail() {
        let lines =
            serve_lines("load-plugin /nonexistent/fixture.clap com.signal.missing\nshutdown\n");
        assert!(lines.iter().any(|line| line.contains("state=crashed")
            && line.contains("load_plugin:library_open_failed")));
    }

    #[test]
    fn wire_token_encoding_escapes_separators() {
        assert_eq!(encode_wire_token("Gain"), "Gain");
        assert_eq!(encode_wire_token("Dry / Wet Mix"), "Dry%20/%20Wet%20Mix");
        assert_eq!(encode_wire_token("a:b;c=d|e%f"), "a%3Ab%3Bc%3Dd%7Ce%25f");
    }
}
