use super::super::support::*;
use super::super::*;

#[test]
fn sample_clips_play_buffer_content_at_their_window() {
    let (mut controller, mut executor) = render_plane();
    // 1024 source frames: value = index / 1024.
    let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
    // Window starts at frame 512, well past the edge ramp warm-up.
    let spec = samples_spec(&values, 512, 512 + 1024, false);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    // Two 256-frame blocks open the edge ramp and reach frame 512.
    warm_up(&mut executor, 2);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Frame 512+128 plays source frame 128, past the clip edge fade.
    let index = 128usize;
    let expected = 128.0 / 1024.0;
    assert!((frames[index * 2] - expected).abs() < 1e-5);
    // Same-rate playback: equality on both channels.
    assert_eq!(frames[index * 2], frames[index * 2 + 1]);
}

#[test]
fn mono_source_upmixes_into_stereo_stage() {
    let (mut controller, mut executor) = render_plane();
    // A MONO source (channels = 1): value = index / 1024.
    let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
    let spec = lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 2001,
            start_frames: 512,
            end_frames: 512 + 1024,
            source: RenderSource::Samples(RenderSampleBuffer::mono(48_000, values.into())),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    );
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Source frame 128, up-mixed mono→stereo at the equal-power 1/√2 gain.
    let index = 128usize;
    let expected = (128.0 / 1024.0) * std::f32::consts::FRAC_1_SQRT_2;
    assert!(
        (frames[index * 2] - expected).abs() < 1e-5,
        "L = {}",
        frames[index * 2]
    );
    assert!(
        (frames[index * 2 + 1] - expected).abs() < 1e-5,
        "R = {}",
        frames[index * 2 + 1]
    );
    // Mono is duplicated equally to both ears.
    assert_eq!(frames[index * 2], frames[index * 2 + 1]);
}

#[test]
fn sample_clips_play_their_final_frame() {
    let (mut controller, mut executor) = render_plane();
    // 256 source frames of a constant; window longer than the source.
    let values = vec![0.5f32; 256];
    let spec = samples_spec(&values, 0, u64::MAX, false);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 1);

    // Frames 0..256 played in the warm-up block. The final source frame
    // (255) must have rendered; beyond the source, silence.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Replay from the start and inspect the last in-range frame.
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Frame 255 is the final source frame; with the clamp it plays.
    assert!(frames[255 * 2].abs() > 0.1);
}
#[test]
fn looping_sample_clips_wrap_to_their_start() {
    let (mut controller, mut executor) = render_plane();
    // 100 source frames: value = (index + 1) / 100, looped.
    let values: Vec<f32> = (0..100).map(|index| (index + 1) as f32 / 100.0).collect();
    let spec = samples_spec(&values, 0, u64::MAX, true);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2); // 512 frames: ramp open, loop wrapped 5x.

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Block covers frames 512..768; frame 512 plays source 512 % 100 = 12.
    let expected = 13.0 / 100.0;
    assert!((frames[0] - expected).abs() < 1e-5);
    // Frame 600 wraps to source 0.
    let wrapped = (600 - 512) * 2;
    assert!((frames[wrapped] - 1.0 / 100.0).abs() < 1e-5);
}
#[test]
fn rate_converted_clips_play_through_the_sinc_path() {
    // 1 kHz sine at 44.1k played on a 48k stream: after the edge ramp
    // and clip fade, output must track the analytic sine to ~60 dB
    // (linear interpolation fails this at ~35-40 dB).
    let (mut controller, mut executor) = render_plane();
    let source_rate = 44_100u32;
    let frequency = 1_000.0f64;
    let mut data = Vec::new();
    for n in 0..44_100 {
        let value =
            (std::f64::consts::TAU * frequency * n as f64 / source_rate as f64).sin() as f32;
        data.push(value);
        data.push(value);
    }
    let spec = lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1005,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::Samples(RenderSampleBuffer::stereo(source_rate, data.into())),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    );
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 4); // 1024 frames: ramp open, fades passed.

    let mut frames = vec![0.0f32; 2048];
    executor.render_block(&mut frames);
    let step = source_rate as f64 / 48_000.0;
    let mut error = 0.0f64;
    let mut power = 0.0f64;
    for frame_index in 0..1024usize {
        let stream_frame = 1024 + frame_index as u64;
        let position = stream_frame as f64 * step;
        let expected = (std::f64::consts::TAU * frequency * position / source_rate as f64).sin();
        let actual = frames[frame_index * 2] as f64;
        error += (actual - expected) * (actual - expected);
        power += expected * expected;
    }
    let snr = 10.0 * (power / error.max(1e-30)).log10();
    assert!(snr > 60.0, "rate-converted playback SNR {snr:.1} dB");
}
