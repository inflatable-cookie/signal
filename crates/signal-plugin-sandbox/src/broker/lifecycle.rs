//! Plugin load/activate/editor/processing commands for the sandbox broker.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use signal_ipc::{PluginAudioBlockLayout, PluginAudioBlockView, PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY};
use signal_plugin::{read_event_from_slice, PluginEvent};

use super::hosted::*;
use super::process::SandboxBrokerProcess;
use super::types::*;

impl SandboxBrokerProcess {
    pub(crate) fn load_plugin(
        &mut self,
        library_path: &str,
        plugin_id: &str,
    ) -> SandboxBrokerReceipt {
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
    /// region. Supports stereo effects (2x2) and instruments (0x2); any other
    /// main-port layout is rejected with a typed `layout_unsupported`
    /// receipt (the parent compiles the chain as passthrough).
    pub(crate) fn activate_plugin(
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
        if !plugin.instance.is_supported_stereo_processor() {
            self.last_state = SandboxBrokerState::LayoutUnsupported;
            return self.receipt(
                SandboxBrokerState::LayoutUnsupported,
                &format!(
                    "unsupported_port_layout|main_ports={main_inputs}x{main_outputs}|supported=0x2,2x2",
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

    /// Apply a batch of normalized parameter writes to the loaded instance
    /// (g12.023). Valid on any loaded plugin — queue-backed formats apply
    /// at the next processed block; AU applies immediately. Preserves
    /// `last_state` (a param set is not a lifecycle transition); the first
    /// failing change crashes the receipt with its typed token.
    pub(crate) fn set_parameters(&mut self, changes: &[(u32, f32)]) -> SandboxBrokerReceipt {
        let Some(plugin) = self.plugin.as_mut() else {
            return self.crashed_receipt("missing_loaded_plugin");
        };
        for (parameter_id, normalized) in changes {
            if let Err(token) = plugin
                .instance
                .set_parameter_normalized(*parameter_id, *normalized)
            {
                return self.crashed_receipt(&format!("set_param:{parameter_id}:{token}"));
            }
        }
        self.receipt(
            SandboxBrokerState::ParamSet,
            &format!("param_set|count={}", changes.len()),
        )
    }

    // ── Child-owned editor windows (g13.027 batch 1) ───────────────────────

    /// Open the child-owned editor window for the loaded plugin: the
    /// per-format spec is extracted here (control thread), the window +
    /// gui session are created on the child's MAIN thread via the GUI
    /// handle (blocking marshal — this thread waits, so instance access
    /// never overlaps). Preserves `last_state` (an editor open is not a
    /// lifecycle transition, matching `param_set`).
    pub(crate) fn open_editor(&mut self, instance: &str) -> SandboxBrokerReceipt {
        let Some(plugin) = self.plugin.as_ref() else {
            return self.crashed_receipt("missing_loaded_plugin");
        };
        let Some(gui) = self.gui.as_ref() else {
            return self.crashed_receipt("open_editor:gui_unavailable");
        };
        let spec = match plugin.instance.child_editor_spec() {
            Ok(spec) => spec,
            Err(token) => return self.crashed_receipt(&format!("open_editor:{token}")),
        };
        match gui.open_editor(instance, spec) {
            Ok((width, height)) => {
                let mut receipt = self.receipt(
                    SandboxBrokerState::EditorOpened,
                    &format!("editor_opened|width={width}|height={height}"),
                );
                receipt
                    .extra
                    .push(("editor_instance".into(), encode_wire_token(instance)));
                receipt.extra.push(("width".into(), width.to_string()));
                receipt.extra.push(("height".into(), height.to_string()));
                receipt
            }
            Err(token) => self.crashed_receipt(&format!("open_editor:{token}")),
        }
    }

    /// Close the child-owned editor window. Tolerant of an already-closed
    /// editor (the user may have closed the window first): the receipt's
    /// `reason` token distinguishes `host_requested` from `not_open`.
    pub(crate) fn close_editor(&mut self, instance: &str) -> SandboxBrokerReceipt {
        let Some(gui) = self.gui.as_ref() else {
            return self.crashed_receipt("close_editor:gui_unavailable");
        };
        match gui.close_editor(instance) {
            Ok(closed) => {
                let reason = if closed { "host_requested" } else { "not_open" };
                let mut receipt = self.receipt(
                    SandboxBrokerState::EditorClosed,
                    &format!("editor_closed|reason={reason}"),
                );
                receipt
                    .extra
                    .push(("editor_instance".into(), encode_wire_token(instance)));
                receipt.extra.push(("reason".into(), reason.into()));
                receipt
            }
            Err(token) => self.crashed_receipt(&format!("close_editor:{token}")),
        }
    }

    /// Spawn the audio thread and wait for `start_processing` to complete
    /// there (CLAP audio-thread contract) before acknowledging the control
    /// command. The parent must not publish render blocks against a thread
    /// that is still entering its processing state. Once ready, the thread
    /// spin/yield-waits on the request stamp and processes every posted
    /// block. No allocation occurs in that loop — all buffers preallocate
    /// before readiness is published.
    pub(crate) fn start_processing(&mut self) -> SandboxBrokerReceipt {
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
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let join = match std::thread::Builder::new()
            .name("sandbox-plugin-audio".into())
            .spawn(move || {
                let max_samples = layout.max_frames as usize * layout.channels as usize;
                let mut input = vec![0.0f32; max_samples];
                let mut output = vec![0.0f32; max_samples];
                let mut events = Vec::with_capacity(PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY);
                match session.start() {
                    Ok(()) => {
                        if ready_tx.send(Ok(())).is_err() {
                            session.stop();
                            return;
                        }
                    }
                    Err(token) => {
                        let _ = ready_tx.send(Err(token));
                        return;
                    }
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
                    events.clear();
                    let event_count = (view.event_count().load(Ordering::Relaxed) as usize)
                        .min(PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY);
                    for index in 0..event_count {
                        let mut encoded = [0u8; PluginEvent::ENCODED_BYTES];
                        unsafe { view.read_event(index, &mut encoded) };
                        if let Ok(event) = read_event_from_slice(&encoded) {
                            events.push(event);
                        }
                    }
                    session.process_interleaved_stereo_with_events(
                        &input[..samples],
                        &mut output[..samples],
                        frames,
                        &events,
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
        // This is a control-plane wait. The parent broker client owns the
        // ten-second receipt deadline and kills the entire child on expiry,
        // which is the only safe way to unwind a plugin whose `start()` call
        // itself never returns.
        match ready_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(token)) => {
                stop.store(true, Ordering::Relaxed);
                let _ = join.join();
                return self.crashed_receipt(&format!("start_processing:{token}"));
            }
            Err(mpsc::RecvError) => {
                stop.store(true, Ordering::Relaxed);
                let _ = join.join();
                return self.crashed_receipt("start_processing:audio_thread_exited");
            }
        }
        audio.thread = Some(AudioThread { stop, join });
        self.last_state = SandboxBrokerState::ProcessingStarted;
        self.receipt(SandboxBrokerState::ProcessingStarted, "processing_started")
    }

    pub(crate) fn stop_audio_thread(plugin: &mut LoadedPlugin) {
        if let Some(audio) = plugin.audio.as_mut() {
            if let Some(thread) = audio.thread.take() {
                thread.stop.store(true, Ordering::Relaxed);
                let _ = thread.join.join();
            }
        }
    }

    pub(crate) fn stop_processing(&mut self) -> SandboxBrokerReceipt {
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
    pub(crate) fn deactivate_plugin(&mut self) -> SandboxBrokerReceipt {
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
    pub(crate) fn unload_plugin(&mut self) -> SandboxBrokerReceipt {
        let Some(mut plugin) = self.plugin.take() else {
            return self.crashed_receipt("missing_loaded_plugin");
        };
        // Editors hold gui sessions pointing into this instance: close
        // them on the main thread BEFORE the instance is destroyed.
        if let Some(gui) = self.gui.as_ref() {
            gui.close_all();
        }
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
}
