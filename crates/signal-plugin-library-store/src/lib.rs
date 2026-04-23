//! Storage traits for the plugin inventory and library. Implement these traits to provide a persistence backend.

#![warn(missing_docs)]


use anyhow::Result;
use serde::{Deserialize, Serialize};
use signal_plugin_inventory::{DiscoveredPlugin, InventoryListItem};
use signal_plugin_library::{
    CollectionRecord, FolderRecord, PluginAnnotationRecord, PluginOrganizationSummary,
    PluginPlacementRecord, PluginUserPolicyRecord, TagAssignmentRecord, TagRecord,
};

/// A batch of library record sets to write atomically in a single unit of work.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLibraryMutationBatch {
    /// Collection records to upsert.
    pub collections: Vec<CollectionRecord>,
    /// Folder records to upsert.
    pub folders: Vec<FolderRecord>,
    /// Plugin placement records to upsert.
    pub plugin_placements: Vec<PluginPlacementRecord>,
    /// Tag records to upsert.
    pub tags: Vec<TagRecord>,
    /// Tag assignment records to upsert.
    pub tag_assignments: Vec<TagAssignmentRecord>,
    /// Plugin annotation records to upsert.
    pub plugin_annotations: Vec<PluginAnnotationRecord>,
    /// Plugin user policy records to upsert.
    pub plugin_user_policies: Vec<PluginUserPolicyRecord>,
}

/// Persistence interface for the plugin inventory.
pub trait PluginInventoryRepository {
    /// Upserts the given discovered plugins, returning the number of rows affected.
    fn upsert_inventory(&self, plugins: &[DiscoveredPlugin]) -> Result<usize>;
    /// Returns the current inventory as a list of lightweight items.
    fn list_inventory(&self) -> Result<Vec<InventoryListItem>>;
}

/// Persistence interface for the plugin library (collections, folders, placements, tags, and annotations).
pub trait PluginLibraryRepository {
    /// Returns all collections.
    fn list_collections(&self) -> Result<Vec<CollectionRecord>>;
    /// Returns all folders across all collections.
    fn list_folders(&self) -> Result<Vec<FolderRecord>>;
    /// Returns all folders belonging to the given collection.
    fn list_folders_for_collection(&self, collection_id: &str) -> Result<Vec<FolderRecord>>;
    /// Returns all plugin placements across all folders.
    fn list_plugin_placements(&self) -> Result<Vec<PluginPlacementRecord>>;
    /// Returns all plugin placements in the given folder.
    fn list_plugin_placements_for_folder(
        &self,
        folder_id: &str,
    ) -> Result<Vec<PluginPlacementRecord>>;
    /// Returns all folder placements for the given plugin.
    fn list_plugin_placements_for_plugin(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PluginPlacementRecord>>;
    /// Returns all tags.
    fn list_tags(&self) -> Result<Vec<TagRecord>>;
    /// Returns all tag assignments.
    fn list_tag_assignments(&self) -> Result<Vec<TagAssignmentRecord>>;
    /// Returns all tag assignments for the given plugin.
    fn list_tag_assignments_for_plugin(&self, plugin_id: &str) -> Result<Vec<TagAssignmentRecord>>;
    /// Returns the annotation record for the given plugin.
    fn get_plugin_annotation(&self, plugin_id: &str) -> Result<PluginAnnotationRecord>;
    /// Returns all plugin annotation records.
    fn list_plugin_annotations(&self) -> Result<Vec<PluginAnnotationRecord>>;
    /// Returns all user policy records.
    fn list_plugin_user_policies(&self) -> Result<Vec<PluginUserPolicyRecord>>;
    /// Returns organisation summaries (folder and tag labels) for all plugins. Default implementation returns an empty list.
    fn list_plugin_organization_summaries(&self) -> Result<Vec<PluginOrganizationSummary>> {
        Ok(Vec::new())
    }
}

/// Transactional write interface for the plugin library.
pub trait PluginLibraryUnitOfWork {
    /// Applies all records in the batch atomically.
    fn apply_library_mutation_batch(&self, batch: &PluginLibraryMutationBatch) -> Result<()>;
}
