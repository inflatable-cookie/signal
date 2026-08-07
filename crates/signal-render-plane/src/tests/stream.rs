use super::support::*;
use super::*;

#[test]
fn stream_clips_play_fed_chunks_sample_accurately() {
    let (mut controller, mut executor) = render_plane();
    let total = 4_096u64;
    let (feeder, handle) = render_stream(48_000, total);
    // Window starts at frame 512, well past the edge ramp warm-up.
    let spec = stream_spec(&handle, 512, 512 + total);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    feed_ramp(&feeder, total, 0, 1_024, 256);

    // Two 256-frame blocks open the edge ramp and reach frame 512.
    warm_up(&mut executor, 2);
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Frame 512+128 plays source frame 128, past the clip edge fade.
    let index = 128usize;
    let expected = 128.0 / total as f32;
    assert!((frames[index * 2] - expected).abs() < 1e-6);
    // 1:1 streaming: identical channels, zero underruns.
    assert_eq!(frames[index * 2], frames[index * 2 + 1]);
    assert_eq!(handle.underrun_frames(), 0);
    // The next block starts past the clip anchor: the read hint follows.
    executor.render_block(&mut frames);
    assert_eq!(feeder.wanted_frame(), 256);
}

#[test]
fn stream_underruns_render_silence_and_count() {
    let (mut controller, mut executor) = render_plane();
    let (feeder, handle) = render_stream(48_000, 48_000);
    controller
        .install_plan(&stream_spec(&handle, 0, 48_000))
        .unwrap();
    controller.set_playing(true).unwrap();

    // Nothing fed: every in-window frame is an underrun, output silent.
    let mut frames = [0.1f32; 512];
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(handle.underrun_frames(), 512);

    // Feed the region the executor wants: audio resumes, count holds.
    feed_ramp(
        &feeder,
        48_000,
        feeder.wanted_frame(),
        feeder.wanted_frame() + 2_048,
        512,
    );
    let before = handle.underrun_frames();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.001));
    assert_eq!(handle.underrun_frames(), before);
}

#[test]
fn stream_seek_retires_stale_chunks_and_resumes_once_fed() {
    let (mut controller, mut executor) = render_plane();
    let total = 1_000_000u64;
    let (feeder, handle) = render_stream(48_000, total);
    controller
        .install_plan(&stream_spec(&handle, 0, total))
        .unwrap();
    controller.set_playing(true).unwrap();
    feed_ramp(&feeder, total, 0, 1_024, 256);
    warm_up(&mut executor, 3);
    assert_eq!(handle.underrun_frames(), 0);

    // Seek far past the retire lookahead: held chunks for the old
    // region must come back via the retired mailbox.
    let target = 600_000u64;
    controller.seek(target).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames); // Ramp-out block; seek lands.
    executor.render_block(&mut frames); // First block at the new region.
    assert!(feeder.wanted_frame() >= target);
    // Old-region chunks retire within a few blocks (stale arrivals can
    // sit one block in a held slot first).
    let mut retired = Vec::new();
    for _ in 0..4 {
        retired.extend(feeder.collect_retired());
        executor.render_block(&mut frames);
    }
    retired.extend(feeder.collect_retired());
    assert!(
        retired.iter().all(|chunk| chunk.start_frame < 1_024),
        "only old-region chunks should retire",
    );
    assert!(!retired.is_empty(), "stale chunks should have retired");

    // Feed the new region: playback resumes with the right content.
    let wanted = feeder.wanted_frame();
    feed_ramp(&feeder, total, wanted, wanted + 4_096, 512);
    let before = handle.underrun_frames();
    assert!(before > 0, "seek without data should have underrun");
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert_eq!(handle.underrun_frames(), before);
    let position = controller.position_frames() - 256;
    let expected = position as f32 / total as f32;
    assert!(
        (frames[0] - expected).abs() < 1e-5,
        "resumed at the wrong content"
    );
}

#[test]
fn rate_converted_streams_play_through_the_sinc_path() {
    // 1 kHz sine at 44.1k streamed onto a 48k plan: same SNR bar as the
    // in-memory rate-converted test — proof the stream path shares the
    // polyphase interpolation.
    let (mut controller, mut executor) = render_plane();
    let source_rate = 44_100u32;
    let total = 44_100u64;
    let frequency = 1_000.0f64;
    let (feeder, handle) = render_stream(source_rate, total);
    controller
        .install_plan(&stream_spec(&handle, 0, u64::MAX))
        .unwrap();
    controller.set_playing(true).unwrap();
    // Feed the whole second up front in 8 large chunks.
    let chunk_frames = total.div_ceil(8);
    let mut start = 0u64;
    while start < total {
        let count = chunk_frames.min(total - start);
        let mut data = Vec::with_capacity(count as usize * 2);
        for n in start..start + count {
            let value =
                (std::f64::consts::TAU * frequency * n as f64 / source_rate as f64).sin() as f32;
            data.push(value);
            data.push(value);
        }
        feeder
            .try_send_chunk(StreamChunk {
                start_frame: start,
                frames: data.into(),
            })
            .unwrap();
        start += count;
    }
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
    assert!(snr > 60.0, "rate-converted stream SNR {snr:.1} dB");
    assert_eq!(handle.underrun_frames(), 0);
}

#[test]
fn plan_swap_mid_stream_keeps_held_chunks_without_underrun() {
    let (mut controller, mut executor) = render_plane();
    let total = 48_000u64;
    let (feeder, handle) = render_stream(48_000, total);
    let spec = stream_spec(&handle, 0, total);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    // Feed only what fits in the mailbox + held slots; after the swap no
    // further feeding happens, so continuity proves the held chunks
    // moved across the plan boundary via the clip inheritance map.
    feed_ramp(&feeder, total, 0, 2_048, 256);
    warm_up(&mut executor, 2); // 512 frames consumed, chunks held.

    // Identity recompile mid-stream (the handle is pointer-equal, so
    // the spec is too — hosts would skip this install; force it).
    controller.install_plan(&spec.clone()).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert_eq!(handle.underrun_frames(), 0, "swap dropped held chunks");
    // Content continues exactly: frame 512 plays source frame 512.
    let expected = 512.0 / total as f32;
    assert!((frames[0] - expected).abs() < 1e-6);
}
#[test]
fn stream_handles_compare_by_pointer_for_cheap_spec_equality() {
    let (_feeder_a, a) = render_stream(48_000, 1_000);
    let b = a.clone();
    let (_feeder_c, c) = render_stream(48_000, 1_000);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.source_sample_rate_hz(), 48_000);
    assert_eq!(a.total_frames(), 1_000);
}
