//! Storage traits for Signal plugin inventory and library consumers.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use signal_plugin_inventory::{DiscoveredPlugin, InventoryListItem};
use signal_plugin_library::{
    CollectionRecord, FolderRecord, PluginAnnotationRecord, PluginOrganizationSummary,
    PluginPlacementRecord, PluginUserPolicyRecord, TagAssignmentRecord, TagRecord,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLibraryMutationBatch {
    pub collections: Vec<CollectionRecord>,
    pub folders: Vec<FolderRecord>,
    pub plugin_placements: Vec<PluginPlacementRecord>,
    pub tags: Vec<TagRecord>,
    pub tag_assignments: Vec<TagAssignmentRecord>,
    pub plugin_annotations: Vec<PluginAnnotationRecord>,
    pub plugin_user_policies: Vec<PluginUserPolicyRecord>,
}

pub trait PluginInventoryRepository {
    fn upsert_inventory(&self, plugins: &[DiscoveredPlugin]) -> Result<usize>;
    fn list_inventory(&self) -> Result<Vec<InventoryListItem>>;
}

pub trait PluginLibraryRepository {
    fn list_collections(&self) -> Result<Vec<CollectionRecord>>;
    fn list_folders(&self) -> Result<Vec<FolderRecord>>;
    fn list_folders_for_collection(&self, collection_id: &str) -> Result<Vec<FolderRecord>>;
    fn list_plugin_placements(&self) -> Result<Vec<PluginPlacementRecord>>;
    fn list_plugin_placements_for_folder(
        &self,
        folder_id: &str,
    ) -> Result<Vec<PluginPlacementRecord>>;
    fn list_plugin_placements_for_plugin(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PluginPlacementRecord>>;
    fn list_tags(&self) -> Result<Vec<TagRecord>>;
    fn list_tag_assignments(&self) -> Result<Vec<TagAssignmentRecord>>;
    fn list_tag_assignments_for_plugin(&self, plugin_id: &str) -> Result<Vec<TagAssignmentRecord>>;
    fn get_plugin_annotation(&self, plugin_id: &str) -> Result<PluginAnnotationRecord>;
    fn list_plugin_annotations(&self) -> Result<Vec<PluginAnnotationRecord>>;
    fn list_plugin_user_policies(&self) -> Result<Vec<PluginUserPolicyRecord>>;
    fn list_plugin_organization_summaries(&self) -> Result<Vec<PluginOrganizationSummary>> {
        Ok(Vec::new())
    }
}

pub trait PluginLibraryUnitOfWork {
    fn apply_library_mutation_batch(&self, batch: &PluginLibraryMutationBatch) -> Result<()>;
}
