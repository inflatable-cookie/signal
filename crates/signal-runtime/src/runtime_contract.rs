use super::*;

/// Deployment profile that shapes default scheduling and prework behaviour.
///
/// `Local` is for single-machine DAW use; `Server` targets headless or
/// multi-client deployments where higher anticipative budgets are appropriate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeProfile {
    /// Standard desktop DAW deployment.
    Local,
    /// Headless or remote rendering deployment.
    Server,
}

/// Immutable boot-time configuration for a [`SignalRuntime`] instance.
///
/// Create via [`RuntimeConfig::local`] or [`RuntimeConfig::server`], then pass
/// to [`SignalRuntime::new`].  These values are fixed for the lifetime of the
/// runtime; use [`RuntimeConfigRequest`] to change sample rate or block size
/// dynamically after `configure()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Sample rate for the runtime graph.
    pub sample_rate: SampleRate,
    /// Graph configuration including block size.
    pub graph: GraphConfig,
    /// Deployment profile that shapes default scheduling behaviour.
    pub profile: RuntimeProfile,
}

impl RuntimeConfig {
    /// Constructs a `Local`-profile config.
    pub fn local(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            graph: GraphConfig { block_size },
            profile: RuntimeProfile::Local,
        }
    }

    /// Constructs a `Server`-profile config.
    pub fn server(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            graph: GraphConfig { block_size },
            profile: RuntimeProfile::Server,
        }
    }
}
