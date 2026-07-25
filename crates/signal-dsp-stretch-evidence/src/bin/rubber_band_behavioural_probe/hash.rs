pub(crate) fn hash_samples(samples: &[f32]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for sample in samples {
        hash_u64(&mut hash, u64::from(sample.to_bits()));
    }
    hash
}

pub(super) fn hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
