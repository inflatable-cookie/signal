use std::{
    ptr,
    sync::{atomic::AtomicI64, Arc},
};

use clap_sys::plugin::clap_plugin;
use signal_plugin::{
    PluginAudioBusDirection, PluginParamChange, PluginParamChangeQueue,
    PLUGIN_PARAM_CHANGE_CAPACITY,
};

use crate::discovery::PluginAudioBusDescriptorList;

use super::super::entry::ClapHostingError;
use super::buffers::ClapAudioBusBuffers;
use super::events::{
    param_in_events_get, param_in_events_size, param_out_events_try_push, ParamEventList,
    ParamOutCapture,
};

/// Raw, movable process handle for one activated instance: the plugin
/// pointer plus preallocated planar audio-bus buffers. The sandbox moves this
/// onto its audio thread; the owning [`ClapHostedInstance`] must outlive it
/// and must not run lifecycle transitions while the session is live.
pub struct ClapProcessSession {
    pub(in crate::hosting::process) plugin: *const clap_plugin,
    pub(in crate::hosting::process) sample_rate_hz: f64,
    pub(in crate::hosting::process) input_buses: ClapAudioBusBuffers,
    pub(in crate::hosting::process) output_buses: ClapAudioBusBuffers,
    pub(in crate::hosting::process) main_input_bus: Option<usize>,
    pub(in crate::hosting::process) main_output_bus: usize,
    pub(in crate::hosting::process) max_frames: usize,
    pub(in crate::hosting::process) steady_time: AtomicI64,
    pub(in crate::hosting::process) processing: bool,
    /// Pending param writes shared with the owning instance (g12.023).
    pub(in crate::hosting::process) param_changes: Arc<PluginParamChangeQueue>,
    /// Drain scratch (preallocated; audio thread never allocates).
    pub(in crate::hosting::process) param_scratch: Vec<PluginParamChange>,
    /// The in-event list served to the plugin, rebuilt per block.
    pub(in crate::hosting::process) param_events: Box<ParamEventList>,
    /// The out-events capture served to the plugin (g12.024).
    pub(in crate::hosting::process) param_out: Box<ParamOutCapture>,
}

// Safety: the session is handed to exactly one audio thread; CLAP's process
// and start/stop_processing are audio-thread functions, and the owner
// serializes lifecycle against the session per the type contract above.
unsafe impl Send for ClapProcessSession {}

impl ClapProcessSession {
    pub(crate) fn new(
        plugin: *const clap_plugin,
        sample_rate_hz: f64,
        max_frames: usize,
        audio_buses: &PluginAudioBusDescriptorList,
        param_changes: Arc<PluginParamChangeQueue>,
        param_out_queue: Arc<PluginParamChangeQueue>,
    ) -> Self {
        let input_buses = audio_buses
            .iter()
            .filter(|bus| bus.direction == PluginAudioBusDirection::Input)
            .collect::<Vec<_>>();
        let output_buses = audio_buses
            .iter()
            .filter(|bus| bus.direction == PluginAudioBusDirection::Output)
            .collect::<Vec<_>>();
        let main_input_bus = input_buses.iter().position(|bus| bus.is_main);
        let main_output_bus = output_buses
            .iter()
            .position(|bus| bus.is_main)
            .expect("supported CLAP layouts always have a main output bus");
        let input_channel_counts = input_buses
            .iter()
            .map(|bus| usize::from(bus.channels))
            .collect::<Vec<_>>();
        let output_channel_counts = output_buses
            .iter()
            .map(|bus| usize::from(bus.channels))
            .collect::<Vec<_>>();
        let mut param_events = Box::new(ParamEventList {
            params: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            notes: Vec::with_capacity(super::events::IN_EVENT_CAPACITY),
            note_expressions: Vec::with_capacity(super::events::IN_EVENT_CAPACITY),
            midi: Vec::with_capacity(super::events::IN_EVENT_CAPACITY),
            order: Vec::with_capacity(
                PLUGIN_PARAM_CHANGE_CAPACITY + super::events::IN_EVENT_CAPACITY,
            ),
            list: clap_sys::events::clap_input_events {
                ctx: ptr::null_mut(),
                size: Some(param_in_events_size),
                get: Some(param_in_events_get),
            },
        });
        // Self-referential ctx: the list lives inside the box (stable
        // address) for the session's whole lifetime.
        param_events.list.ctx = (&mut *param_events as *mut ParamEventList).cast();
        let mut param_out = Box::new(ParamOutCapture {
            queue: param_out_queue,
            list: clap_sys::events::clap_output_events {
                ctx: ptr::null_mut(),
                try_push: Some(param_out_events_try_push),
            },
        });
        param_out.list.ctx = (&mut *param_out as *mut ParamOutCapture).cast();
        Self {
            plugin,
            sample_rate_hz,
            input_buses: ClapAudioBusBuffers::new(&input_channel_counts, max_frames),
            output_buses: ClapAudioBusBuffers::new(&output_channel_counts, max_frames),
            main_input_bus,
            main_output_bus,
            max_frames,
            steady_time: AtomicI64::new(0),
            processing: false,
            param_changes,
            param_scratch: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            param_events,
            param_out,
        }
    }

    /// `start_processing` on the audio thread; must precede `process`.
    pub fn start(&mut self) -> Result<(), ClapHostingError> {
        if self.processing {
            return Ok(());
        }
        let ok = unsafe {
            (*self.plugin)
                .start_processing
                .map(|start| start(self.plugin))
                .unwrap_or(true)
        };
        if !ok {
            return Err(ClapHostingError::new("start_processing_failed"));
        }
        self.processing = true;
        Ok(())
    }

    /// `stop_processing` on the audio thread.
    pub fn stop(&mut self) {
        if !self.processing {
            return;
        }
        if let Some(stop) = unsafe { (*self.plugin).stop_processing } {
            unsafe { stop(self.plugin) };
        }
        self.processing = false;
    }

    /// Whether `start()` has succeeded and `stop()` has not yet run.
    pub fn is_processing(&self) -> bool {
        self.processing
    }
}
