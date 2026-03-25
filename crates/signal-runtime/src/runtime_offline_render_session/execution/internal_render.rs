use super::super::*;
#[path = "internal_render/block_loop.rs"]
mod block_loop;
#[path = "internal_render/result.rs"]
mod result;

pub(super) struct OfflineRenderSynchronousPass {
    pub(super) preview: RuntimeOfflineRenderContractPreview,
    pub(super) plugin_execution_boundary: RuntimeOfflinePluginExecutionBoundary,
    pub(super) delegated_execution_request: RuntimeOfflinePluginDelegatedExecutionRequest,
    pub(super) main_mix: Option<AudioBuffer>,
    pub(super) stem_outputs: BTreeMap<String, Option<AudioBuffer>>,
    pub(super) total_frames: usize,
    pub(super) total_block_count: usize,
    pub(super) rendered_frames: usize,
    pub(super) block_count: usize,
    pub(super) checkpoint_drafts: Vec<OfflineRenderCheckpointDraft>,
}

impl SignalRuntime {
    pub(super) fn render_offline_internal(
        &self,
        request: RuntimeOfflineRenderRequest,
        collect_checkpoints: bool,
    ) -> Result<RuntimeOfflineRenderExecutionReceipt, RuntimeError> {
        let pass = self.run_offline_render_synchronous_pass(&request, collect_checkpoints)?;
        self.finalize_offline_render_synchronous_receipt(request, collect_checkpoints, pass)
    }
}
