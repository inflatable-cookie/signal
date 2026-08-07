use super::support::*;
use super::*;

#[test]
fn offline_render_drives_stage_processors_in_offline_waiting() {
    let backend = Arc::new(OfflineOnlyGainProcessor::default());
    let processor = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut sum = master(vec![1]);
    sum.stage_id = 2;
    sum.kind = RenderStageKind::Sum;
    sum.processor = Some(processor.clone());
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(1, 1.0, vec![constant_clip(11, 1.0)]),
            sum,
            master(vec![2]),
        ],
    };
    let options = OfflineRenderOptions {
        start_frame: 0,
        frame_count: 2_048,
        block_frames: 128,
        capture_stage_ids: Vec::new(),
    };

    let rendered = render_plan_to_pcm(&spec, &options).expect("offline render");

    assert_eq!(
        backend
            .bypassed_blocks
            .load(std::sync::atomic::Ordering::Relaxed),
        0,
        "no block may bypass the insert during an offline render",
    );
    // Past the clip edge fade the source is a 1.0 plateau, so the insert
    // is audible as an exact halving on every remaining sample.
    let guard = 256 * 2;
    assert!(rendered.master.len() > guard);
    for (index, sample) in rendered.master.iter().enumerate().skip(guard) {
        assert!(
            (sample - 0.5).abs() < 1e-6,
            "sample {index}: {sample} (insert dropped for this block)",
        );
    }

    // Restored, not left latched: the same handle may be live on the
    // audio thread after the bounce, where the realtime bound is correct.
    assert!(
        !backend.offline.load(std::sync::atomic::Ordering::Relaxed),
        "offline waiting must be restored when the render ends",
    );
}
#[test]
fn offline_render_is_sample_identical_to_a_manual_executor_loop() {
    // Identity gate: render_plan_to_pcm and a hand-rolled
    // controller/executor loop over the same spec and block size must
    // produce byte-identical PCM. Same code path today (this is the
    // point of WYSIWYG bounce); the test exists to catch any future
    // offline-only divergence.
    let spec = reference_spec();
    let options = OfflineRenderOptions {
        start_frame: 960,
        frame_count: 48_000,
        block_frames: 512,
        capture_stage_ids: Vec::new(),
    };
    let output = render_plan_to_pcm(&spec, &options).unwrap();

    let (mut controller, mut executor) = render_plane();
    controller.set_stream_channels(2).unwrap();
    controller.install_plan(&spec).unwrap();
    controller.seek(options.start_frame).unwrap();
    controller.set_playing(true).unwrap();
    executor.drain_commands();
    executor.set_edge_gain_immediate(1.0);
    let mut manual = Vec::new();
    let mut block = vec![0.0f32; 512 * 2];
    let mut remaining = options.frame_count as usize;
    while remaining > 0 {
        let frames_this_block = remaining.min(512);
        let slice = &mut block[..frames_this_block * 2];
        executor.render_block(slice);
        manual.extend_from_slice(slice);
        remaining -= frames_this_block;
    }

    assert_eq!(output.master.len(), manual.len());
    assert!(
        output
            .master
            .iter()
            .zip(manual.iter())
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "offline driver diverged from the manual executor loop",
    );
    assert_eq!(output.channels, 2);
    assert_eq!(output.sample_rate_hz, 48_000);
}

#[test]
fn offline_render_completes_ten_seconds_faster_than_realtime() {
    let spec = reference_spec();
    let options = OfflineRenderOptions {
        frame_count: 480_000, // 10 s at 48 kHz.
        ..OfflineRenderOptions::default()
    };
    let started = std::time::Instant::now();
    let output = render_plan_to_pcm(&spec, &options).unwrap();
    let elapsed = started.elapsed();
    assert_eq!(output.master.len(), 480_000 * 2);
    // Generous bound (debug builds, loaded CI): still far inside the
    // 10 s of audio rendered, proving faster-than-realtime.
    assert!(
        elapsed.as_secs_f64() < 8.0,
        "10 s bounce took {elapsed:?} — not faster than realtime",
    );
}

#[test]
fn unity_stems_sum_to_the_master() {
    // Two lanes at unity through identity edges into a unity master:
    // the captured post-fader stems must sum to the master output.
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(1, 1.0, vec![tone_clip(11, 440.0)]),
            lane(2, 1.0, vec![tone_clip(21, 553.0)]),
            master(vec![1, 2]),
        ],
    };
    let options = OfflineRenderOptions {
        frame_count: 24_000,
        capture_stage_ids: vec![1, 2],
        ..OfflineRenderOptions::default()
    };
    let output = render_plan_to_pcm(&spec, &options).unwrap();
    assert_eq!(output.stems.len(), 2);
    let (stem_a_id, stem_a) = &output.stems[0];
    let (stem_b_id, stem_b) = &output.stems[1];
    assert_eq!((*stem_a_id, *stem_b_id), (1, 2));
    assert_eq!(stem_a.len(), output.master.len());
    assert_eq!(stem_b.len(), output.master.len());
    for index in 0..output.master.len() {
        let sum = stem_a[index] + stem_b[index];
        assert!(
            (sum - output.master[index]).abs() < 1e-6,
            "stem sum diverged from master at sample {index}: {sum} vs {}",
            output.master[index],
        );
    }
}
