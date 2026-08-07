//! VST3 bus / port layout helpers for hosted instances.

use std::ffi::c_void;
use std::ptr;

use super::super::wire::*;

/// Main-bus stereo port layout summary for a hosted VST3 instance (mirrors
/// `ClapHostedPortLayout`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vst3HostedPortLayout {
    /// Channel count of the main audio input bus (0 = none).
    pub main_input_channels: u16,
    /// Channel count of the main audio output bus (0 = none).
    pub main_output_channels: u16,
}

#[derive(Clone, Debug)]
pub(crate) struct Vst3AudioBusLayout {
    pub(crate) input_channels: Vec<u16>,
    pub(crate) output_channels: Vec<u16>,
    pub(crate) main_input: Option<usize>,
    pub(crate) main_output: Option<usize>,
}

impl Vst3AudioBusLayout {
    pub(crate) fn port_layout(&self) -> Vst3HostedPortLayout {
        Vst3HostedPortLayout {
            main_input_channels: self
                .main_input
                .map(|index| self.input_channels[index])
                .unwrap_or(0),
            main_output_channels: self
                .main_output
                .map(|index| self.output_channels[index])
                .unwrap_or(0),
        }
    }
}

/// Lifecycle state of a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostedInstanceState {
    Created,
    Active,
}

impl Vst3HostedPortLayout {
    /// Phase 1 supports exactly a stereo main in + stereo main out effect.
    pub fn is_stereo_effect(&self) -> bool {
        self.main_input_channels == 2 && self.main_output_channels == 2
    }

    /// MIDI instrument layout supported by the current host: no main audio
    /// input and one stereo main output.
    pub fn is_stereo_instrument(&self) -> bool {
        self.main_input_channels == 0 && self.main_output_channels == 2
    }

    /// Whether the current stereo process session can host this layout.
    pub fn is_supported_stereo_processor(&self) -> bool {
        self.is_stereo_effect() || self.is_stereo_instrument()
    }
}

/// Read every declared audio bus while identifying the main bus in each
/// direction. ProcessData must retain this complete topology even when only
/// the main buses are active.
pub(crate) unsafe fn audio_bus_layout(component: *mut c_void) -> Vst3AudioBusLayout {
    let vtable = vtable_of::<ComponentVTable>(component);
    let mut layout = Vst3AudioBusLayout {
        input_channels: Vec::new(),
        output_channels: Vec::new(),
        main_input: None,
        main_output: None,
    };
    for (direction, channels, main) in [
        (K_INPUT, &mut layout.input_channels, &mut layout.main_input),
        (
            K_OUTPUT,
            &mut layout.output_channels,
            &mut layout.main_output,
        ),
    ] {
        let count = ((*vtable).get_bus_count)(component, K_AUDIO, direction).max(0);
        for index in 0..count {
            let mut info = BusInfo::zeroed();
            if ((*vtable).get_bus_info)(component, K_AUDIO, direction, index, &mut info)
                != K_RESULT_OK
            {
                channels.push(0);
                continue;
            }
            channels.push(info.channel_count.clamp(0, u16::MAX as i32) as u16);
            *main = select_main_bus(*main, info.bus_type, index as usize);
        }
        if main.is_none() && !channels.is_empty() {
            *main = Some(0);
        }
    }
    layout
}

pub(crate) fn select_main_bus(
    current: Option<usize>,
    bus_type: i32,
    index: usize,
) -> Option<usize> {
    current.or_else(|| (bus_type == K_MAIN).then_some(index))
}

pub(crate) unsafe fn bus_arrangements(
    processor: *mut c_void,
    direction: i32,
    channel_counts: &[u16],
) -> Vec<u64> {
    let vtable = vtable_of::<AudioProcessorVTable>(processor);
    channel_counts
        .iter()
        .enumerate()
        .map(|(index, channels)| {
            let mut arrangement = 0;
            if ((*vtable).get_bus_arrangement)(processor, direction, index as i32, &mut arrangement)
                == K_RESULT_OK
            {
                arrangement
            } else if *channels == 2 {
                STEREO_ARRANGEMENT
            } else {
                0
            }
        })
        .collect()
}

pub(crate) fn pointer_or_null(values: &mut [u64]) -> *mut u64 {
    if values.is_empty() {
        ptr::null_mut()
    } else {
        values.as_mut_ptr()
    }
}
