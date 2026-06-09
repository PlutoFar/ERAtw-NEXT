use eratw_engine::{DialogueCondition, DialogueScene, WorldState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};
use thiserror::Error;

pub const CONTENT_SCHEMA_VERSION: &str = "content-package/v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPackageManifest {
    pub schema_version: String,
    pub namespace: String,
    pub package_id: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

impl ContentPackageManifest {
    pub fn new(namespace: impl Into<String>, package_id: impl Into<String>) -> Self {
        Self {
            schema_version: CONTENT_SCHEMA_VERSION.to_string(),
            namespace: namespace.into(),
            package_id: package_id.into(),
            version: "0.1.0".to_string(),
            dependencies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPackage {
    pub manifest: ContentPackageManifest,
    pub dialogue_scenes: Vec<DialogueScene>,
}

impl ContentPackage {
    pub fn validate(&self) -> Result<ContentValidationReport, ContentValidationError> {
        let mut report = ContentValidationReport::default();

        validate_manifest(&self.manifest, &mut report)?;
        validate_dialogue_scenes(&self.dialogue_scenes, &mut report);

        Ok(report)
    }

    pub fn install_into_world(
        &self,
        mut world: WorldState,
    ) -> Result<WorldState, ContentInstallError> {
        let report = self.validate()?;
        if !report.is_clean() {
            return Err(ContentInstallError::ValidationFailed(report));
        }

        merge_dialogue_scenes(&mut world, self.dialogue_scenes.clone())?;
        world.event_log.push(format!(
            "内容包 {}:{} 已加载。",
            self.manifest.namespace, self.manifest.package_id
        ));
        Ok(world)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentValidationReport {
    pub issues: Vec<ContentIssue>,
}

impl ContentValidationReport {
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    fn push(&mut self, code: ContentIssueCode, target: impl Into<String>) {
        self.issues.push(ContentIssue {
            code,
            target: target.into(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentIssue {
    pub code: ContentIssueCode,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentIssueCode {
    EmptyPackageId,
    EmptyNamespace,
    DuplicateDialogueSceneId,
    DuplicateDialogueNodeId,
    EmptyDialogueSceneId,
    EmptyDialogueNodeId,
    EmptyDialogueText,
    MissingEntryNode,
    MissingChoiceNextNode,
    EmptyConditionReference,
    InvalidConditionTime,
    UnreachableDialogueNode,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContentValidationError {
    #[error("unsupported content schema: {0}")]
    UnsupportedSchema(String),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContentInstallError {
    #[error(transparent)]
    Validation(#[from] ContentValidationError),
    #[error("content validation failed with {0} issue(s)")]
    ValidationFailed(ContentValidationReport),
    #[error("dialogue scene already exists: {0}")]
    DuplicateDialogueScene(String),
}

impl std::fmt::Display for ContentValidationReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} issue(s)", self.issues.len())
    }
}

fn validate_manifest(
    manifest: &ContentPackageManifest,
    report: &mut ContentValidationReport,
) -> Result<(), ContentValidationError> {
    if manifest.schema_version != CONTENT_SCHEMA_VERSION {
        return Err(ContentValidationError::UnsupportedSchema(
            manifest.schema_version.clone(),
        ));
    }

    if manifest.namespace.trim().is_empty() {
        report.push(ContentIssueCode::EmptyNamespace, "manifest.namespace");
    }

    if manifest.package_id.trim().is_empty() {
        report.push(ContentIssueCode::EmptyPackageId, "manifest.package_id");
    }

    Ok(())
}

fn validate_dialogue_scenes(scenes: &[DialogueScene], report: &mut ContentValidationReport) {
    let mut scene_ids = BTreeSet::new();

    for scene in scenes {
        if scene.id.trim().is_empty() {
            report.push(ContentIssueCode::EmptyDialogueSceneId, "dialogue_scene");
        } else if !scene_ids.insert(scene.id.as_str()) {
            report.push(ContentIssueCode::DuplicateDialogueSceneId, &scene.id);
        }

        validate_dialogue_scene(scene, report);
    }
}

fn validate_dialogue_scene(scene: &DialogueScene, report: &mut ContentValidationReport) {
    let mut node_ids = BTreeSet::new();
    let mut duplicate_node_ids = BTreeSet::new();

    for node in &scene.nodes {
        if node.id.trim().is_empty() {
            report.push(ContentIssueCode::EmptyDialogueNodeId, &scene.id);
        } else if !node_ids.insert(node.id.as_str()) {
            duplicate_node_ids.insert(node.id.as_str());
            report.push(
                ContentIssueCode::DuplicateDialogueNodeId,
                format!("{}:{}", scene.id, node.id),
            );
        }

        if node.text.trim().is_empty() {
            report.push(
                ContentIssueCode::EmptyDialogueText,
                format!("{}:{}", scene.id, node.id),
            );
        }

        for choice in &node.choices {
            for condition in &choice.conditions {
                validate_dialogue_condition(
                    condition,
                    &format!("{}:{}:{}", scene.id, node.id, choice.id),
                    report,
                );
            }

            if let Some(next_node_id) = &choice.next_node_id {
                if !node_ids.contains(next_node_id.as_str())
                    && !scene.nodes.iter().any(|node| node.id == *next_node_id)
                {
                    report.push(
                        ContentIssueCode::MissingChoiceNextNode,
                        format!("{}:{}:{}", scene.id, node.id, next_node_id),
                    );
                }
            }
        }
    }

    if !node_ids.contains(scene.entry_node_id.as_str()) {
        report.push(
            ContentIssueCode::MissingEntryNode,
            format!("{}:{}", scene.id, scene.entry_node_id),
        );
        return;
    }

    if !duplicate_node_ids.is_empty() {
        return;
    }

    for unreachable in unreachable_dialogue_nodes(scene) {
        report.push(
            ContentIssueCode::UnreachableDialogueNode,
            format!("{}:{}", scene.id, unreachable),
        );
    }
}

fn validate_dialogue_condition(
    condition: &DialogueCondition,
    target_prefix: &str,
    report: &mut ContentValidationReport,
) {
    match condition {
        DialogueCondition::CharacterAtLocation {
            character_id,
            location_id,
        } => {
            if character_id.trim().is_empty() || location_id.trim().is_empty() {
                report.push(ContentIssueCode::EmptyConditionReference, target_prefix);
            }
        }
        DialogueCondition::CharacterMoodAtLeast { character_id, .. } => {
            if character_id.trim().is_empty() {
                report.push(ContentIssueCode::EmptyConditionReference, target_prefix);
            }
        }
        DialogueCondition::RelationshipAffinityAtLeast {
            source_character_id,
            target_character_id,
            ..
        } => {
            if source_character_id.trim().is_empty() || target_character_id.trim().is_empty() {
                report.push(ContentIssueCode::EmptyConditionReference, target_prefix);
            }
        }
        DialogueCondition::WeatherIs { .. } => {}
        DialogueCondition::TimeAtLeast { hour, minute } => {
            if *hour >= 24 || *minute >= 60 {
                report.push(ContentIssueCode::InvalidConditionTime, target_prefix);
            }
        }
    }
}

fn unreachable_dialogue_nodes(scene: &DialogueScene) -> Vec<String> {
    let mut reachable = BTreeSet::new();
    let mut pending = VecDeque::from([scene.entry_node_id.as_str()]);

    while let Some(node_id) = pending.pop_front() {
        if !reachable.insert(node_id) {
            continue;
        }

        let Some(node) = scene.nodes.iter().find(|node| node.id == node_id) else {
            continue;
        };

        for choice in &node.choices {
            if let Some(next_node_id) = &choice.next_node_id {
                pending.push_back(next_node_id);
            }
        }
    }

    scene
        .nodes
        .iter()
        .filter(|node| !reachable.contains(node.id.as_str()))
        .map(|node| node.id.clone())
        .collect()
}

fn merge_dialogue_scenes(
    world: &mut WorldState,
    scenes: Vec<DialogueScene>,
) -> Result<(), ContentInstallError> {
    for scene in &scenes {
        if world
            .dialogue_scenes
            .iter()
            .any(|existing| existing.id == scene.id)
        {
            return Err(ContentInstallError::DuplicateDialogueScene(
                scene.id.clone(),
            ));
        }
    }

    world.dialogue_scenes.extend(scenes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use eratw_engine::{DialogueChoice, DialogueCondition, DialogueEffect, DialogueNode};

    #[test]
    fn clean_package_validates() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.demo"),
            dialogue_scenes: vec![scene_with_nodes(vec![
                node_with_choice("entry", Some("next")),
                node_with_choice("next", None),
            ])],
        };

        let report = package.validate().unwrap();

        assert!(report.is_clean());
    }

    #[test]
    fn unsupported_schema_is_error() {
        let mut package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.demo"),
            dialogue_scenes: Vec::new(),
        };
        package.manifest.schema_version = "content-package/v999".to_string();

        let result = package.validate();

        assert_eq!(
            result,
            Err(ContentValidationError::UnsupportedSchema(
                "content-package/v999".to_string()
            ))
        );
    }

    #[test]
    fn dialogue_validation_reports_duplicate_missing_and_unreachable_nodes() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.demo"),
            dialogue_scenes: vec![scene_with_nodes(vec![
                node_with_choice("entry", Some("missing")),
                node_with_choice("entry", None),
                node_with_choice("orphan", None),
            ])],
        };

        let report = package.validate().unwrap();

        assert_eq!(
            issue_codes(&report),
            vec![
                ContentIssueCode::MissingChoiceNextNode,
                ContentIssueCode::DuplicateDialogueNodeId,
            ]
        );
    }

    #[test]
    fn dialogue_validation_reports_unreachable_nodes_when_ids_are_unique() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.demo"),
            dialogue_scenes: vec![scene_with_nodes(vec![
                node_with_choice("entry", None),
                node_with_choice("orphan", None),
            ])],
        };

        let report = package.validate().unwrap();

        assert_eq!(
            issue_codes(&report),
            vec![ContentIssueCode::UnreachableDialogueNode]
        );
    }

    #[test]
    fn manifest_validation_reports_empty_ids() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("", ""),
            dialogue_scenes: Vec::new(),
        };

        let report = package.validate().unwrap();

        assert_eq!(
            issue_codes(&report),
            vec![
                ContentIssueCode::EmptyNamespace,
                ContentIssueCode::EmptyPackageId,
            ]
        );
    }

    #[test]
    fn dialogue_validation_reports_invalid_choice_conditions() {
        let mut node = node_with_choice("entry", None);
        node.choices = vec![DialogueChoice {
            id: "blocked".to_string(),
            label: "异常条件".to_string(),
            next_node_id: None,
            conditions: vec![
                DialogueCondition::CharacterAtLocation {
                    character_id: "".to_string(),
                    location_id: "school_gate".to_string(),
                },
                DialogueCondition::TimeAtLeast {
                    hour: 25,
                    minute: 0,
                },
            ],
            effects: Vec::new(),
        }];
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.demo"),
            dialogue_scenes: vec![scene_with_nodes(vec![node])],
        };

        let report = package.validate().unwrap();

        assert_eq!(
            issue_codes(&report),
            vec![
                ContentIssueCode::EmptyConditionReference,
                ContentIssueCode::InvalidConditionTime,
            ]
        );
    }

    #[test]
    fn clean_package_installs_dialogue_scenes_into_world() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.extra"),
            dialogue_scenes: vec![DialogueScene {
                id: "scene.extra".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node_with_choice("entry", None)],
            }],
        };
        let world = WorldState::bootstrap_demo();

        let installed = package.install_into_world(world).unwrap();

        assert!(installed
            .dialogue_scenes
            .iter()
            .any(|scene| scene.id == "scene.extra"));
        assert!(installed
            .event_log
            .iter()
            .any(|entry| entry.contains("内容包 core:core.extra 已加载")));
    }

    #[test]
    fn invalid_package_does_not_install() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.invalid"),
            dialogue_scenes: vec![scene_with_nodes(vec![
                node_with_choice("entry", None),
                node_with_choice("orphan", None),
            ])],
        };
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world);

        assert!(matches!(
            result,
            Err(ContentInstallError::ValidationFailed(report))
                if issue_codes(&report) == vec![ContentIssueCode::UnreachableDialogueNode]
        ));
    }

    #[test]
    fn install_rejects_existing_dialogue_scene_id() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.duplicate"),
            dialogue_scenes: vec![DialogueScene {
                id: "demo_morning".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node_with_choice("entry", None)],
            }],
        };
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world);

        assert_eq!(
            result,
            Err(ContentInstallError::DuplicateDialogueScene(
                "demo_morning".to_string()
            ))
        );
    }

    fn scene_with_nodes(nodes: Vec<DialogueNode>) -> DialogueScene {
        DialogueScene {
            id: "scene.demo".to_string(),
            entry_node_id: "entry".to_string(),
            nodes,
        }
    }

    fn node_with_choice(id: &str, next_node_id: Option<&str>) -> DialogueNode {
        DialogueNode {
            id: id.to_string(),
            speaker_id: "demo_heroine".to_string(),
            text: "测试文本。".to_string(),
            choices: next_node_id
                .map(|next_node_id| {
                    vec![DialogueChoice {
                        id: "next".to_string(),
                        label: "继续".to_string(),
                        next_node_id: Some(next_node_id.to_string()),
                        conditions: Vec::new(),
                        effects: vec![DialogueEffect::AddLog {
                            message: "继续。".to_string(),
                        }],
                    }]
                })
                .unwrap_or_default(),
        }
    }

    fn issue_codes(report: &ContentValidationReport) -> Vec<ContentIssueCode> {
        report
            .issues
            .iter()
            .map(|issue| issue.code.clone())
            .collect()
    }
}
