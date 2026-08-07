use super::support::*;
use super::*;

#[test]
fn renders_silence_without_plan_and_when_stopped() {
    let (mut controller, mut executor) = render_plane();
    let mut frames = [1.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    controller.install_plan(&tone_spec(440.0)).unwrap();
    let mut frames = [1.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(controller.position_frames(), 0);
}

#[test]
fn renders_tone_and_advances_clock_when_playing() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
    assert_eq!(controller.position_frames(), 256);
    assert!(controller.playing());

    // Both channels carry the same mono sum.
    assert_eq!(frames[10], frames[11]);
}

#[test]
fn seek_moves_the_stream_clock() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();
    controller.seek(96_000).unwrap();

    let mut frames = [0.0f32; 128];
    executor.render_block(&mut frames);
    assert_eq!(controller.position_frames(), 96_000 + 64);
}

#[test]
fn windows_gate_lane_audibility_on_the_stream_clock() {
    let (mut controller, mut executor) = render_plane();
    let mut clip = tone_clip(440.0);
    clip.start_frames = 128;
    clip.end_frames = 256;
    let spec = lane_master_spec(0.5, vec![clip]);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    // Block 0 covers frames 0..128: outside the window, silent.
    let mut frames = [0.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Block 1 covers frames 128..256: inside the window, audible.
    let mut frames = [0.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
}
#[test]
fn transport_stop_ramps_out_instead_of_stepping() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    controller.set_playing(false).unwrap();
    let mut frames = [0.0f32; 1024];
    executor.render_block(&mut frames);
    // Ramp-out block: starts audible, ends silent, no step bigger than
    // the tone's own slope plus the ramp slope.
    assert!(frames[0].abs() > 0.0 || frames[2].abs() > 0.0);
    let tail = &frames[1000..];
    assert!(tail.iter().all(|sample| *sample == 0.0));
    let max_step = frames
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0f32, f32::max);
    assert!(max_step < 0.05, "stop produced a step of {max_step}");

    // Fully stopped afterwards: silence and a held clock.
    let position = controller.position_frames();
    let mut frames = [1.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(controller.position_frames(), position);
}

#[test]
fn seek_while_playing_ramps_out_then_jumps() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);
    let before = controller.position_frames();

    controller.seek(96_000).unwrap();
    let mut frames = [0.0f32; 512];
    // Ramp-out block at the old position; seek lands at its end.
    executor.render_block(&mut frames);
    assert_eq!(controller.position_frames(), 96_000);
    let _ = before;
    // Next block plays from the new position, ramping back in.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
    assert_eq!(controller.position_frames(), 96_000 + 256);
}

#[test]
fn loop_region_rejects_inverted_or_empty_bounds() {
    let (controller, _executor) = render_plane();
    assert!(controller.set_loop_region(Some((100, 100))).is_err());
    assert!(controller.set_loop_region(Some((200, 100))).is_err());
    assert!(controller.set_loop_region(Some((0, 1))).is_ok());
    assert!(controller.set_loop_region(None).is_ok());
}
