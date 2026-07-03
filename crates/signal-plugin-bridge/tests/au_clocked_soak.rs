//! Device-less clocked soak with the AU in-process processor on a Sum stage
//! (g11.032): the render plane runs under
//! `signal_hardware::FakeClockedBackend` for a sustained stretch of
//! callbacks with the stock Apple AUDelay pulled through `AudioUnitRender`
//! on every block.
//!
//! Like the VST3 soak, the counting-allocator zero-alloc proof cannot run
//! from a shared test target; this soak asserts the observable contract
//! instead: sustained clocked load with pull-model AU rendering never
//! panics, never misses a block (`miss_count == 0` — the alloc-free session
//! kept every deadline), health counters advance, and the (fully dry,
//! identity) insert stays audibly live in the stage meters.

#![cfg(target_os = "macos")]

use std::sync::Arc;
use std::time::Duration;

use signal_hardware::{FakeClockedBackend, OutputStreamBackend, OutputStreamSpec};
use signal_plugin_au::AU_REGISTRY_COMPONENT_PATH;
use signal_plugin_bridge::InProcessAuProcessor;
use signal_render_plane::{
    render_plane, ChannelFormat, RenderClipSpec, RenderEdgeSpec, RenderPlanSpec,
    RenderPluginProcessor, RenderSource, RenderStageKind, RenderStageSpec,
};

const AUDELAY_LOAD_KEY: &str = "aufx:dely:appl";
const AUDELAY_WET_DRY_MIX: u32 = 0;

const SAMPLE_RATE_HZ: u32 = 48_000;
const BLOCK_FRAMES: u32 = 256;
const LANE_ID: u64 = 1;
const INSERT_ID: u64 = 2;
const MASTER_ID: u64 = 100;

fn plan_with_processor(processor: RenderPluginProcessor) -> RenderPlanSpec {
    RenderPlanSpec {
        sample_rate_hz: SAMPLE_RATE_HZ,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                processor: None,
                stage_id: LANE_ID,
                format: ChannelFormat::stereo(),
                gain: 0.5,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![RenderClipSpec {
                        clip_id: 11,
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::TestTone {
                            frequency_hz: 440.0,
                        },
                        loop_source: false,
                    }],
                },
                inputs: Vec::new(),
            },
            // The AU insert renders every block of this Sum stage.
            RenderStageSpec {
                processor: Some(processor),
                stage_id: INSERT_ID,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: LANE_ID,
                    gain: 1.0,
                    matrix: None,
                }],
            },
            RenderStageSpec {
                processor: None,
                stage_id: MASTER_ID,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: INSERT_ID,
                    gain: 1.0,
                    matrix: None,
                }],
            },
        ],
    }
}

#[test]
fn au_in_process_soak_processes_every_clocked_block_without_misses() {
    let backend = Arc::new(
        InProcessAuProcessor::load_and_activate(
            std::path::Path::new(AU_REGISTRY_COMPONENT_PATH),
            AUDELAY_LOAD_KEY,
            SAMPLE_RATE_HZ,
            signal_render_plane::MAX_BLOCK_FRAMES as u32,
        )
        .expect("stock AUDelay should load and activate"),
    );
    // Fully dry: the insert is identity, so meters prove liveness exactly.
    backend
        .set_parameter(AUDELAY_WET_DRY_MIX, 0.0)
        .expect("wet/dry mix set");
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

    let (mut controller, mut executor) = render_plane();
    let hardware = FakeClockedBackend::new();
    let stream = hardware
        .open_output_stream(
            OutputStreamSpec {
                sample_rate_hz: SAMPLE_RATE_HZ,
                channels: 2,
                buffer_frames: Some(BLOCK_FRAMES),
            },
            Box::new(move |frames| {
                executor.render_block(frames);
            }),
        )
        .expect("open fake clocked stream");
    assert_eq!(stream.last_error(), None);

    controller
        .set_stream_channels(stream.channels())
        .expect("record stream channels");
    controller
        .install_plan(&plan_with_processor(handle))
        .expect("install plan");
    controller.set_playing(true).expect("play");

    // ~1.2 s of simulated callbacks (256 frames ≈ 5.3 ms per block).
    std::thread::sleep(Duration::from_millis(1_200));

    let callbacks = controller.callback_count();
    assert!(
        callbacks >= 100,
        "expected sustained clocked callbacks, saw {callbacks}",
    );
    assert!(controller.position_frames() > SAMPLE_RATE_HZ as u64 / 2);
    assert_eq!(
        backend.miss_count(),
        0,
        "the alloc-free AU session must never bypass under clocked load",
    );

    // The insert is audibly live and (fully dry) identity: the insert stage
    // meters track the lane's peak closely.
    let meters = controller.meters();
    let lane = meters
        .iter()
        .find(|(id, _, _)| *id == LANE_ID)
        .expect("lane meter");
    let insert = meters
        .iter()
        .find(|(id, _, _)| *id == INSERT_ID)
        .expect("insert meter");
    assert!(lane.1 > 0.01, "lane peak should move, saw {}", lane.1);
    assert!(
        insert.1 > 0.005,
        "insert peak should move, saw {}",
        insert.1
    );
    assert!(
        (insert.1 - lane.1).abs() <= lane.1 * 0.2,
        "dry AUDelay insert must track the lane peak: {} vs {}",
        insert.1,
        lane.1,
    );

    controller.set_playing(false).expect("stop");
    std::thread::sleep(Duration::from_millis(120));
    drop(stream);
    backend.shutdown();
    drop(backend);
}
