/// Slowest playback the preview kernel accepts.
///
/// Work per callback scales as `1/ratio`, so a floor is what makes Contract
/// `046`'s bounded-work requirement satisfiable at all. At `0.25` — four times
/// faster than source — a stereo `128`-frame callback measured `2.36%` of its
/// budget in `g10.040` Batch 40.1.
pub const REALTIME_PREVIEW_STREAM_MIN_RATIO: f64 = 0.25;

/// Largest ratio the frozen geometry covers.
///
/// Contract `046`'s overlap law requires `analysis_hop * ratio <= 0.75 *
/// window_size`, which at the frozen `128`/`512` geometry is exactly `3.0`.
/// Higher ratios are cheap — `0.20%` of budget — so this is a spectral coverage
/// limit, not a cost one. Exceeding it needs the contract's hop reduction,
/// which changes the geometry and is out of this lane's scope.
pub const REALTIME_PREVIEW_STREAM_MAX_RATIO: f64 = 3.0;

/// Working-set ceiling, stereo at `MAX_BLOCK_FRAMES`.
///
/// Batch 40.2 computed `804.3 KiB` from a measured state plus a sized source
/// ring. Derived after the design, deliberately: `g10.039` froze a ceiling
/// before its design existed and moved it three times.
pub const REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES: usize = 1024 * 1024;
