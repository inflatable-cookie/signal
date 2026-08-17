//! Public host-edge proof: host assembly → bridge backend → offline render.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use signal_host_local::LocalRuntimeHost;
use signal_plugin::{PluginFormat, PluginIsolationTier};
use signal_plugin_clap::fixture::{compile_clap_fixture, rustc_available, CLAP_FIXTURE_GAIN};
use signal_render_plane::{
    render_plan_to_pcm, ChannelFormat, OfflineRenderOptions, RenderClipSpec, RenderEdgeSpec,
    RenderPlanSpec, RenderPluginProcessor, RenderSampleBuffer, RenderSource, RenderStageKind,
    RenderStageSpec,
};
use signal_runtime::{PluginScanRequest, RuntimeConfig, RuntimeSupervisorApi, SignalRuntime};

struct FixtureDir {
    path: PathBuf,
}

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn unique_fixture_dir(label: &str) -> FixtureDir {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "signal-host-local-public-offline-{label}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("fixture directory should be created");
    FixtureDir { path }
}

fn scan_root(host: &mut LocalRuntimeHost, root: &Path, format: PluginFormat) {
    host.start_plugin_scan(PluginScanRequest {
        roots: vec![root.display().to_string()],
        formats: vec![format],
    })
    .expect("public host-edge fixture scan should succeed");
}

fn constant_stereo_plan(
    processor: Option<RenderPluginProcessor>,
    sample_rate_hz: u32,
    frames: usize,
) -> RenderPlanSpec {
    let mut data = Vec::with_capacity(frames * 2);
    for _ in 0..frames {
        data.push(0.5);
        data.push(0.5);
    }
    let buffer = RenderSampleBuffer::stereo(sample_rate_hz, Arc::from(data));
    RenderPlanSpec {
        sample_rate_hz,
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
                        source: RenderSource::Samples(buffer),
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
    }
}

#[test]
fn local_public_host_edge_drives_offline_render_from_prepare_plugin_processor() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    const SAMPLE_RATE_HZ: u32 = 48_000;
    let directory = unique_fixture_dir("clap");
    let plugin_type_id = "com.signal.host-edge-offline-clap";
    compile_clap_fixture(
        &directory.path,
        plugin_type_id,
        "Signal Host Edge Offline CLAP",
        0,
    )
    .expect("clap fixture should compile");

    let mut host = LocalRuntimeHost::new(SignalRuntime::new(RuntimeConfig::local(
        SAMPLE_RATE_HZ,
        512,
    )));
    scan_root(&mut host, &directory.path, PluginFormat::Clap);
    let processor = host
        .prepare_plugin_processor(plugin_type_id, PluginIsolationTier::InProcess)
        .expect("public host-edge prepare should construct an in-process backend");

    let options = OfflineRenderOptions {
        start_frame: 0,
        frame_count: 1_024,
        block_frames: 128,
        capture_stage_ids: Vec::new(),
    };
    let dry = render_plan_to_pcm(&constant_stereo_plan(None, SAMPLE_RATE_HZ, 1_024), &options)
        .expect("dry offline render");
    let wet = render_plan_to_pcm(
        &constant_stereo_plan(Some(processor), SAMPLE_RATE_HZ, 1_024),
        &options,
    )
    .expect("wet offline render from host-prepared processor");
    assert_eq!(dry.master.len(), wet.master.len());

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
    assert!(checked > 0, "offline host-edge proof should inspect audio");
}
