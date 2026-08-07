//! Sandbox broker process: command serve loop and plugin lifecycle.

use std::io::{self, BufRead, Write};

use crate::child_gui::ChildGuiHandle;
use signal_ipc::SharedMemoryBroker;

use super::hosted::*;
use super::types::*;

pub struct SandboxBrokerProcess {
    pub(crate) broker: SharedMemoryBroker,
    pub(crate) sandbox_id: String,
    pub(crate) instance_id: String,
    pub(crate) processing_epoch: u64,
    pub(crate) attached: Option<AttachedRegion>,
    pub(crate) plugin: Option<LoadedPlugin>,
    pub(crate) last_state: SandboxBrokerState,
    /// Marshals editor lifecycle onto the child's main thread (g13.027).
    /// `None` outside the real child process (unit tests, non-GUI serves):
    /// editor commands then fail with the typed `gui_unavailable` token.
    pub(crate) gui: Option<ChildGuiHandle>,
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
            gui: None,
        }
    }
}

impl SandboxBrokerProcess {
    pub fn set_gui_handle(&mut self, gui: ChildGuiHandle) {
        self.gui = Some(gui);
    }

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
                Ok(SandboxBrokerCommand::SetParameters { changes }) => {
                    let receipt = self.set_parameters(&changes);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::OpenEditor { instance }) => {
                    let receipt = self.open_editor(&instance);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::CloseEditor { instance }) => {
                    let receipt = self.close_editor(&instance);
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

    pub(crate) fn receipt(&self, state: SandboxBrokerState, detail: &str) -> SandboxBrokerReceipt {
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

    pub(crate) fn crashed_receipt(&mut self, detail: &str) -> SandboxBrokerReceipt {
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
