//! VST3 hosted instance lifecycle.

mod controller;
mod hosted;
mod layout;

pub use hosted::Vst3HostedInstance;
#[cfg(test)]
pub(crate) use layout::select_main_bus;
pub(crate) use layout::Vst3AudioBusLayout;
pub use layout::Vst3HostedPortLayout;
