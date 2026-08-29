use std::ffi::c_void;
use std::sync::Arc;

use signal_plugin::{
    PluginEvent, PluginParamChange, PluginParamChangeQueue, PLUGIN_PARAM_CHANGE_CAPACITY,
};

use crate::vst3_host_adapter::hosting::Vst3HostingError;

use super::super::instance::Vst3AudioBusLayout;
use super::super::wire::*;
use super::buffers::Vst3AudioBusBuffers;

/// Raw, movable process handle for one activated VST3 instance: the
/// `IAudioProcessor` pointer plus planar stereo buffers preallocated at the
/// activated max block size. The sandbox moves this onto its audio thread;
/// the owning `Vst3HostedInstance` must outlive it and must not run
/// lifecycle transitions while the session is live. The per-block
/// `ProcessData`/`AudioBusBuffers` structs are stack-built from the
/// preallocated buffers, so processing never allocates.
pub struct Vst3ProcessSession {
    pub(crate) processor: *mut c_void,
    pub(crate) sample_rate_hz: f64,
    pub(crate) project_time_samples: i64,
    pub(crate) input_left: Vec<f32>,
    pub(crate) input_right: Vec<f32>,
    pub(crate) output_left: Vec<f32>,
    pub(crate) output_right: Vec<f32>,
    pub(crate) input_buses: Vst3AudioBusBuffers,
    pub(crate) output_buses: Vst3AudioBusBuffers,
    pub(crate) processing: bool,
    /// Pending param writes shared with the owning instance (g12.023).
    pub(crate) param_changes: Arc<PluginParamChangeQueue>,
    /// Drain scratch (preallocated; audio thread never allocates).
    pub(crate) param_scratch: Vec<PluginParamChange>,
    /// The host-side `IParameterChanges` rebuilt per block; boxed so the
    /// pointers handed to the plugin stay stable.
    pub(crate) input_changes: Box<HostParameterChanges>,
    /// The host-side input `IEventList` rebuilt per block (note events).
    pub(crate) input_events: Box<HostEventList>,
    /// Writable sinks for plugin-originated parameter changes and events.
    /// They are cleared each block; inspection does not consume them yet.
    pub(crate) output_changes: Box<HostParameterChanges>,
    pub(crate) output_events: Box<HostEventList>,
    /// CC → parameter assignments (`IMidiMapping`, queried at load); `None`
    /// drops CC events.
    pub(crate) midi_cc_params: Option<Arc<[Option<u32>; VST3_MIDI_CONTROLLER_COUNT]>>,
}

// Safety: the session is handed to exactly one audio thread;
// `setProcessing`/`process` are the VST3 processing-thread methods, and the
// owner serializes lifecycle against the session per the type contract.
unsafe impl Send for Vst3ProcessSession {}

impl Vst3ProcessSession {
    pub(crate) fn new(
        processor: *mut c_void,
        sample_rate_hz: f64,
        max_frames: usize,
        audio_bus_layout: Vst3AudioBusLayout,
        param_changes: Arc<PluginParamChangeQueue>,
        midi_cc_params: Option<Arc<[Option<u32>; VST3_MIDI_CONTROLLER_COUNT]>>,
    ) -> Self {
        Self {
            processor,
            sample_rate_hz,
            project_time_samples: 0,
            input_left: vec![0.0; max_frames],
            input_right: vec![0.0; max_frames],
            output_left: vec![0.0; max_frames],
            output_right: vec![0.0; max_frames],
            input_buses: Vst3AudioBusBuffers::new(
                &audio_bus_layout.input_channels,
                audio_bus_layout.main_input,
                max_frames,
            ),
            output_buses: Vst3AudioBusBuffers::new(
                &audio_bus_layout.output_channels,
                audio_bus_layout.main_output,
                max_frames,
            ),
            processing: false,
            param_changes,
            param_scratch: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            input_changes: HostParameterChanges::new(),
            input_events: HostEventList::new(),
            output_changes: HostParameterChanges::new(),
            output_events: HostEventList::new(),
            midi_cc_params,
        }
    }

    /// `setProcessing(true)` on the audio thread; must precede `process`.
    pub fn start(&mut self) -> Result<(), Vst3HostingError> {
        if self.processing {
            return Ok(());
        }
        let result = unsafe {
            let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
            ((*vtable).set_processing)(self.processor, 1)
        };
        // Plugins may answer kNotImplemented; only hard failures block.
        let _ = result;
        self.processing = true;
        Ok(())
    }

    /// `setProcessing(false)` on the audio thread.
    pub fn stop(&mut self) {
        if !self.processing {
            return;
        }
        unsafe {
            let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
            let _ = ((*vtable).set_processing)(self.processor, 0);
        }
        self.processing = false;
    }

    /// Whether `start()` has run and `stop()` has not yet.
    pub fn is_processing(&self) -> bool {
        self.processing
    }

    /// Run one block through `IAudioProcessor::process` using the
    /// preallocated planar buffers. Returns `false` on plugin error.
    ///
    /// # Safety
    /// `frames` must be within the preallocated buffer bounds (callers clamp).
    pub(crate) unsafe fn process_planar(&mut self, frames: usize, events: &[PluginEvent]) -> bool {
        // Drain pending param writes into the block's IParameterChanges
        // (block-boundary application, offset 0). Alloc-free.
        if self.param_changes.is_empty() {
            self.input_changes.clear();
        } else {
            self.param_changes.drain_coalesced(&mut self.param_scratch);
            self.input_changes.set_changes(&self.param_scratch);
        }
        // Note/CC delivery: notes ride the input IEventList; CC events map
        // through the load-time IMidiMapping table onto parameter-change
        // points carrying the event's intra-block sample offset (VST3's
        // input-CC contract). Unmapped or mapping-less CC drops silently —
        // there is nothing honest to send instead.
        self.input_events.clear();
        for event in events {
            match event {
                PluginEvent::Note(note) => self.input_events.push_note(note),
                PluginEvent::ControlChange(change) => {
                    if let Some(map) = &self.midi_cc_params {
                        if let Some(parameter_id) = map[usize::from(change.controller & 0x7F)] {
                            self.input_changes.push_point(
                                parameter_id,
                                change.offset_frames.min(i32::MAX as u32) as i32,
                                f64::from(change.value.clamp(0.0, 1.0)),
                            );
                        }
                    }
                }
                PluginEvent::Midi(midi) => {
                    let status = midi.status & 0xF0;
                    let (controller, value) = match status {
                        0xE0 => (
                            VST3_PITCH_BEND_CONTROLLER,
                            f64::from(
                                u16::from(midi.data1 & 0x7F) | (u16::from(midi.data2 & 0x7F) << 7),
                            ) / 16_383.0,
                        ),
                        0xD0 => (
                            VST3_AFTERTOUCH_CONTROLLER,
                            f64::from(midi.data1 & 0x7F) / 127.0,
                        ),
                        _ => continue,
                    };
                    if let Some(parameter_id) =
                        self.midi_cc_params.as_ref().and_then(|map| map[controller])
                    {
                        self.input_changes.push_point(
                            parameter_id,
                            midi.offset_frames.min(i32::MAX as u32) as i32,
                            value,
                        );
                    }
                }
                _ => {}
            }
        }
        self.output_changes.clear();
        self.output_events.clear();
        let input_parameter_changes =
            (&mut *self.input_changes as *mut HostParameterChanges).cast();
        let input_events = (&mut *self.input_events as *mut HostEventList).cast();
        let output_parameter_changes =
            (&mut *self.output_changes as *mut HostParameterChanges).cast();
        let output_events = (&mut *self.output_events as *mut HostEventList).cast();
        self.input_buses.clear(frames);
        self.input_buses
            .copy_main_from(&self.input_left, &self.input_right, frames);
        self.output_buses.clear(frames);
        let num_inputs = self.input_buses.len();
        let num_outputs = self.output_buses.len();
        let inputs = self.input_buses.as_mut_ptr();
        let outputs = self.output_buses.as_mut_ptr();
        let project_time_music = self.project_time_samples as f64 / self.sample_rate_hz * 2.0;
        let mut process_context = ProcessContext {
            state: K_PROJECT_TIME_MUSIC_VALID
                | K_TEMPO_VALID
                | K_BAR_POSITION_VALID
                | K_TIME_SIG_VALID
                | K_CONT_TIME_VALID,
            sample_rate: self.sample_rate_hz,
            project_time_samples: self.project_time_samples,
            system_time: 0,
            continuous_time_samples: self.project_time_samples,
            project_time_music,
            bar_position_music: (project_time_music / 4.0).floor() * 4.0,
            cycle_start_music: 0.0,
            cycle_end_music: 0.0,
            tempo: 120.0,
            time_sig_numerator: 4,
            time_sig_denominator: 4,
            chord: ProcessChord {
                key_note: 0,
                root_note: 0,
                chord_mask: 0,
            },
            smpte_offset_subframes: 0,
            frame_rate: ProcessFrameRate {
                frames_per_second: 0,
                flags: 0,
            },
            samples_to_next_clock: 0,
        };
        let mut data = ProcessData {
            process_mode: K_REALTIME,
            symbolic_sample_size: K_SAMPLE32,
            num_samples: frames as i32,
            num_inputs,
            num_outputs,
            inputs,
            outputs,
            input_parameter_changes,
            output_parameter_changes,
            input_events,
            output_events,
            process_context: (&mut process_context as *mut ProcessContext).cast(),
        };
        let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
        let result = ((*vtable).process)(self.processor, &mut data) == K_RESULT_OK;
        self.output_buses
            .copy_main_to(&mut self.output_left, &mut self.output_right, frames);
        self.project_time_samples += frames as i64;
        result
    }

    /// Process one block: interleaved stereo in, interleaved stereo out.
    /// Alloc-free (buffers preallocated at activate). On plugin error the
    /// input passes through unchanged. Returns `false` on error.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        let frames = frame_count
            .min(self.input_left.len())
            .min(input.len() / 2)
            .min(output.len() / 2);
        for frame in 0..frames {
            self.input_left[frame] = input[frame * 2];
            self.input_right[frame] = input[frame * 2 + 1];
        }
        if !unsafe { self.process_planar(frames, &[]) } {
            output[..frames * 2].copy_from_slice(&input[..frames * 2]);
            return false;
        }
        for frame in 0..frames {
            output[frame * 2] = self.output_left[frame];
            output[frame * 2 + 1] = self.output_right[frame];
        }
        true
    }

    /// In-place variant for the in-process isolation tier: processes the
    /// interleaved stereo buffer and writes the result back over it ONLY on
    /// success; on plugin error the buffer is left untouched (bypass
    /// semantics). Alloc-free. `true` = buffer transformed.
    pub fn process_in_place(&mut self, io: &mut [f32], frame_count: usize) -> bool {
        self.process_in_place_with_events(io, frame_count, &[])
    }

    /// [`Self::process_in_place`] with a per-block plugin event slice
    /// (sorted by `offset_frames`): note events ride the input
    /// `IEventList`; CC events become `IMidiMapping`-mapped parameter
    /// changes at their sample offsets. Alloc-free. `true` = buffer
    /// transformed.
    pub fn process_in_place_with_events(
        &mut self,
        io: &mut [f32],
        frame_count: usize,
        events: &[PluginEvent],
    ) -> bool {
        let frames = frame_count.min(self.input_left.len()).min(io.len() / 2);
        for frame in 0..frames {
            self.input_left[frame] = io[frame * 2];
            self.input_right[frame] = io[frame * 2 + 1];
        }
        if !unsafe { self.process_planar(frames, events) } {
            return false;
        }
        for frame in 0..frames {
            io[frame * 2] = self.output_left[frame];
            io[frame * 2 + 1] = self.output_right[frame];
        }
        true
    }
}
