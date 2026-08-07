use std::ptr;

use clap_sys::audio_buffer::clap_audio_buffer;

pub(crate) struct ClapAudioBusBuffers {
    samples: Vec<Vec<Vec<f32>>>,
    _channel_pointers: Vec<Vec<*mut f32>>,
    descriptors: Vec<clap_audio_buffer>,
}

impl ClapAudioBusBuffers {
    pub(crate) fn new(channel_counts: &[usize], max_frames: usize) -> Self {
        let mut samples = channel_counts
            .iter()
            .map(|&channel_count| vec![vec![0.0; max_frames]; channel_count])
            .collect::<Vec<_>>();
        let mut channel_pointers = samples
            .iter_mut()
            .map(|channels| {
                channels
                    .iter_mut()
                    .map(|channel| channel.as_mut_ptr())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let descriptors = channel_pointers
            .iter_mut()
            .map(|channels| clap_audio_buffer {
                data32: if channels.is_empty() {
                    ptr::null_mut()
                } else {
                    channels.as_mut_ptr()
                },
                data64: ptr::null_mut(),
                channel_count: channels.len() as u32,
                latency: 0,
                constant_mask: 0,
            })
            .collect();
        Self {
            samples,
            _channel_pointers: channel_pointers,
            descriptors,
        }
    }

    pub(crate) fn clear(&mut self, frames: usize) {
        for bus in &mut self.samples {
            for channel in bus {
                channel[..frames].fill(0.0);
            }
        }
    }

    pub(crate) fn copy_interleaved_stereo_into(
        &mut self,
        bus_index: usize,
        input: &[f32],
        frames: usize,
    ) {
        let Some(bus) = self.samples.get_mut(bus_index) else {
            return;
        };
        let [left, right, ..] = bus.as_mut_slice() else {
            return;
        };
        for frame in 0..frames {
            left[frame] = input[frame * 2];
            right[frame] = input[frame * 2 + 1];
        }
    }

    pub(crate) fn copy_interleaved_stereo_from(
        &self,
        bus_index: usize,
        output: &mut [f32],
        frames: usize,
    ) {
        let Some(bus) = self.samples.get(bus_index) else {
            return;
        };
        let [left, right, ..] = bus.as_slice() else {
            return;
        };
        for frame in 0..frames {
            output[frame * 2] = left[frame];
            output[frame * 2 + 1] = right[frame];
        }
    }

    pub(crate) fn as_ptr(&self) -> *const clap_audio_buffer {
        if self.descriptors.is_empty() {
            ptr::null()
        } else {
            self.descriptors.as_ptr()
        }
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut clap_audio_buffer {
        if self.descriptors.is_empty() {
            ptr::null_mut()
        } else {
            self.descriptors.as_mut_ptr()
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.descriptors.len() as u32
    }
}
