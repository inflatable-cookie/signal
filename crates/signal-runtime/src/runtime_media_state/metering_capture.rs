use super::*;

impl RuntimeMeteringStateModel {
    pub(crate) fn snapshot(&self) -> RuntimeMeteringSnapshot {
        self.snapshot.clone()
    }

    pub(crate) fn capture(
        &mut self,
        sample_rate_hz: u32,
        output: &AudioBuffer,
        meter_sources: Vec<RuntimeMeterSourceSnapshot>,
    ) {
        let samples = output.samples();
        let sample_count = samples.len();
        let mean_square = if sample_count == 0 {
            0.0
        } else {
            samples
                .iter()
                .copied()
                .map(|sample| {
                    let sample = f64::from(sample);
                    sample * sample
                })
                .sum::<f64>()
                / sample_count as f64
        };
        let clipped_samples = samples
            .iter()
            .filter(|sample| sample.abs() >= 0.999)
            .count() as u64;
        self.clipped_sample_count = self.clipped_sample_count.saturating_add(clipped_samples);
        self.integrated_sum += mean_square * sample_count as f64;
        self.integrated_sample_count = self
            .integrated_sample_count
            .saturating_add(sample_count.min(u64::MAX as usize) as u64);
        Self::push_window_block(
            &mut self.momentary_blocks,
            &mut self.momentary_sum,
            &mut self.momentary_sample_count,
            RuntimeMeteringWindowBlock {
                mean_square,
                sample_count,
            },
            Self::window_sample_target(sample_rate_hz, output.channel_count().0, 0.4),
        );
        Self::push_window_block(
            &mut self.short_term_blocks,
            &mut self.short_term_sum,
            &mut self.short_term_sample_count,
            RuntimeMeteringWindowBlock {
                mean_square,
                sample_count,
            },
            Self::window_sample_target(sample_rate_hz, output.channel_count().0, 3.0),
        );

        let meters = meter_sources;
        let main_output = meters.iter().find(|meter| meter.bus_id == "main:out");

        self.snapshot = RuntimeMeteringSnapshot {
            meter_count: meters.len(),
            main_output_peak_level: main_output.map(|meter| meter.peak_level),
            main_output_rms_level: main_output.map(|meter| meter.rms_level),
            momentary_loudness_lufs: Self::lufs_from_weighted_sum(
                self.momentary_sum,
                self.momentary_sample_count,
            ),
            short_term_loudness_lufs: Self::lufs_from_weighted_sum(
                self.short_term_sum,
                self.short_term_sample_count,
            ),
            integrated_loudness_lufs: Self::lufs_from_weighted_sum_u64(
                self.integrated_sum,
                self.integrated_sample_count,
            ),
            clipped_sample_count: self.clipped_sample_count,
            track_lanes: Vec::new(),
            bus_groups: Vec::new(),
            console_groups: Vec::new(),
            send_returns: Vec::new(),
            bus_connection_count: 0,
            auxiliary_path_count: 0,
            bus_connections: Vec::new(),
            auxiliary_paths: Vec::new(),
            summary: format!(
                "meters={} main_peak={:?} main_rms={:?} momentary_lufs={:?} short_term_lufs={:?} integrated_lufs={:?} clipped={}",
                meters.len(),
                main_output.map(|meter| meter.peak_level),
                main_output.map(|meter| meter.rms_level),
                Self::lufs_from_weighted_sum(self.momentary_sum, self.momentary_sample_count),
                Self::lufs_from_weighted_sum(self.short_term_sum, self.short_term_sample_count),
                Self::lufs_from_weighted_sum_u64(
                    self.integrated_sum,
                    self.integrated_sample_count,
                ),
                self.clipped_sample_count,
            ),
            meters,
        };
    }

    fn push_window_block(
        window: &mut VecDeque<RuntimeMeteringWindowBlock>,
        sum: &mut f64,
        sample_count: &mut usize,
        block: RuntimeMeteringWindowBlock,
        target_samples: usize,
    ) {
        *sum += block.mean_square * block.sample_count as f64;
        *sample_count = sample_count.saturating_add(block.sample_count);
        window.push_back(block);
        while *sample_count > target_samples.max(1) {
            let Some(removed) = window.pop_front() else {
                break;
            };
            *sum -= removed.mean_square * removed.sample_count as f64;
            *sample_count = sample_count.saturating_sub(removed.sample_count);
        }
    }

    fn window_sample_target(sample_rate_hz: u32, channel_count: usize, seconds: f64) -> usize {
        ((sample_rate_hz as f64) * seconds).round() as usize * channel_count.max(1)
    }

    fn lufs_from_weighted_sum(sum: f64, sample_count: usize) -> Option<f32> {
        if sample_count == 0 {
            return None;
        }
        Self::lufs_from_mean_square(sum / sample_count as f64)
    }

    fn lufs_from_weighted_sum_u64(sum: f64, sample_count: u64) -> Option<f32> {
        if sample_count == 0 {
            return None;
        }
        Self::lufs_from_mean_square(sum / sample_count as f64)
    }

    fn lufs_from_mean_square(mean_square: f64) -> Option<f32> {
        if mean_square <= f64::EPSILON {
            None
        } else {
            Some((10.0 * mean_square.log10()) as f32)
        }
    }
}

impl Default for RuntimeMeteringStateModel {
    fn default() -> Self {
        Self {
            snapshot: RuntimeMeteringSnapshot {
                meter_count: 0,
                main_output_peak_level: None,
                main_output_rms_level: None,
                momentary_loudness_lufs: None,
                short_term_loudness_lufs: None,
                integrated_loudness_lufs: None,
                clipped_sample_count: 0,
                meters: Vec::new(),
                track_lanes: Vec::new(),
                bus_groups: Vec::new(),
                console_groups: Vec::new(),
                send_returns: Vec::new(),
                bus_connection_count: 0,
                auxiliary_path_count: 0,
                bus_connections: Vec::new(),
                auxiliary_paths: Vec::new(),
                summary: "meters=0 main_peak=None main_rms=None momentary_lufs=None short_term_lufs=None integrated_lufs=None clipped=0".to_string(),
            },
            momentary_blocks: VecDeque::new(),
            short_term_blocks: VecDeque::new(),
            momentary_sum: 0.0,
            short_term_sum: 0.0,
            momentary_sample_count: 0,
            short_term_sample_count: 0,
            integrated_sum: 0.0,
            integrated_sample_count: 0,
            clipped_sample_count: 0,
        }
    }
}
