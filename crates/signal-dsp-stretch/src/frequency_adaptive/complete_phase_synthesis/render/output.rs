use rustfft::num_complex::Complex64;

pub(super) type LayerOutputs = [Vec<Vec<Complex64>>; 3];

pub(super) fn allocate_layer_outputs(
    capture: bool,
    channels: usize,
    domain_len: usize,
) -> Option<LayerOutputs> {
    capture.then(|| {
        std::array::from_fn(|_| vec![vec![Complex64::new(0.0, 0.0); domain_len]; channels])
    })
}

pub(super) fn crop_outputs(
    outputs: &[Vec<Complex64>],
    layer_outputs: Option<LayerOutputs>,
    crop: usize,
    target_len: usize,
) -> (Vec<Vec<f64>>, Option<[Vec<Vec<f64>>; 3]>) {
    let crop_channel = |channel: &[Complex64]| {
        channel[crop..crop + target_len]
            .iter()
            .map(|value| value.re)
            .collect::<Vec<_>>()
    };
    let samples = outputs
        .iter()
        .map(|channel| crop_channel(channel))
        .collect::<Vec<_>>();
    let layer_samples = layer_outputs.map(|layers| {
        layers.map(|channels| {
            channels
                .iter()
                .map(|channel| crop_channel(channel))
                .collect::<Vec<_>>()
        })
    });
    (samples, layer_samples)
}
