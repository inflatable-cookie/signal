use hound::{SampleFormat as HoundSampleFormat, WavSpec, WavWriter};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn unique_test_path(label: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!("signal-host-local-{label}-{nanos}.{extension}"))
}

pub(crate) fn temp_artifact_dir(label: &str) -> PathBuf {
    let path = unique_test_path(label, "dir");
    let _ = fs::create_dir_all(&path);
    path
}

pub(crate) fn write_test_wav(path: &Path) {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: HoundSampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec).expect("create wav");
    for index in 0..128 {
        let sample = ((index as f32 / 127.0) * i16::MAX as f32 * 0.5) as i16;
        writer.write_sample(sample).expect("write wav sample");
    }
    writer.finalize().expect("finalize wav");
}
