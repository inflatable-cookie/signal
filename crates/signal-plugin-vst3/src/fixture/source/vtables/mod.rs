mod component;
mod controller;
mod midi;
mod object;
mod processor;

pub(crate) fn vtables_fragment(default_bus_channels: u16) -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}",
        object::object_header_fragment(),
        midi::midi_fragment(),
        component::component_vtable_fragment(default_bus_channels),
        processor::processor_vtable_fragment(),
        controller::controller_vtable_fragment(),
        object::object_footer_fragment(),
        component::component_impl_fragment(default_bus_channels),
        processor::processor_impl_fragment(),
        controller::controller_impl_fragment(),
    )
}
