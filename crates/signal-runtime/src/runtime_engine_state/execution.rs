#[path = "execution/process.rs"]
mod process;

use super::super::*;

impl RuntimeEngineState {
    fn take_plugin_node_render_batch(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Option<PluginNodeRenderBatch> {
        self.pending_plugin_node_renders
            .remove(&(processing_epoch, block_sequence))
    }

    fn retire_stale_plugin_node_renders(&mut self, processing_epoch: u64, block_sequence: u64) {
        self.pending_plugin_node_renders
            .retain(|(render_epoch, render_block_sequence), _| {
                *render_epoch > processing_epoch
                    || (*render_epoch == processing_epoch
                        && *render_block_sequence >= block_sequence)
            });
    }
}
