use super::support::*;
use super::*;

#[test]
fn int16_dither_round_trip_stays_within_a_lsb_and_decorrelates() {
    let dir = std::env::temp_dir().join(format!(
        "render-plane-offline-dither-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("dither.wav");

    // A slow ramp plus a long constant plateau: the plateau exposes
    // dither decorrelation, the ramp exercises quantization accuracy.
    let mut samples = Vec::new();
    for index in 0..4_000 {
        samples.push((index as f32 / 4_000.0) * 0.5 - 0.25);
    }
    samples.extend(std::iter::repeat_n(0.000_02f32, 4_000));
    write_wav(&path, &samples, 1, 48_000, WavBitDepth::Int16).unwrap();

    let mut reader = hound::WavReader::open(&path).unwrap();
    assert_eq!(reader.spec().bits_per_sample, 16);
    let decoded: Vec<i16> = reader.samples::<i16>().map(Result::unwrap).collect();
    assert_eq!(decoded.len(), samples.len());
    let lsb = 1.0 / 32_768.0;
    for (index, (source, quantized)) in samples.iter().zip(decoded.iter()).enumerate() {
        let restored = *quantized as f32 / 32_768.0;
        assert!(
            (restored - source).abs() <= 1.5 * lsb,
            "sample {index} drifted past 1.5 LSB: {source} -> {restored}",
        );
    }
    // The constant plateau sits between integer codes; TPDF dither must
    // toggle adjacent codes rather than collapsing to one value.
    let plateau = &decoded[4_000..];
    assert!(
        plateau.iter().any(|value| *value != plateau[0]),
        "dithered constant plateau quantized to a single code",
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn float32_wav_round_trips_bit_exactly() {
    let dir = std::env::temp_dir().join(format!("render-plane-offline-f32-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("float.wav");
    let samples: Vec<f32> = (0..512).map(|index| (index as f32 * 0.01).sin()).collect();
    write_wav(&path, &samples, 2, 44_100, WavBitDepth::Float32).unwrap();
    let mut reader = hound::WavReader::open(&path).unwrap();
    assert_eq!(reader.spec().sample_rate, 44_100);
    assert_eq!(reader.spec().channels, 2);
    let decoded: Vec<f32> = reader.samples::<f32>().map(Result::unwrap).collect();
    assert_eq!(decoded.len(), samples.len());
    assert!(samples
        .iter()
        .zip(decoded.iter())
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn bounce_starts_at_full_level_with_no_transport_fade_in() {
    // A constant-amplitude source mid-clip: the first exported sample
    // must already be at full level. With the realtime edge envelope a
    // 5 ms fade-in would zero the first sample.
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![lane(1, 1.0, vec![constant_clip(11, 0.5)]), master(vec![1])],
    };
    let options = OfflineRenderOptions {
        start_frame: 4_800, // Mid-clip: past the clip edge declick fade.
        frame_count: 256,
        ..OfflineRenderOptions::default()
    };
    let output = render_plan_to_pcm(&spec, &options).unwrap();
    assert!(
        (output.master[0] - 0.5).abs() < 1e-6,
        "first bounce sample read {} — transport fade-in leaked into the export",
        output.master[0],
    );
    assert!(output
        .master
        .iter()
        .all(|sample| (sample - 0.5).abs() < 1e-6));
}
