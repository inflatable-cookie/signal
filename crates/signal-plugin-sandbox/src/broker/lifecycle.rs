//! Plugin load/activate/editor/processing commands for the sandbox broker.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use signal_ipc::{PluginAudioBlockLayout, PluginAudioBlockView, PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY};
use signal_plugin::{read_event_from_slice, PluginEvent};

use super::hosted::*;
use super::process::SandboxBrokerProcess;
use super::types::*;

struct ProcessingMember {
    view: PluginAudioBlockView,
    session: HostedProcessSession,
    layout: PluginAudioBlockLayout,
    handled: u32,
    input: Vec<f32>,
    output: Vec<f32>,
    events: Vec<PluginEvent>,
}

impl SandboxBrokerProcess {
    pub(crate) fn load_plugin(
        &mut self,
        library_path: &str,
        plugin_id: &str,
    ) -> SandboxBrokerReceipt {
        let instance_id = self.sandbox_id.clone();
        self.load_plugin_instance(&instance_id, library_path, plugin_id)
    }

    pub(crate) fn load_plugin_instance(
        &mut self,
        instance_id: &str,
        library_path: &str,
        plugin_id: &str,
    ) -> SandboxBrokerReceipt {
        if self.audio_thread.is_some() {
            return self.crashed_receipt_for(instance_id, "already_processing");
        }
        if self.plugins.contains_key(instance_id) {
            return self.crashed_receipt_for(instance_id, "plugin_already_loaded");
        }
        let instance = match HostedPluginInstance::load(library_path, plugin_id) {
            Ok(instance) => instance,
            Err(token) => {
                return self.crashed_receipt_for(instance_id, &format!("load_plugin:{token}"));
            }
        };
        let parameters = instance.parameters();
        let (main_inputs, main_outputs) = instance.main_ports();
        self.plugins.insert(
            instance_id.to_string(),
            LoadedPlugin {
                instance,
                plugin_id: plugin_id.to_string(),
                audio: None,
            },
        );
        self.last_state = SandboxBrokerState::PluginLoaded;
        let mut receipt = self.plugin_receipt(
            instance_id,
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

    pub(crate) fn activate_plugin(
        &mut self,
        sample_rate_hz: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> SandboxBrokerReceipt {
        let instance_id = self.sandbox_id.clone();
        self.activate_plugin_instance(&instance_id, sample_rate_hz, min_frames, max_frames)
    }

    /// Activate a loaded instance and lease its shared-memory audio block
    /// region. Supports stereo effects (2x2) and instruments (0x2); any other
    /// main-port layout is rejected with a typed `layout_unsupported`
    /// receipt (the parent compiles the chain as passthrough).
    pub(crate) fn activate_plugin_instance(
        &mut self,
        instance_id: &str,
        sample_rate_hz: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> SandboxBrokerReceipt {
        if self.audio_thread.is_some() {
            return self.crashed_receipt_for(instance_id, "already_processing");
        }
        let Some(plugin) = self.plugins.get_mut(instance_id) else {
            return self.crashed_receipt_for(instance_id, "missing_loaded_plugin");
        };
        if plugin.audio.is_some() {
            return self.crashed_receipt_for(instance_id, "plugin_already_activated");
        }
        if sample_rate_hz <= 0.0 || max_frames == 0 || min_frames > max_frames {
            return self.crashed_receipt_for(instance_id, "activate_invalid_configuration");
        }
        let (main_inputs, main_outputs) = plugin.instance.main_ports();
        if !plugin.instance.is_supported_stereo_processor() {
            self.last_state = SandboxBrokerState::LayoutUnsupported;
            return self.plugin_receipt(
                instance_id,
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
            return self.crashed_receipt_for(instance_id, &format!("activate:{token}"));
        }
        let block_layout = PluginAudioBlockLayout {
            max_frames,
            channels: 2,
        };
        let lease_id = format!("plugin-audio:{instance_id}");
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
                return self
                    .crashed_receipt_for(instance_id, &format!("shm_create:{}", error.detail()));
            }
        };
        let metadata = region.metadata().clone();
        plugin.audio = Some(ActivatedAudio {
            region,
            layout: block_layout,
        });
        self.last_state = SandboxBrokerState::PluginActivated;
        let mut receipt = self.plugin_receipt(
            instance_id,
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

    /// Apply a batch of normalized parameter writes to the default instance
    /// (g12.023). Valid on any loaded plugin — queue-backed formats apply
    /// at the next processed block; AU applies immediately. Preserves
    /// `last_state` (a param set is not a lifecycle transition); the first
    /// failing change crashes the receipt with its typed token.
    pub(crate) fn set_parameters(&mut self, changes: &[(u32, f32)]) -> SandboxBrokerReceipt {
        let instance_id = self.sandbox_id.clone();
        let Some(plugin) = self.plugins.get_mut(&instance_id) else {
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

    /// Open the child-owned editor window for the default plugin: the
    /// per-format spec is extracted here (control thread), the window +
    /// gui session are created on the child's MAIN thread via the GUI
    /// handle (blocking marshal — this thread waits, so instance access
    /// never overlaps). Preserves `last_state` (an editor open is not a
    /// lifecycle transition, matching `param_set`).
    pub(crate) fn open_editor(&mut self, instance: &str) -> SandboxBrokerReceipt {
        let instance_id = self.sandbox_id.clone();
        let Some(plugin) = self.plugins.get(&instance_id) else {
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

    /// Spawn the boundary audio thread and wait for every activated member's
    /// `start_processing` to complete there (CLAP audio-thread contract)
    /// before acknowledging the control command. One thread polls member
    /// request stamps. v1 does not add members after this command.
    pub(crate) fn start_processing(&mut self) -> SandboxBrokerReceipt {
        if self.audio_thread.is_some() {
            return self.crashed_receipt("already_processing");
        }
        if self.plugins.is_empty() {
            return self.crashed_receipt("missing_loaded_plugin");
        }
        if !self.plugins.values().any(|plugin| plugin.audio.is_some()) {
            return self.crashed_receipt("plugin_not_activated");
        }

        let mut members = Vec::new();
        for plugin in self.plugins.values_mut() {
            let Some(audio) = plugin.audio.as_mut() else {
                continue;
            };
            let session = match plugin.instance.process_session() {
                Ok(session) => session,
                Err(token) => {
                    return self.crashed_receipt(&format!("process_session:{token}"));
                }
            };
            let layout = audio.layout;
            // Safety: each mapped region lives in `ActivatedAudio` until the
            // thread is stopped and joined (stop/deactivate/teardown all join
            // before dropping the region).
            let view = unsafe {
                PluginAudioBlockView::new(audio.region.as_mut_slice().as_mut_ptr(), layout)
            };
            let handled = view.response_seq().load(Ordering::Acquire);
            let max_samples = layout.max_frames as usize * layout.channels as usize;
            members.push(ProcessingMember {
                view,
                session,
                layout,
                handled,
                input: vec![0.0f32; max_samples],
                output: vec![0.0f32; max_samples],
                events: Vec::with_capacity(PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY),
            });
        }

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let join = match std::thread::Builder::new()
            .name("sandbox-plugin-audio".into())
            .spawn(move || {
                run_member_audio_thread(members, thread_stop, ready_tx);
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
        self.audio_thread = Some(AudioThread { stop, join });
        self.last_state = SandboxBrokerState::ProcessingStarted;
        self.receipt(SandboxBrokerState::ProcessingStarted, "processing_started")
    }

    pub(crate) fn stop_audio_thread(&mut self) {
        if let Some(thread) = self.audio_thread.take() {
            thread.stop.store(true, Ordering::Relaxed);
            let _ = thread.join.join();
        }
    }

    pub(crate) fn stop_processing(&mut self) -> SandboxBrokerReceipt {
        if self.audio_thread.is_none() {
            if self.plugins.is_empty() {
                return self.crashed_receipt("missing_loaded_plugin");
            }
            return self.crashed_receipt("not_processing");
        }
        self.stop_audio_thread();
        self.last_state = SandboxBrokerState::ProcessingStopped;
        self.receipt(SandboxBrokerState::ProcessingStopped, "processing_stopped")
    }

    pub(crate) fn deactivate_plugin(&mut self) -> SandboxBrokerReceipt {
        let instance_id = self.sandbox_id.clone();
        self.deactivate_plugin_instance(&instance_id)
    }

    /// Deactivate one instance and destroy its audio block region. Stops the
    /// boundary audio thread first when it is still running.
    pub(crate) fn deactivate_plugin_instance(&mut self, instance_id: &str) -> SandboxBrokerReceipt {
        let Some(plugin) = self.plugins.get_mut(instance_id) else {
            return self.crashed_receipt_for(instance_id, "missing_loaded_plugin");
        };
        if plugin.audio.is_none() {
            return self.crashed_receipt_for(instance_id, "plugin_not_activated");
        }
        self.stop_audio_thread();
        let plugin = self
            .plugins
            .get_mut(instance_id)
            .expect("plugin checked above");
        let audio = plugin.audio.take().expect("audio checked above");
        let metadata = audio.region.metadata().clone();
        drop(audio.region);
        let destroy_result = self.broker.destroy_region(&metadata);
        let plugin = self
            .plugins
            .get_mut(instance_id)
            .expect("plugin checked above");
        if let Err(token) = plugin.instance.deactivate() {
            return self.crashed_receipt_for(instance_id, &format!("deactivate:{token}"));
        }
        if let Err(error) = destroy_result {
            return self
                .crashed_receipt_for(instance_id, &format!("shm_destroy:{}", error.detail()));
        }
        self.last_state = SandboxBrokerState::PluginDeactivated;
        self.plugin_receipt(
            instance_id,
            SandboxBrokerState::PluginDeactivated,
            "plugin_deactivated|shm_destroyed",
        )
    }

    pub(crate) fn unload_plugin(&mut self) -> SandboxBrokerReceipt {
        let instance_id = self.sandbox_id.clone();
        self.unload_plugin_instance(&instance_id)
    }

    /// Full teardown of one instance: stop processing, deactivate, destroy
    /// the instance and close the library when it is the last user.
    pub(crate) fn unload_plugin_instance(&mut self, instance_id: &str) -> SandboxBrokerReceipt {
        if !self.plugins.contains_key(instance_id) {
            return self.crashed_receipt_for(instance_id, "missing_loaded_plugin");
        }
        self.stop_audio_thread();
        if let Some(gui) = self.gui.as_ref() {
            gui.close_all();
        }
        let Some(mut plugin) = self.plugins.remove(instance_id) else {
            return self.crashed_receipt_for(instance_id, "missing_loaded_plugin");
        };
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
        self.plugin_receipt(instance_id, SandboxBrokerState::PluginUnloaded, &detail)
    }

    pub(crate) fn unload_all_plugins(&mut self) {
        self.stop_audio_thread();
        let ids: Vec<String> = self.plugins.keys().cloned().collect();
        for id in ids {
            let _ = self.unload_plugin_instance(&id);
        }
    }
}

fn run_member_audio_thread(
    mut members: Vec<ProcessingMember>,
    thread_stop: Arc<AtomicBool>,
    ready_tx: mpsc::SyncSender<Result<(), String>>,
) {
    for index in 0..members.len() {
        match members[index].session.start() {
            Ok(()) => {}
            Err(token) => {
                for started in members.iter_mut().take(index) {
                    started.session.stop();
                }
                let _ = ready_tx.send(Err(token));
                return;
            }
        }
    }
    if ready_tx.send(Ok(())).is_err() {
        for member in &mut members {
            member.session.stop();
        }
        return;
    }

    let mut spins = 0u32;
    while !thread_stop.load(Ordering::Relaxed) {
        let mut idle = true;
        for member in &mut members {
            let request = member.view.request_seq().load(Ordering::Acquire);
            if request == member.handled {
                continue;
            }
            idle = false;
            let frames = (member.view.frame_count().load(Ordering::Relaxed) as usize)
                .min(member.layout.max_frames as usize);
            let samples = frames * member.layout.channels as usize;
            // Safety: request/response stamping serializes access to
            // the sample areas between the two processes.
            unsafe { member.view.read_input(&mut member.input[..samples]) };
            member.events.clear();
            let event_count = (member.view.event_count().load(Ordering::Relaxed) as usize)
                .min(PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY);
            for event_index in 0..event_count {
                let mut encoded = [0u8; PluginEvent::ENCODED_BYTES];
                unsafe { member.view.read_event(event_index, &mut encoded) };
                if let Ok(event) = read_event_from_slice(&encoded) {
                    member.events.push(event);
                }
            }
            member.session.process_interleaved_stereo_with_events(
                &member.input[..samples],
                &mut member.output[..samples],
                frames,
                &member.events,
            );
            unsafe { member.view.write_output(&member.output[..samples]) };
            member.view.response_seq().store(request, Ordering::Release);
            member.handled = request;
        }
        if idle {
            spins += 1;
            if spins >= AUDIO_SPIN_PER_YIELD {
                spins = 0;
                std::thread::yield_now();
            } else {
                std::hint::spin_loop();
            }
        } else {
            spins = 0;
        }
    }
    for member in &mut members {
        member.session.stop();
    }
}
