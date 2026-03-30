use super::*;

pub(super) fn temp_media_path(label: &str, extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be monotonic enough for temp files")
        .as_nanos();
    let sequence = TEMP_PATH_COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!(
        "signal-runtime-{label}-{nonce}-{sequence}.{extension}"
    ))
}

pub(super) fn temp_capture_path(label: &str) -> PathBuf {
    temp_media_path(label, "wav")
}

pub(super) fn temp_artifact_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be monotonic enough for temp dirs")
        .as_nanos();
    let sequence = TEMP_PATH_COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!("signal-runtime-{label}-{nonce}-{sequence}"))
}

pub(super) fn write_test_wav(path: &Path) {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: HoundSampleFormat::Float,
    };
    let mut writer = WavWriter::create(path, spec).expect("test wav should be created");
    for frame in 0..128 {
        let sample = ((frame as f32 / 128.0) * 2.0) - 1.0;
        writer
            .write_sample(sample)
            .expect("test wav sample should be written");
    }
    writer.finalize().expect("test wav should finalize");
}

pub(super) fn write_transient_test_wav(path: &Path) {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: HoundSampleFormat::Float,
    };
    let mut writer = WavWriter::create(path, spec).expect("test wav should be created");
    for frame in 0..48_000 {
        let sample = if frame % 6_000 == 0 { 1.0 } else { 0.0 };
        writer
            .write_sample(sample)
            .expect("test wav sample should be written");
    }
    writer.finalize().expect("test wav should finalize");
}

pub(super) fn write_test_aiff(path: &Path) {
    use std::io::Write;

    let frames = 128u32;
    let sample_rate_extended = [0x40, 0x0E, 0xBB, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let samples = (0..frames)
        .map(|frame| {
            let sample = ((frame as f32 / 128.0) * 2.0) - 1.0;
            (sample * i16::MAX as f32) as i16
        })
        .collect::<Vec<_>>();
    let data_size = samples.len() as u32 * 2;
    let ssnd_size = 8 + data_size;
    let form_size = 4 + (8 + 18) + (8 + ssnd_size);
    let mut file = fs::File::create(path).expect("test aiff should be created");
    file.write_all(b"FORM").expect("write FORM");
    file.write_all(&form_size.to_be_bytes())
        .expect("write FORM size");
    file.write_all(b"AIFF").expect("write AIFF signature");
    file.write_all(b"COMM").expect("write COMM");
    file.write_all(&18u32.to_be_bytes())
        .expect("write COMM size");
    file.write_all(&1u16.to_be_bytes())
        .expect("write channel count");
    file.write_all(&frames.to_be_bytes())
        .expect("write frame count");
    file.write_all(&16u16.to_be_bytes())
        .expect("write sample size");
    file.write_all(&sample_rate_extended)
        .expect("write sample rate");
    file.write_all(b"SSND").expect("write SSND");
    file.write_all(&ssnd_size.to_be_bytes())
        .expect("write SSND size");
    file.write_all(&0u32.to_be_bytes()).expect("write offset");
    file.write_all(&0u32.to_be_bytes())
        .expect("write block size");
    for sample in samples {
        file.write_all(&sample.to_be_bytes())
            .expect("write AIFF sample");
    }
}
