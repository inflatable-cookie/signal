use std::sync::atomic::Ordering;

use clap_sys::process::{clap_process, CLAP_PROCESS_ERROR};
use signal_plugin::PluginEvent;

use super::session::ClapProcessSession;

impl ClapProcessSession {
    /// Process one block: optional interleaved stereo in, stereo out.
    /// Alloc-free (buffers preallocated at activate). On plugin error the
    /// input passes through unchanged. Returns `false` on error.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        let frames = frame_count
            .min(self.max_frames)
            .min(input.len() / 2)
            .min(output.len() / 2);
        let in_events = self.prepare_in_events(&[]);
        self.input_buses.clear(frames);
        self.output_buses.clear(frames);
        if let Some(main_input_bus) = self.main_input_bus {
            self.input_buses
                .copy_interleaved_stereo_into(main_input_bus, input, frames);
        }
        let steady_time = self.steady_time.load(Ordering::Relaxed);
        let audio_inputs = self.input_buses.as_ptr();
        let audio_inputs_count = self.input_buses.len();
        let audio_outputs = self.output_buses.as_mut_ptr();
        let audio_outputs_count = self.output_buses.len();
        let transport = self.transport(steady_time);
        let process = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: &transport,
            audio_inputs,
            audio_outputs,
            audio_inputs_count,
            audio_outputs_count,
            in_events,
            out_events: &self.param_out.list,
        };
        self.steady_time
            .store(steady_time + frames as i64, Ordering::Relaxed);

        let status = unsafe {
            (*self.plugin)
                .process
                .map(|process_fn| process_fn(self.plugin, &process))
                .unwrap_or(CLAP_PROCESS_ERROR)
        };
        if status == CLAP_PROCESS_ERROR {
            output[..frames * 2].copy_from_slice(&input[..frames * 2]);
            return false;
        }
        self.output_buses
            .copy_interleaved_stereo_from(self.main_output_bus, output, frames);
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
    /// (sorted by `offset_frames`): note events map to CLAP note in-events
    /// (float velocity preserved), CC events downconvert to 3-byte MIDI at
    /// this boundary. Alloc-free. `true` = buffer transformed.
    pub fn process_in_place_with_events(
        &mut self,
        io: &mut [f32],
        frame_count: usize,
        events: &[PluginEvent],
    ) -> bool {
        let frames = frame_count.min(self.max_frames).min(io.len() / 2);
        let in_events = self.prepare_in_events(events);
        self.input_buses.clear(frames);
        self.output_buses.clear(frames);
        if let Some(main_input_bus) = self.main_input_bus {
            self.input_buses
                .copy_interleaved_stereo_into(main_input_bus, io, frames);
        }
        let steady_time = self.steady_time.load(Ordering::Relaxed);
        let audio_inputs = self.input_buses.as_ptr();
        let audio_inputs_count = self.input_buses.len();
        let audio_outputs = self.output_buses.as_mut_ptr();
        let audio_outputs_count = self.output_buses.len();
        let transport = self.transport(steady_time);
        let process = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: &transport,
            audio_inputs,
            audio_outputs,
            audio_inputs_count,
            audio_outputs_count,
            in_events,
            out_events: &self.param_out.list,
        };
        self.steady_time
            .store(steady_time + frames as i64, Ordering::Relaxed);

        let status = unsafe {
            (*self.plugin)
                .process
                .map(|process_fn| process_fn(self.plugin, &process))
                .unwrap_or(CLAP_PROCESS_ERROR)
        };
        if status == CLAP_PROCESS_ERROR {
            return false;
        }
        self.output_buses
            .copy_interleaved_stereo_from(self.main_output_bus, io, frames);
        true
    }
}
