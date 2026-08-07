use super::super::types::*;

impl SandboxBrokerClientSession {
    /// Sends `load-plugin` and returns the child's parameter inventory.
    ///
    /// The v1 wire format is whitespace-separated: library paths containing
    /// whitespace are rejected here rather than corrupting the command line.
    pub fn load_plugin(
        &mut self,
        library_path: &str,
        plugin_id: &str,
    ) -> std::io::Result<SandboxPluginInventory> {
        if library_path.chars().any(char::is_whitespace)
            || plugin_id.chars().any(char::is_whitespace)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "plugin library paths and ids with whitespace are unsupported by the v1 broker wire format",
            ));
        }
        self.write_command(&format!("load-plugin {library_path} {plugin_id}"))?;
        let receipt = self.read_receipt()?;
        if receipt.state != SandboxBrokerReceiptState::PluginLoaded {
            return Err(std::io::Error::other(format!(
                "unexpected broker load-plugin state: {} ({})",
                receipt.state, receipt.detail
            )));
        }
        let parameters = receipt
            .extra_value("params")
            .map(parse_parameter_inventory)
            .unwrap_or_default();
        Ok(SandboxPluginInventory {
            parameters,
            detail: receipt.detail,
        })
    }

    /// Sends `activate` and returns either the audio block lease or the
    /// typed layout rejection.
    pub fn activate_plugin(
        &mut self,
        sample_rate_hz: u32,
        min_frames: u32,
        max_frames: u32,
    ) -> std::io::Result<SandboxPluginActivateOutcome> {
        self.write_command(&format!(
            "activate {sample_rate_hz} {min_frames} {max_frames}"
        ))?;
        let receipt = self.read_receipt()?;
        match receipt.state {
            SandboxBrokerReceiptState::PluginActivated => {
                let shm_path = receipt
                    .extra_value("shm_path")
                    .map(decode_wire_token)
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "plugin_activated receipt missing shm_path",
                        )
                    })?;
                let shm_bytes = receipt
                    .extra_value("shm_bytes")
                    .and_then(|value| value.parse::<u32>().ok())
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "plugin_activated receipt missing shm_bytes",
                        )
                    })?;
                let lease_max_frames = receipt
                    .extra_value("max_frames")
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(max_frames);
                let channels = receipt
                    .extra_value("channels")
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(2);
                Ok(SandboxPluginActivateOutcome::Activated(
                    SandboxPluginAudioLease {
                        region_id: receipt.region_id.unwrap_or_default(),
                        lease_id: receipt.lease_id.unwrap_or_default(),
                        shm_path,
                        shm_bytes,
                        max_frames: lease_max_frames,
                        channels,
                        detail: receipt.detail,
                    },
                ))
            }
            SandboxBrokerReceiptState::LayoutUnsupported => {
                Ok(SandboxPluginActivateOutcome::LayoutUnsupported {
                    detail: receipt.detail,
                })
            }
            other => Err(std::io::Error::other(format!(
                "unexpected broker activate state: {} ({})",
                other, receipt.detail
            ))),
        }
    }

    pub(crate) fn simple_plugin_command(
        &mut self,
        command: &str,
        expected: SandboxBrokerReceiptState,
    ) -> std::io::Result<String> {
        self.write_command(command)?;
        let receipt = self.read_receipt()?;
        if receipt.state != expected {
            return Err(std::io::Error::other(format!(
                "unexpected broker {command} state: {} ({})",
                receipt.state, receipt.detail
            )));
        }
        Ok(receipt.detail)
    }

    /// Sends `start-processing`: the child spawns its audio thread.
    pub fn start_processing(&mut self) -> std::io::Result<String> {
        self.simple_plugin_command(
            "start-processing",
            SandboxBrokerReceiptState::ProcessingStarted,
        )
    }

    /// Sends `stop-processing`: the child stops and joins its audio thread.
    pub fn stop_processing(&mut self) -> std::io::Result<String> {
        self.simple_plugin_command(
            "stop-processing",
            SandboxBrokerReceiptState::ProcessingStopped,
        )
    }

    /// Sends `deactivate`: the child deactivates the plugin and destroys the
    /// audio block region (any parent mapping goes stale first — detach
    /// before calling this).
    pub fn deactivate_plugin(&mut self) -> std::io::Result<String> {
        self.simple_plugin_command("deactivate", SandboxBrokerReceiptState::PluginDeactivated)
    }

    /// Sends `unload-plugin`: full plugin teardown in the child.
    pub fn unload_plugin(&mut self) -> std::io::Result<String> {
        self.simple_plugin_command("unload-plugin", SandboxBrokerReceiptState::PluginUnloaded)
    }
}
