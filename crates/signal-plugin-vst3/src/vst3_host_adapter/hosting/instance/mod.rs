//! VST3 hosted instance lifecycle.

mod controller;
mod gui;
mod hosted;
mod layout;
mod lifecycle;
mod load;
mod state;

pub use hosted::Vst3HostedInstance;
#[cfg(test)]
pub(crate) use layout::select_main_bus;
pub(crate) use layout::Vst3AudioBusLayout;
pub use layout::Vst3HostedPortLayout;
