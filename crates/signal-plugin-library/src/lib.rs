//! Canonical plugin library domain for Signal consumers.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub collection_id: String,
    pub slug: String,
    pub display_name: String,
    pub scope: String,
    pub position: i64,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderRecord {
    pub folder_id: String,
    pub collection_id: String,
    pub parent_folder_id: Option<String>,
    pub slug: String,
    pub display_name: String,
    pub position: i64,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPlacementRecord {
    pub placement_id: String,
    pub plugin_id: String,
    pub folder_id: String,
    pub position: i64,
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagRecord {
    pub tag_id: String,
    pub slug: String,
    pub display_name: String,
    pub position: i64,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagAssignmentRecord {
    pub tag_assignment_id: String,
    pub tag_id: String,
    pub plugin_id: String,
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginAnnotationRecord {
    pub plugin_id: String,
    pub favorite: bool,
    pub hidden: bool,
    pub display_alias: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginUserPolicyRecord {
    pub plugin_id: String,
    pub preferred_isolation_mode: String,
    pub user_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderPathView {
    pub folder_id: String,
    pub collection_id: String,
    pub parent_folder_id: Option<String>,
    pub display_name: String,
    pub position: i64,
    pub depth: usize,
    pub path_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginOrganizationSummary {
    pub plugin_id: String,
    pub folder_labels: Vec<String>,
    pub tag_labels: Vec<String>,
}

pub fn normalize_display_name(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("name cannot be empty"));
    }
    Ok(trimmed.to_string())
}

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

pub fn next_position(existing: impl Iterator<Item = i64>) -> i64 {
    existing.max().unwrap_or(0) + 10
}

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
