use super::*;

#[test]
fn fixture_plugin_processes_a_chain_insert_through_the_real_engine_offline_render() {
    let _slot = sandbox_child_slot();
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = unique_fixture_dir();
    let library = compile_clap_fixture(
        &directory.path,
        FIXTURE_PLUGIN_ID,
        "Signal Sandbox Hosting Fixture",
        0,
    )
    .expect("fixture should compile");

    let (mut client, processor) = spawn_processing_session(&library);
    let handle = RenderPluginProcessor::new(Arc::clone(&processor) as Arc<_>);

    // Constant-content source lane so the differencing is exact.
    let mut data = Vec::new();
    for _ in 0..SAMPLE_RATE_HZ / 2 {
        data.push(0.5f32);
        data.push(0.5f32);
    }
    let buffer = RenderSampleBuffer::stereo(SAMPLE_RATE_HZ, data.into());
    let plan = |processor: Option<RenderPluginProcessor>| RenderPlanSpec {
        sample_rate_hz: SAMPLE_RATE_HZ,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 1,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![RenderClipSpec {
                        clip_id: 11,
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::Samples(buffer.clone()),
                        loop_source: true,
                        fade_in_frames: 0,
                        fade_out_frames: 0,
                    }],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor,
                events: None,
                stage_id: 2,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: 1,
                    gain: 1.0,
                    matrix: None,
                }],
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 100,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: 2,
                    gain: 1.0,
                    matrix: None,
                }],
            },
        ],
    };
    let options = OfflineRenderOptions {
        start_frame: 0,
        frame_count: 4_800,
        block_frames: 128,
        capture_stage_ids: Vec::new(),
    };

    // Warm the child's audio thread so the offline render (faster than
    // realtime, no retries) never races its first block.
    let mut warm = vec![0.0f32; 256];
    process_offline(&handle, &mut warm, 128, 2);

    let dry = render_plan_to_pcm(&plan(None), &options).expect("dry render");
    let wet = render_plan_to_pcm(&plan(Some(handle)), &options).expect("wet render");
    assert_eq!(dry.master.len(), wet.master.len());

    // Render-differencing: skip the clip edge fade, then every sample must
    // be dry × fixture gain — the insert audibly halves the mix.
    let fade_guard = 64 * 2;
    let mut checked = 0usize;
    for (index, (dry_sample, wet_sample)) in dry
        .master
        .iter()
        .zip(wet.master.iter())
        .enumerate()
        .skip(fade_guard)
    {
        assert!(
            (wet_sample - dry_sample * CLAP_FIXTURE_GAIN).abs() < 1e-6,
            "sample {index}: wet {wet_sample} vs dry {dry_sample} * {CLAP_FIXTURE_GAIN}",
        );
        checked += 1;
    }
    assert!(checked > 8_000, "differencing covered the render");
    // The dry mix itself was audible (the test has teeth).
    assert!(dry.master[fade_guard].abs() > 0.4);

    client.stop_processing().expect("stop-processing");
    client.unload_plugin().expect("unload-plugin");
    client.shutdown().expect("shutdown");
}
