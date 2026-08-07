use super::support::*;
use super::*;

#[test]
fn delay_stage_moves_an_impulse_by_exact_stream_frames() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&impulse_delay_spec(7)).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = [0.0f32; 256 * 2];
    executor.render_block(&mut frames);

    assert_eq!(frames[100 * 2], 0.0);
    assert!(frames[107 * 2] > 0.0);
    assert_eq!(frames[107 * 2], frames[107 * 2 + 1]);
    assert_eq!(frames[106 * 2], 0.0);
    assert_eq!(frames[108 * 2], 0.0);
}

#[test]
fn delay_stage_carries_ring_state_across_plan_swap() {
    let (mut controller, mut executor) = render_plane();
    let spec = impulse_delay_spec(300);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    let mut first = [0.0f32; 256 * 2];
    executor.render_block(&mut first);
    assert!(first.iter().all(|sample| *sample == 0.0));

    controller.install_plan(&spec).unwrap();
    let mut second = [0.0f32; 256 * 2];
    executor.render_block(&mut second);
    assert_eq!(second[144 * 2], 1.0);
    assert_eq!(second[144 * 2 + 1], 1.0);
}

#[test]
fn sum_stage_processor_transforms_the_summed_scratch() {
    let (mut controller, mut executor) = render_plane();
    let backend = Arc::new(FakeGainProcessor {
        gain: 0.5,
        calls: AtomicU64::new(0),
    });
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    controller
        .install_plan(&processor_spec(Some(handle)))
        .unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 4);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Past the edge ramp and clip fade: 0.5 content × 0.5 plugin gain.
    assert!(
        (frames[100 * 2] - 0.25).abs() < 1e-5,
        "processed sample read {}",
        frames[100 * 2],
    );
    assert!(backend.calls.load(Ordering::Relaxed) > 0);
}

#[test]
fn processor_miss_bypasses_and_counts_without_touching_scratch() {
    let (mut controller, mut executor) = render_plane();
    let backend = Arc::new(AlwaysMissProcessor {
        misses: AtomicU64::new(0),
    });
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    controller
        .install_plan(&processor_spec(Some(handle)))
        .unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 4);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Bypass: dry content flows through the insert untouched.
    assert!(
        (frames[100 * 2] - 0.5).abs() < 1e-5,
        "bypassed sample read {}",
        frames[100 * 2],
    );
    assert!(backend.misses.load(Ordering::Relaxed) > 0);
}

#[test]
fn processor_absent_and_bypassed_render_identically() {
    let render = |processor: Option<RenderPluginProcessor>| -> Vec<f32> {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&processor_spec(processor)).unwrap();
        controller.set_playing(true).unwrap();
        let mut collected = Vec::new();
        let mut frames = [0.0f32; 512];
        for _ in 0..8 {
            executor.render_block(&mut frames);
            collected.extend_from_slice(&frames);
        }
        collected
    };
    let dry = render(None);
    let bypassed = render(Some(RenderPluginProcessor::new(Arc::new(
        AlwaysMissProcessor {
            misses: AtomicU64::new(0),
        },
    ))));
    assert_eq!(dry, bypassed, "bypass must be bit-identical to absent");
}

#[test]
fn compile_rejects_processors_on_non_sum_stages() {
    let handle = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
        misses: AtomicU64::new(0),
    }));
    let mut spec = tone_spec(440.0);
    spec.stages[0].processor = Some(handle);
    let (mut controller, _executor) = render_plane();
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(error.message.contains("not a Sum stage"), "{error}");
}

#[test]
fn processor_swap_is_structural_not_a_gain_fast_path() {
    let handle_a = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
        misses: AtomicU64::new(0),
    }));
    let handle_b = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
        misses: AtomicU64::new(0),
    }));
    let with_a = processor_spec(Some(handle_a.clone()));
    // Clone (same sample-buffer Arc) so only the processor may differ.
    let with_a_again = with_a.clone();
    let mut with_b = with_a.clone();
    with_b.stages[1].processor = Some(handle_b);
    let _ = handle_a;
    // Same handle: gain-only diff logic sees no change.
    assert_eq!(with_a.differs_only_in_gains(&with_a_again), Some(vec![]));
    // Different handle (same everything else): structural.
    assert_eq!(with_a.differs_only_in_gains(&with_b), None);
}
