use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::vst3_host_adapter::hosting::Vst3HostingError;

use super::super::process::Vst3ProcessSession;
use super::super::wire::*;
use super::hosted::Vst3HostedInstance;
use super::layout::{
    audio_bus_layout, bus_arrangements, pointer_or_null, HostedInstanceState, Vst3HostedPortLayout,
};

impl Vst3HostedInstance {
    /// Current main-bus port layout, including successful activation-time
    /// negotiation.
    pub fn port_layout(&self) -> Vst3HostedPortLayout {
        self.port_layout
    }

    /// Current processor-reported latency in sample frames.
    pub fn latency_frames(&self) -> u32 {
        unsafe {
            let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
            ((*vtable).get_latency_samples)(self.processor)
        }
    }

    /// Number of controller `kLatencyChanged` restart notifications.
    pub fn latency_change_count(&self) -> u64 {
        self.component_handler
            .as_ref()
            .map(|handler| handler.latency_changes.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Shared restart flags accepted from `IComponentHandler`. Audio hosts
    /// use this to stop at a block boundary before the control thread
    /// services the requested lifecycle transition.
    pub fn pending_restart_flags(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.pending_restart_flags)
    }

    /// Deactivate, refresh dynamic I/O when requested, reactivate, and build
    /// a replacement process session on the owning control thread.
    pub fn restart_processing(
        &mut self,
        flags: u32,
    ) -> Result<Vst3ProcessSession, Vst3HostingError> {
        let sample_rate_hz = self.activated_sample_rate_hz;
        let max_frames = self.activated_max_frames;
        self.deactivate()?;
        if flags & VST3_RESTART_IO_CHANGED != 0 {
            self.audio_bus_layout = unsafe { audio_bus_layout(self.component) };
            self.port_layout = self.audio_bus_layout.port_layout();
        }
        self.activate(sample_rate_hz, 1, max_frames)?;
        self.process_session()
    }

    /// Activate for processing by negotiating the available main buses to a
    /// stereo effect (2-in/2-out) or instrument (0-in/2-out), then selecting
    /// 32-bit samples, calling `setupProcessing`, activating the main buses,
    /// and calling `setActive(true)`. Unsupported negotiation fails with the
    /// stable `layout_unsupported` token, same as the CLAP path. Components
    /// without any audio output fail with `no_audio_buses`; their editors may
    /// still be hosted without creating a process session.
    pub fn activate(
        &mut self,
        sample_rate_hz: f64,
        _min_frames: u32,
        max_frames: u32,
    ) -> Result<(), Vst3HostingError> {
        if self.state == HostedInstanceState::Active {
            return Err(Vst3HostingError::new("already_active"));
        }
        if self.audio_bus_layout.main_output.is_none() {
            return Err(Vst3HostingError::new("no_audio_buses"));
        }
        unsafe {
            let processor = vtable_of::<AudioProcessorVTable>(self.processor);
            let has_audio_input = self.audio_bus_layout.main_input.is_some();

            // VST3 requires the arrangement array to cover every declared bus,
            // including inactive auxiliaries. Preserve each auxiliary layout
            // and negotiate only the main bus to stereo.
            let mut input_arrangements = bus_arrangements(
                self.processor,
                K_INPUT,
                &self.audio_bus_layout.input_channels,
            );
            let mut output_arrangements = bus_arrangements(
                self.processor,
                K_OUTPUT,
                &self.audio_bus_layout.output_channels,
            );
            if let Some(index) = self.audio_bus_layout.main_input {
                input_arrangements[index] = STEREO_ARRANGEMENT;
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                output_arrangements[index] = STEREO_ARRANGEMENT;
            }
            let _ = ((*processor).set_bus_arrangements)(
                self.processor,
                pointer_or_null(&mut input_arrangements),
                input_arrangements.len() as i32,
                pointer_or_null(&mut output_arrangements),
                output_arrangements.len() as i32,
            );
            let mut verified_input = 0u64;
            let mut verified_output = 0u64;
            let input_verified = !has_audio_input
                || (((*processor).get_bus_arrangement)(
                    self.processor,
                    K_INPUT,
                    self.audio_bus_layout.main_input.unwrap_or(0) as i32,
                    &mut verified_input,
                ) == K_RESULT_OK
                    && verified_input == STEREO_ARRANGEMENT);
            let output_result = ((*processor).get_bus_arrangement)(
                self.processor,
                K_OUTPUT,
                self.audio_bus_layout.main_output.unwrap_or(0) as i32,
                &mut verified_output,
            );
            if !input_verified
                || output_result != K_RESULT_OK
                || verified_output != STEREO_ARRANGEMENT
            {
                return Err(Vst3HostingError::new("layout_unsupported"));
            }
            if let Some(index) = self.audio_bus_layout.main_input {
                self.audio_bus_layout.input_channels[index] = 2;
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                self.audio_bus_layout.output_channels[index] = 2;
            }
            self.port_layout = self.audio_bus_layout.port_layout();

            if ((*processor).can_process_sample_size)(self.processor, K_SAMPLE32) != K_RESULT_OK {
                return Err(Vst3HostingError::new("sample_size_unsupported"));
            }

            let mut setup = ProcessSetup {
                process_mode: K_REALTIME,
                symbolic_sample_size: K_SAMPLE32,
                max_samples_per_block: max_frames as i32,
                sample_rate: sample_rate_hz,
            };
            if ((*processor).setup_processing)(self.processor, &mut setup) != K_RESULT_OK {
                return Err(Vst3HostingError::new("setup_processing_failed"));
            }

            let component = vtable_of::<ComponentVTable>(self.component);
            if let Some(index) = self.audio_bus_layout.main_input {
                let _ =
                    ((*component).activate_bus)(self.component, K_AUDIO, K_INPUT, index as i32, 1);
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                let _ =
                    ((*component).activate_bus)(self.component, K_AUDIO, K_OUTPUT, index as i32, 1);
            }
            if ((*component).set_active)(self.component, 1) != K_RESULT_OK {
                return Err(Vst3HostingError::new("set_active_failed"));
            }
        }
        self.state = HostedInstanceState::Active;
        self.activated_sample_rate_hz = sample_rate_hz;
        self.activated_max_frames = max_frames;
        Ok(())
    }

    /// Deactivate an active instance (no-op tokened error when inactive).
    pub fn deactivate(&mut self) -> Result<(), Vst3HostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(Vst3HostingError::new("not_active"));
        }
        unsafe {
            let component = vtable_of::<ComponentVTable>(self.component);
            let _ = ((*component).set_active)(self.component, 0);
        }
        self.state = HostedInstanceState::Created;
        Ok(())
    }

    /// Build the raw process session for the sandbox audio thread. Only
    /// valid while active; the session preallocates its planar buffers at
    /// the activated max block size, so processing never allocates.
    pub fn process_session(&self) -> Result<Vst3ProcessSession, Vst3HostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(Vst3HostingError::new("not_active"));
        }
        Ok(Vst3ProcessSession::new(
            self.processor,
            self.activated_sample_rate_hz,
            self.activated_max_frames as usize,
            self.audio_bus_layout.clone(),
            Arc::clone(&self.param_changes),
            self.midi_cc_params.clone(),
        ))
    }
}
