use signal_analysis::AnalysisStage;
use signal_analysis_loudness::{LoudnessMeter, LoudnessMeterConfig};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

fn main() {
    let amplitude = parse_arg("--amplitude").unwrap_or(0.2);
    let sample_rate = parse_arg("--sample-rate").unwrap_or(48_000.0) as u32;
    let seconds = parse_arg("--seconds").unwrap_or(4.0);

    let audio = sine(sample_rate, 1_000.0, amplitude, seconds);
    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
    let result = meter.analyze(&audio);

    println!("sample_rate={sample_rate}");
    println!("amplitude={amplitude:.3}");
    println!("integrated_lufs={:.3}", result.integrated_lufs);
    println!("lra_lu={:.3}", result.loudness_range_lu);
    println!("true_peak_dbtp={:.3}", result.true_peak_dbtp);
    println!("confidence={:.3}", result.confidence.0);
}

fn parse_arg(flag: &str) -> Option<f32> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next()?.parse().ok();
        }
    }
    None
}

fn sine(sample_rate: u32, frequency: f32, amplitude: f32, seconds: f32) -> AudioBuffer {
    let frames = (sample_rate as f32 * seconds).round() as usize;
    let mut samples = Vec::with_capacity(frames);
    for index in 0..frames {
        let t = index as f32 / sample_rate as f32;
        samples.push(amplitude * (core::f32::consts::TAU * frequency * t).sin());
    }
    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}
