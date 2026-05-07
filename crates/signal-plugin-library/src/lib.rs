//! Canonical plugin library domain. Models the user's organised plugin collection: folders, tags, placements, and tag assignments.

#![warn(missing_docs)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A top-level collection in the plugin library (e.g. "My Plugins"). Collections group folders and carry a scope and display order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionRecord {
    /// Unique identifier for this collection.
    pub collection_id: String,
    /// URL-safe slug for this collection.
    pub slug: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Scope string (e.g. `user`, `system`).
    pub scope: String,
    /// Display order position.
    pub position: i64,
    /// Whether this collection is archived.
    pub archived: bool,
}

/// A folder within a collection. Folders can be nested via `parent_folder_id` and hold plugin placements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderRecord {
    /// Unique identifier for this folder.
    pub folder_id: String,
    /// The collection this folder belongs to.
    pub collection_id: String,
    /// Parent folder ID, or `None` if this is a top-level folder.
    pub parent_folder_id: Option<String>,
    /// URL-safe slug for this folder.
    pub slug: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Display order position within the parent.
    pub position: i64,
    /// Whether this folder is archived.
    pub archived: bool,
}

/// A placement of a plugin inside a folder, with an explicit display order and origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPlacementRecord {
    /// Unique identifier for this placement.
    pub placement_id: String,
    /// The plugin being placed.
    pub plugin_id: String,
    /// The folder this placement belongs to.
    pub folder_id: String,
    /// Display order position within the folder.
    pub position: i64,
    /// Origin string describing how this placement was created (e.g. `user`, `auto`).
    pub origin: String,
}

/// A user-defined tag that can be assigned to plugins for filtering and search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagRecord {
    /// Unique identifier for this tag.
    pub tag_id: String,
    /// URL-safe slug for this tag.
    pub slug: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Display order position.
    pub position: i64,
    /// Whether this tag is archived.
    pub archived: bool,
}

/// Assignment of a tag to a specific plugin, including the origin of the assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagAssignmentRecord {
    /// Unique identifier for this assignment.
    pub tag_assignment_id: String,
    /// The tag being assigned.
    pub tag_id: String,
    /// The plugin this tag is assigned to.
    pub plugin_id: String,
    /// Origin string describing how this assignment was created.
    pub origin: String,
}

/// User annotations for a plugin: favourite flag, hidden flag, and optional display alias.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginAnnotationRecord {
    /// The plugin these annotations apply to.
    pub plugin_id: String,
    /// Whether this plugin is marked as a favourite.
    pub favorite: bool,
    /// Whether this plugin is hidden from normal views.
    pub hidden: bool,
    /// Optional user-assigned display alias overriding the plugin name.
    pub display_alias: Option<String>,
    /// ISO 8601 timestamp of the last annotation update, if available.
    pub updated_at: Option<String>,
}

/// User-configured policy for a plugin, including preferred isolation mode and free-text notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginUserPolicyRecord {
    /// The plugin this policy applies to.
    pub plugin_id: String,
    /// Preferred sandbox isolation mode (e.g. `strict`, `relaxed`).
    pub preferred_isolation_mode: String,
    /// Free-text user notes about this plugin.
    pub user_notes: Option<String>,
}

/// A view of a folder including its full path label and nesting depth, computed by [`build_folder_path_views`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderPathView {
    /// The folder this view describes.
    pub folder_id: String,
    /// The collection this folder belongs to.
    pub collection_id: String,
    /// Parent folder ID, or `None` for a top-level folder.
    pub parent_folder_id: Option<String>,
    /// Human-readable display name of this folder.
    pub display_name: String,
    /// Display order position within the parent.
    pub position: i64,
    /// Nesting depth (0 = top-level).
    pub depth: usize,
    /// Full path label built from ancestor display names, e.g. `Root / Child`.
    pub path_label: String,
}

/// Summary of a plugin's current folder and tag assignments, used in list and search views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginOrganizationSummary {
    /// The plugin this summary describes.
    pub plugin_id: String,
    /// Display labels of folders this plugin appears in.
    pub folder_labels: Vec<String>,
    /// Display labels of tags assigned to this plugin.
    pub tag_labels: Vec<String>,
}

/// Trims and validates a display name. Returns an error if the result would be empty.
pub fn normalize_display_name(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("name cannot be empty"));
    }
    Ok(trimmed.to_string())
}

/// Converts a display name to a lowercase kebab-case slug. Falls back to `"item"` if the result would be empty.
pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_dash = false;
    for ch in input.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_was_dash = false;
        } else if !previous_was_dash {
            slug.push('-');
            previous_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "item".to_string()
    } else {
        slug
    }
}

/// Returns `base` if it is not already in `existing`, otherwise appends `-2`, `-3`, etc. until unique.
pub fn unique_slug(base: &str, existing: &[&str]) -> String {
    if !existing.contains(&base) {
        return base.to_string();
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !existing.contains(&candidate.as_str()) {
            return candidate;
        }
        suffix += 1;
    }
}

/// Returns a position value 10 greater than the current maximum, or 10 if the iterator is empty.
pub fn next_position(existing: impl Iterator<Item = i64>) -> i64 {
    existing.max().unwrap_or(0) + 10
}

/// Returns an error if `candidate_parent_id` is `folder_id` or any descendant of it, preventing circular folder hierarchies.
pub fn ensure_not_descendant(
    folder_id: &str,
    candidate_parent_id: &str,
    folders: &[FolderRecord],
) -> Result<()> {
    let parent_lookup = folders
        .iter()
        .map(|folder| {
            (
                folder.folder_id.as_str(),
                folder.parent_folder_id.as_deref(),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut current = Some(candidate_parent_id);

    while let Some(folder) = current {
        if folder == folder_id {
            return Err(anyhow!("folder cannot move under its own descendant"));
        }
        current = parent_lookup.get(folder).copied().flatten();
    }

    Ok(())
}

/// Builds [`FolderPathView`] entries for all folders, computing full path labels and depths. Results are sorted by collection, path label, and position.
pub fn build_folder_path_views(folders: &[FolderRecord]) -> Vec<FolderPathView> {
    let records = folders
        .iter()
        .cloned()
        .map(|folder| (folder.folder_id.clone(), folder))
        .collect::<HashMap<_, _>>();
    let mut views = folders
        .iter()
        .cloned()
        .map(|folder| {
            let mut segments = vec![folder.display_name.clone()];
            let mut depth = 0usize;
            let mut current_parent = folder.parent_folder_id.clone();

            while let Some(parent_id) = current_parent {
                if let Some(parent) = records.get(&parent_id) {
                    segments.push(parent.display_name.clone());
                    current_parent = parent.parent_folder_id.clone();
                    depth += 1;
                } else {
                    break;
                }
            }

            segments.reverse();

            FolderPathView {
                folder_id: folder.folder_id,
                collection_id: folder.collection_id,
                parent_folder_id: folder.parent_folder_id,
                display_name: folder.display_name,
                position: folder.position,
                depth,
                path_label: segments.join(" / "),
            }
        })
        .collect::<Vec<_>>();

    views.sort_by(|left, right| {
        left.collection_id
            .cmp(&right.collection_id)
            .then(left.path_label.cmp(&right.path_label))
            .then(left.position.cmp(&right.position))
    });
    views
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_folder_paths() {
        let folders = vec![
            FolderRecord {
                folder_id: "root".to_string(),
                collection_id: "col".to_string(),
                parent_folder_id: None,
                slug: "root".to_string(),
                display_name: "Root".to_string(),
                position: 10,
                archived: false,
            },
            FolderRecord {
                folder_id: "child".to_string(),
                collection_id: "col".to_string(),
                parent_folder_id: Some("root".to_string()),
                slug: "child".to_string(),
                display_name: "Child".to_string(),
                position: 20,
                archived: false,
            },
        ];

        let views = build_folder_path_views(&folders);
        assert_eq!(views[0].path_label, "Root");
        assert_eq!(views[1].path_label, "Root / Child");
    }
}
