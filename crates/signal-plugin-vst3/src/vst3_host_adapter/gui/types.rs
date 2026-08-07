/// One host-side view callback observed from the plugin, drained by the
/// embedding host (which owns the actual window and applies the change).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3GuiEvent {
    /// The plugin asks the host to resize its window (`IPlugFrame::resizeView`).
    RequestResize {
        /// Requested content width (logical units on macOS).
        width: u32,
        /// Requested content height (logical units on macOS).
        height: u32,
    },
}

/// `Steinberg::ViewRect`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ViewRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl ViewRect {
    pub(crate) fn size(&self) -> (u32, u32) {
        (
            (self.right - self.left).max(0) as u32,
            (self.bottom - self.top).max(0) as u32,
        )
    }

    pub(crate) fn from_size(width: u32, height: u32) -> Self {
        Self {
            left: 0,
            top: 0,
            right: width.min(i32::MAX as u32) as i32,
            bottom: height.min(i32::MAX as u32) as i32,
        }
    }
}
