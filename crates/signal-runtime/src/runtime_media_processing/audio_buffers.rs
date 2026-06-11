use signal_primitives::AudioBuffer;

pub(crate) fn hash_audio_buffer(buffer: &AudioBuffer) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for sample in buffer.samples() {
        hash ^= u64::from(sample.to_bits());
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= buffer.frames().0 as u64;
    hash = hash.wrapping_mul(1099511628211);
    hash ^= buffer.channel_count().0 as u64;
    hash
}

pub(crate) fn peak_abs(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
}
