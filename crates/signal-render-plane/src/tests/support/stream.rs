use super::super::*;

/// Feed `[from, to)` of a ramp (value = frame / total) in fixed chunks.
pub(in crate::tests) fn feed_ramp(
    feeder: &StreamFeeder,
    total: u64,
    from: u64,
    to: u64,
    chunk_frames: u64,
) {
    let mut start = from - from % chunk_frames;
    while start < to.min(total) {
        let count = chunk_frames.min(total - start);
        let mut data = Vec::with_capacity(count as usize * 2);
        for frame in start..start + count {
            let value = frame as f32 / total as f32;
            data.push(value);
            data.push(value);
        }
        if feeder
            .try_send_chunk(StreamChunk {
                start_frame: start,
                frames: data.into(),
            })
            .is_err()
        {
            return; // Mailbox full: enough read-ahead for the test.
        }
        start += count;
    }
}

/// Push `count` stereo frames of a ramp starting at `value_base`
/// (value = (value_base + i) / 10_000) and return the next base.
pub(in crate::tests) fn push_ramp(feeder: &LiveInputFeeder, value_base: u64, count: usize) -> u64 {
    let mut data = Vec::with_capacity(count * 2);
    for index in 0..count {
        let value = (value_base + index as u64) as f32 / 10_000.0;
        data.push(value);
        data.push(value);
    }
    assert_eq!(feeder.push_slice(&data), count, "test ring overflowed");
    value_base + count as u64
}
