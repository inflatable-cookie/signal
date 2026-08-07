use std::ptr;

use super::super::wire::AudioBusBuffers;

pub(crate) struct Vst3AudioBusBuffers {
    pub(crate) _channel_samples: Vec<Vec<Box<[f32]>>>,
    pub(crate) _channel_pointers: Vec<Box<[*mut f32]>>,
    pub(crate) descriptors: Vec<AudioBusBuffers>,
    pub(crate) main_index: Option<usize>,
}

impl Vst3AudioBusBuffers {
    pub(crate) fn new(
        channel_counts: &[u16],
        main_index: Option<usize>,
        max_frames: usize,
    ) -> Self {
        // The SDK permits null sample addresses for inactive buses, but some
        // multi-output frameworks still render every declared bus. Back all
        // channels with discardable scratch so those plugins remain safe.
        let mut channel_samples = channel_counts
            .iter()
            .map(|channels| {
                (0..usize::from(*channels))
                    .map(|_| vec![0.0; max_frames].into_boxed_slice())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let flat_pointers = channel_samples
            .iter_mut()
            .flat_map(|channels| channels.iter_mut().map(|samples| samples.as_mut_ptr()))
            .collect::<Vec<_>>();
        let mut channel_offset = 0;
        let mut channel_pointers = channel_counts
            .iter()
            .map(|channels| {
                let own_start = channel_offset;
                let own_end = own_start + usize::from(*channels);
                channel_offset = own_end;
                flat_pointers[own_start..own_end]
                    .iter()
                    .chain(flat_pointers[..own_start].iter())
                    .chain(flat_pointers[own_end..].iter())
                    .copied()
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .collect::<Vec<_>>();
        let descriptors = channel_pointers
            .iter_mut()
            .zip(channel_counts)
            .map(|(channels, channel_count)| AudioBusBuffers {
                // `channel_pointers` deliberately carries fallback pointers
                // after this bus's own channels, but the VST3 descriptor must
                // still advertise this bus's declared channel count. Using
                // the backing slice length here makes every bus appear to
                // contain the total channels across the plugin, which breaks
                // multi-output instruments such as Kontakt.
                num_channels: i32::from(*channel_count),
                silence_flags: 0,
                channel_buffers32: channels.as_mut_ptr(),
            })
            .collect();
        Self {
            _channel_samples: channel_samples,
            _channel_pointers: channel_pointers,
            descriptors,
            main_index,
        }
    }

    pub(crate) fn copy_main_from(&mut self, left: &[f32], right: &[f32], frames: usize) {
        let Some(index) = self.main_index else {
            return;
        };
        let channels = &mut self._channel_samples[index];
        if channels.len() >= 2 {
            channels[0][..frames].copy_from_slice(&left[..frames]);
            channels[1][..frames].copy_from_slice(&right[..frames]);
        }
    }

    pub(crate) fn copy_main_to(&self, left: &mut [f32], right: &mut [f32], frames: usize) {
        let Some(index) = self.main_index else {
            return;
        };
        let channels = &self._channel_samples[index];
        if channels.len() >= 2 {
            left[..frames].copy_from_slice(&channels[0][..frames]);
            right[..frames].copy_from_slice(&channels[1][..frames]);
        }
    }

    pub(crate) fn clear(&mut self, frames: usize) {
        for bus in &mut self._channel_samples {
            for channel in bus {
                channel[..frames].fill(0.0);
            }
        }
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AudioBusBuffers {
        if self.descriptors.is_empty() {
            ptr::null_mut()
        } else {
            self.descriptors.as_mut_ptr()
        }
    }

    pub(crate) fn len(&self) -> i32 {
        self.descriptors.len() as i32
    }
}
