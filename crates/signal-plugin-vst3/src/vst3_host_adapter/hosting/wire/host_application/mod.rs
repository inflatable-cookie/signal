//! VST3 hosting wire: host_application.

mod application;
mod attribute_list;
mod message;

#[cfg(test)]
mod tests;

pub(crate) use application::*;
