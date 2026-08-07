//! Audio processing structs and IAudioProcessor vtable.

use std::ffi::c_void;

use super::super::com::*;

/// `Steinberg::Vst::ProcessSetup`.
#[repr(C)]
pub(crate) struct ProcessSetup {
    pub(crate) process_mode: i32,
    pub(crate) symbolic_sample_size: i32,
    pub(crate) max_samples_per_block: i32,
    pub(crate) sample_rate: f64,
}

/// `Steinberg::Vst::AudioBusBuffers` (32-bit float member of the union).
#[repr(C)]
pub(crate) struct AudioBusBuffers {
    pub(crate) num_channels: i32,
    pub(crate) silence_flags: u64,
    pub(crate) channel_buffers32: *mut *mut f32,
}

/// `Steinberg::Vst::ProcessData` (input parameter changes live per
/// g12.023; event queues still null).
#[repr(C)]
pub(crate) struct ProcessData {
    pub(crate) process_mode: i32,
    pub(crate) symbolic_sample_size: i32,
    pub(crate) num_samples: i32,
    pub(crate) num_inputs: i32,
    pub(crate) num_outputs: i32,
    pub(crate) inputs: *mut AudioBusBuffers,
    pub(crate) outputs: *mut AudioBusBuffers,
    pub(crate) input_parameter_changes: *mut c_void,
    pub(crate) output_parameter_changes: *mut c_void,
    pub(crate) input_events: *mut c_void,
    pub(crate) output_events: *mut c_void,
    pub(crate) process_context: *mut c_void,
}

/// `Steinberg::Vst::Chord`.
#[repr(C)]
pub(crate) struct ProcessChord {
    pub(crate) key_note: u8,
    pub(crate) root_note: u8,
    pub(crate) chord_mask: i16,
}

/// `Steinberg::Vst::FrameRate`.
#[repr(C)]
pub(crate) struct ProcessFrameRate {
    pub(crate) frames_per_second: u32,
    pub(crate) flags: u32,
}

/// `Steinberg::Vst::ProcessContext`.
#[repr(C)]
pub(crate) struct ProcessContext {
    pub(crate) state: u32,
    pub(crate) sample_rate: f64,
    pub(crate) project_time_samples: i64,
    pub(crate) system_time: i64,
    pub(crate) continuous_time_samples: i64,
    pub(crate) project_time_music: f64,
    pub(crate) bar_position_music: f64,
    pub(crate) cycle_start_music: f64,
    pub(crate) cycle_end_music: f64,
    pub(crate) tempo: f64,
    pub(crate) time_sig_numerator: i32,
    pub(crate) time_sig_denominator: i32,
    pub(crate) chord: ProcessChord,
    pub(crate) smpte_offset_subframes: i32,
    pub(crate) frame_rate: ProcessFrameRate,
    pub(crate) samples_to_next_clock: i32,
}

/// `IAudioProcessor`.
#[repr(C)]
pub(crate) struct AudioProcessorVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) set_bus_arrangements:
        unsafe extern "C" fn(*mut c_void, *mut u64, i32, *mut u64, i32) -> Tresult,
    pub(crate) get_bus_arrangement:
        unsafe extern "C" fn(*mut c_void, i32, i32, *mut u64) -> Tresult,
    pub(crate) can_process_sample_size: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
    pub(crate) get_latency_samples: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) setup_processing: unsafe extern "C" fn(*mut c_void, *mut ProcessSetup) -> Tresult,
    pub(crate) set_processing: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    pub(crate) process: unsafe extern "C" fn(*mut c_void, *mut ProcessData) -> Tresult,
    pub(crate) get_tail_samples: unsafe extern "C" fn(*mut c_void) -> u32,
}
