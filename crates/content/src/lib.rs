use eratw_engine::{
    Character, DialogueCondition, DialogueEffect, DialogueScene, Location, Relationship,
    ResourceAsset, ScheduledEvent, ScheduledEventKind, WorldState,
};
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
    #[serde(default)]
    pub locations: Vec<Location>,
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    #[serde(default)]
    pub resources: Vec<ResourceAsset>,
    #[serde(default)]
    pub dialogue_scenes: Vec<DialogueScene>,
    #[serde(default)]
    pub scheduled_events: Vec<ScheduledEvent>,
}

impl ContentPackage {
    pub fn validate(&self) -> Result<ContentValidationReport, ContentValidationError> {
        let mut report = ContentValidationReport::default();

        validate_manifest(&self.manifest, &mut report)?;
        validate_locations(&self.locations, &mut report);
        validate_characters(&self.characters, &mut report);
        validate_relationships(&self.relationships, &mut report);
        validate_resources(&self.resources, &mut report);
        validate_dialogue_scenes(&self.dialogue_scenes, &mut report);
        validate_scheduled_events(&self.scheduled_events, &mut report);

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

        merge_locations(&mut world, self.locations.clone())?;
        merge_characters(&mut world, self.characters.clone())?;
        merge_relationships(&mut world, self.relationships.clone())?;
        merge_resources(&mut world, self.resources.clone())?;
        ensure_world_references_exist(&world)?;
        ensure_dialogue_references_exist(&world, &self.dialogue_scenes)?;
        merge_dialogue_scenes(&mut world, self.dialogue_scenes.clone())?;
        merge_scheduled_events(&mut world, self.scheduled_events.clone())?;
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
    EmptyLocationId,
    DuplicateLocationId,
    EmptyLocationName,
    EmptyLocationTerrain,
    EmptyCharacterId,
    DuplicateCharacterId,
    EmptyCharacterName,
    EmptyCharacterLocation,
    EmptyRelationshipReference,
    DuplicateRelationship,
    EmptyResourceId,
    DuplicateResourceId,
    EmptyResourcePath,
    EmptyResourceLicense,
    EmptyResourceAuthor,
    DuplicateDialogueSceneId,
    DuplicateDialogueNodeId,
    EmptyDialogueSceneId,
    EmptyDialogueNodeId,
    EmptyDialogueText,
    EmptyDialogueResourceRef,
    MissingEntryNode,
    MissingChoiceNextNode,
    EmptyConditionReference,
    InvalidConditionTime,
    UnreachableDialogueNode,
    EmptyScheduledEventId,
    DuplicateScheduledEventId,
    InvalidScheduledEventTime,
    InvalidScheduledRepeat,
    EmptyScheduledEventReference,
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
    #[error("scheduled event already exists: {0}")]
    DuplicateScheduledEvent(String),
    #[error("location already exists: {0}")]
    DuplicateLocation(String),
    #[error("character already exists: {0}")]
    DuplicateCharacter(String),
    #[error("relationship already exists: {source_character_id} -> {target_character_id}")]
    DuplicateRelationship {
        source_character_id: String,
        target_character_id: String,
    },
    #[error("resource already exists: {0}")]
    DuplicateResource(String),
    #[error("location reference is missing: {target} -> {location_id}")]
    MissingLocationReference { target: String, location_id: String },
    #[error("character reference is missing: {target} -> {character_id}")]
    MissingCharacterReference {
        target: String,
        character_id: String,
    },
    #[error("relationship reference is missing: {target} -> {source_character_id} -> {target_character_id}")]
    MissingRelationshipReference {
        target: String,
        source_character_id: String,
        target_character_id: String,
    },
    #[error("dialogue resource is missing: {node_id} -> {resource_id}")]
    MissingDialogueResource {
        node_id: String,
        resource_id: String,
    },
    #[error("scheduled event dialogue scene is missing: {event_id} -> {scene_id}")]
    MissingScheduledEventScene { event_id: String, scene_id: String },
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

fn validate_locations(locations: &[Location], report: &mut ContentValidationReport) {
    let mut location_ids = BTreeSet::new();

    for location in locations {
        let target = if location.id.trim().is_empty() {
            "location".to_string()
        } else {
            location.id.clone()
        };

        if location.id.trim().is_empty() {
            report.push(ContentIssueCode::EmptyLocationId, "location");
        } else if !location_ids.insert(location.id.as_str()) {
            report.push(ContentIssueCode::DuplicateLocationId, &location.id);
        }

        if location.name.trim().is_empty() {
            report.push(ContentIssueCode::EmptyLocationName, &target);
        }

        if location.terrain.trim().is_empty() {
            report.push(ContentIssueCode::EmptyLocationTerrain, &target);
        }
    }
}

fn validate_characters(characters: &[Character], report: &mut ContentValidationReport) {
    let mut character_ids = BTreeSet::new();

    for character in characters {
        let target = if character.id.trim().is_empty() {
            "character".to_string()
        } else {
            character.id.clone()
        };

        if character.id.trim().is_empty() {
            report.push(ContentIssueCode::EmptyCharacterId, "character");
        } else if !character_ids.insert(character.id.as_str()) {
            report.push(ContentIssueCode::DuplicateCharacterId, &character.id);
        }

        if character.display_name.trim().is_empty() {
            report.push(ContentIssueCode::EmptyCharacterName, &target);
        }

        if character.location_id.trim().is_empty() {
            report.push(ContentIssueCode::EmptyCharacterLocation, &target);
        }
    }
}

fn validate_relationships(relationships: &[Relationship], report: &mut ContentValidationReport) {
    let mut relationship_ids = BTreeSet::new();

    for relationship in relationships {
        let target = format!(
            "{}->{}",
            relationship.source_character_id, relationship.target_character_id
        );

        if relationship.source_character_id.trim().is_empty()
            || relationship.target_character_id.trim().is_empty()
        {
            report.push(ContentIssueCode::EmptyRelationshipReference, &target);
        } else if !relationship_ids.insert((
            relationship.source_character_id.as_str(),
            relationship.target_character_id.as_str(),
        )) {
            report.push(ContentIssueCode::DuplicateRelationship, target);
        }
    }
}

fn validate_resources(resources: &[ResourceAsset], report: &mut ContentValidationReport) {
    let mut resource_ids = BTreeSet::new();

    for resource in resources {
        let target = if resource.resource_id.trim().is_empty() {
            "resource".to_string()
        } else {
            resource.resource_id.clone()
        };

        if resource.resource_id.trim().is_empty() {
            report.push(ContentIssueCode::EmptyResourceId, "resource");
        } else if !resource_ids.insert(resource.resource_id.as_str()) {
            report.push(ContentIssueCode::DuplicateResourceId, &resource.resource_id);
        }

        if resource.source_path.trim().is_empty() {
            report.push(ContentIssueCode::EmptyResourcePath, &target);
        }

        if resource.license.trim().is_empty() || resource.license.trim() == "unknown" {
            report.push(ContentIssueCode::EmptyResourceLicense, &target);
        }

        if resource.author.trim().is_empty() || resource.author.trim() == "unknown" {
            report.push(ContentIssueCode::EmptyResourceAuthor, &target);
        }
    }
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

        for resource_ref in &node.resource_refs {
            if resource_ref.trim().is_empty() {
                report.push(
                    ContentIssueCode::EmptyDialogueResourceRef,
                    format!("{}:{}", scene.id, node.id),
                );
            }
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

fn ensure_world_references_exist(world: &WorldState) -> Result<(), ContentInstallError> {
    for character in &world.characters {
        if !location_exists(world, &character.location_id) {
            return Err(ContentInstallError::MissingLocationReference {
                target: format!("character:{}", character.id),
                location_id: character.location_id.clone(),
            });
        }
    }

    for relationship in &world.relationships {
        ensure_character_ref_exists(
            world,
            &format!(
                "relationship:{}->{}",
                relationship.source_character_id, relationship.target_character_id
            ),
            &relationship.source_character_id,
        )?;
        ensure_character_ref_exists(
            world,
            &format!(
                "relationship:{}->{}",
                relationship.source_character_id, relationship.target_character_id
            ),
            &relationship.target_character_id,
        )?;
    }

    Ok(())
}

fn ensure_dialogue_references_exist(
    world: &WorldState,
    scenes: &[DialogueScene],
) -> Result<(), ContentInstallError> {
    let resource_ids: BTreeSet<&str> = world
        .resources
        .iter()
        .map(|resource| resource.resource_id.as_str())
        .collect();

    for scene in scenes {
        for node in &scene.nodes {
            ensure_character_ref_exists(
                world,
                &format!("dialogue:{}:{}", scene.id, node.id),
                &node.speaker_id,
            )?;

            for resource_id in &node.resource_refs {
                if !resource_ids.contains(resource_id.as_str()) {
                    return Err(ContentInstallError::MissingDialogueResource {
                        node_id: format!("{}:{}", scene.id, node.id),
                        resource_id: resource_id.clone(),
                    });
                }
            }

            for choice in &node.choices {
                let choice_target = format!("dialogue:{}:{}:{}", scene.id, node.id, choice.id);
                for condition in &choice.conditions {
                    ensure_dialogue_condition_refs_exist(world, &choice_target, condition)?;
                }

                for effect in &choice.effects {
                    ensure_dialogue_effect_refs_exist(world, &choice_target, effect)?;
                }
            }
        }
    }

    Ok(())
}

fn ensure_dialogue_condition_refs_exist(
    world: &WorldState,
    target: &str,
    condition: &DialogueCondition,
) -> Result<(), ContentInstallError> {
    match condition {
        DialogueCondition::CharacterAtLocation {
            character_id,
            location_id,
        } => {
            ensure_character_ref_exists(world, target, character_id)?;
            ensure_location_ref_exists(world, target, location_id)?;
        }
        DialogueCondition::CharacterMoodAtLeast { character_id, .. } => {
            ensure_character_ref_exists(world, target, character_id)?;
        }
        DialogueCondition::RelationshipAffinityAtLeast {
            source_character_id,
            target_character_id,
            ..
        } => {
            ensure_relationship_ref_exists(
                world,
                target,
                source_character_id,
                target_character_id,
            )?;
        }
        DialogueCondition::WeatherIs { .. } | DialogueCondition::TimeAtLeast { .. } => {}
    }

    Ok(())
}

fn ensure_dialogue_effect_refs_exist(
    world: &WorldState,
    target: &str,
    effect: &DialogueEffect,
) -> Result<(), ContentInstallError> {
    match effect {
        DialogueEffect::AdjustCharacterState { character_id, .. } => {
            ensure_character_ref_exists(world, target, character_id)?;
        }
        DialogueEffect::AdjustRelationship {
            source_character_id,
            target_character_id,
            ..
        } => {
            ensure_relationship_ref_exists(
                world,
                target,
                source_character_id,
                target_character_id,
            )?;
        }
        DialogueEffect::ChangeWeather { .. } | DialogueEffect::AddLog { .. } => {}
    }

    Ok(())
}

fn ensure_scheduled_event_refs_exist(
    world: &WorldState,
    event: &ScheduledEvent,
) -> Result<(), ContentInstallError> {
    let target = format!("scheduled_event:{}", event.id);

    for condition in &event.conditions {
        ensure_dialogue_condition_refs_exist(world, &target, condition)?;
    }

    match &event.kind {
        ScheduledEventKind::ChangeWeather { .. } => {}
        ScheduledEventKind::StartDialogue { .. } => {}
        ScheduledEventKind::AdjustRelationship {
            source_character_id,
            target_character_id,
            ..
        } => {
            ensure_relationship_ref_exists(
                world,
                &target,
                source_character_id,
                target_character_id,
            )?;
        }
        ScheduledEventKind::AdjustCharacterState { character_id, .. } => {
            ensure_character_ref_exists(world, &target, character_id)?;
        }
    }

    Ok(())
}

fn ensure_character_ref_exists(
    world: &WorldState,
    target: &str,
    character_id: &str,
) -> Result<(), ContentInstallError> {
    if character_exists(world, character_id) {
        Ok(())
    } else {
        Err(ContentInstallError::MissingCharacterReference {
            target: target.to_string(),
            character_id: character_id.to_string(),
        })
    }
}

fn ensure_location_ref_exists(
    world: &WorldState,
    target: &str,
    location_id: &str,
) -> Result<(), ContentInstallError> {
    if location_exists(world, location_id) {
        Ok(())
    } else {
        Err(ContentInstallError::MissingLocationReference {
            target: target.to_string(),
            location_id: location_id.to_string(),
        })
    }
}

fn ensure_relationship_ref_exists(
    world: &WorldState,
    target: &str,
    source_character_id: &str,
    target_character_id: &str,
) -> Result<(), ContentInstallError> {
    ensure_character_ref_exists(world, target, source_character_id)?;
    ensure_character_ref_exists(world, target, target_character_id)?;

    if world.relationships.iter().any(|relationship| {
        relationship.source_character_id == source_character_id
            && relationship.target_character_id == target_character_id
    }) {
        Ok(())
    } else {
        Err(ContentInstallError::MissingRelationshipReference {
            target: target.to_string(),
            source_character_id: source_character_id.to_string(),
            target_character_id: target_character_id.to_string(),
        })
    }
}

fn character_exists(world: &WorldState, character_id: &str) -> bool {
    matches!(character_id, "player" | "system")
        || world
            .characters
            .iter()
            .any(|character| character.id == character_id)
}

fn location_exists(world: &WorldState, location_id: &str) -> bool {
    world
        .locations
        .iter()
        .any(|location| location.id == location_id)
}

fn merge_locations(
    world: &mut WorldState,
    locations: Vec<Location>,
) -> Result<(), ContentInstallError> {
    for location in &locations {
        if world
            .locations
            .iter()
            .any(|existing| existing.id == location.id)
        {
            return Err(ContentInstallError::DuplicateLocation(location.id.clone()));
        }
    }

    world.locations.extend(locations);
    Ok(())
}

fn merge_characters(
    world: &mut WorldState,
    characters: Vec<Character>,
) -> Result<(), ContentInstallError> {
    for character in &characters {
        if world
            .characters
            .iter()
            .any(|existing| existing.id == character.id)
        {
            return Err(ContentInstallError::DuplicateCharacter(
                character.id.clone(),
            ));
        }
    }

    world.characters.extend(characters);
    Ok(())
}

fn merge_relationships(
    world: &mut WorldState,
    relationships: Vec<Relationship>,
) -> Result<(), ContentInstallError> {
    for relationship in &relationships {
        if world.relationships.iter().any(|existing| {
            existing.source_character_id == relationship.source_character_id
                && existing.target_character_id == relationship.target_character_id
        }) {
            return Err(ContentInstallError::DuplicateRelationship {
                source_character_id: relationship.source_character_id.clone(),
                target_character_id: relationship.target_character_id.clone(),
            });
        }
    }

    world.relationships.extend(relationships);
    Ok(())
}

fn merge_resources(
    world: &mut WorldState,
    resources: Vec<ResourceAsset>,
) -> Result<(), ContentInstallError> {
    for resource in &resources {
        if world
            .resources
            .iter()
            .any(|existing| existing.resource_id == resource.resource_id)
        {
            return Err(ContentInstallError::DuplicateResource(
                resource.resource_id.clone(),
            ));
        }
    }

    world.resources.extend(resources);
    world
        .resources
        .sort_by(|left, right| left.resource_id.cmp(&right.resource_id));
    Ok(())
}

fn validate_scheduled_events(events: &[ScheduledEvent], report: &mut ContentValidationReport) {
    let mut event_ids = BTreeSet::new();

    for event in events {
        let target = if event.id.trim().is_empty() {
            "scheduled_event".to_string()
        } else {
            event.id.clone()
        };

        if event.id.trim().is_empty() {
            report.push(ContentIssueCode::EmptyScheduledEventId, "scheduled_event");
        } else if !event_ids.insert(event.id.as_str()) {
            report.push(ContentIssueCode::DuplicateScheduledEventId, &event.id);
        }

        if !event.due.is_valid() {
            report.push(ContentIssueCode::InvalidScheduledEventTime, &target);
        }

        if let Some(repeat) = &event.repeat {
            if repeat.every_minutes == 0 || repeat.remaining_runs == Some(0) {
                report.push(ContentIssueCode::InvalidScheduledRepeat, &target);
            }
        }

        for condition in &event.conditions {
            validate_dialogue_condition(condition, &format!("scheduled_event:{}", target), report);
        }

        validate_scheduled_event_kind(&event.kind, &target, report);
    }
}

fn validate_scheduled_event_kind(
    kind: &ScheduledEventKind,
    target: &str,
    report: &mut ContentValidationReport,
) {
    match kind {
        ScheduledEventKind::ChangeWeather { .. } => {}
        ScheduledEventKind::StartDialogue { scene_id } => {
            if scene_id.trim().is_empty() {
                report.push(ContentIssueCode::EmptyScheduledEventReference, target);
            }
        }
        ScheduledEventKind::AdjustRelationship {
            source_character_id,
            target_character_id,
            ..
        } => {
            if source_character_id.trim().is_empty() || target_character_id.trim().is_empty() {
                report.push(ContentIssueCode::EmptyScheduledEventReference, target);
            }
        }
        ScheduledEventKind::AdjustCharacterState { character_id, .. } => {
            if character_id.trim().is_empty() {
                report.push(ContentIssueCode::EmptyScheduledEventReference, target);
            }
        }
    }
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

fn merge_scheduled_events(
    world: &mut WorldState,
    events: Vec<ScheduledEvent>,
) -> Result<(), ContentInstallError> {
    for event in &events {
        if world
            .scheduled_events
            .iter()
            .any(|existing| existing.id == event.id)
        {
            return Err(ContentInstallError::DuplicateScheduledEvent(
                event.id.clone(),
            ));
        }

        if let ScheduledEventKind::StartDialogue { scene_id } = &event.kind {
            if !world
                .dialogue_scenes
                .iter()
                .any(|scene| scene.id == *scene_id)
            {
                return Err(ContentInstallError::MissingScheduledEventScene {
                    event_id: event.id.clone(),
                    scene_id: scene_id.clone(),
                });
            }
        }

        ensure_scheduled_event_refs_exist(world, event)?;
    }

    world.scheduled_events.extend(events);
    world.scheduled_events.sort_by(compare_scheduled_events);
    Ok(())
}

fn compare_scheduled_events(left: &ScheduledEvent, right: &ScheduledEvent) -> std::cmp::Ordering {
    scheduled_event_absolute_minute(left)
        .cmp(&scheduled_event_absolute_minute(right))
        .then_with(|| right.priority.cmp(&left.priority))
        .then_with(|| left.id.cmp(&right.id))
}

fn scheduled_event_absolute_minute(event: &ScheduledEvent) -> u64 {
    u64::from(event.due.day.saturating_sub(1)) * 24 * 60
        + u64::from(event.due.hour) * 60
        + u64::from(event.due.minute)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eratw_engine::{
        CharacterState, DialogueChoice, DialogueCondition, DialogueEffect, DialogueNode,
        ScheduledRepeat, ScheduledTime, Weather,
    };

    #[test]
    fn clean_package_validates() {
        let package = package_with(
            "core.demo",
            vec![scene_with_nodes(vec![
                node_with_choice("entry", Some("next")),
                node_with_choice("next", None),
            ])],
            Vec::new(),
        );

        let report = package.validate().unwrap();

        assert!(report.is_clean());
    }

    #[test]
    fn unsupported_schema_is_error() {
        let mut package = package_with("core.demo", Vec::new(), Vec::new());
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
        let package = package_with(
            "core.demo",
            vec![scene_with_nodes(vec![
                node_with_choice("entry", Some("missing")),
                node_with_choice("entry", None),
                node_with_choice("orphan", None),
            ])],
            Vec::new(),
        );

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
        let package = package_with(
            "core.demo",
            vec![scene_with_nodes(vec![
                node_with_choice("entry", None),
                node_with_choice("orphan", None),
            ])],
            Vec::new(),
        );

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
            locations: Vec::new(),
            characters: Vec::new(),
            relationships: Vec::new(),
            resources: Vec::new(),
            dialogue_scenes: Vec::new(),
            scheduled_events: Vec::new(),
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
        let package = package_with("core.demo", vec![scene_with_nodes(vec![node])], Vec::new());

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
        let package = package_with(
            "core.extra",
            vec![DialogueScene {
                id: "scene.extra".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node_with_choice("entry", None)],
            }],
            Vec::new(),
        );
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
        let package = package_with(
            "core.invalid",
            vec![scene_with_nodes(vec![
                node_with_choice("entry", None),
                node_with_choice("orphan", None),
            ])],
            Vec::new(),
        );
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
        let package = package_with(
            "core.duplicate",
            vec![DialogueScene {
                id: "demo_morning".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node_with_choice("entry", None)],
            }],
            Vec::new(),
        );
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world);

        assert_eq!(
            result,
            Err(ContentInstallError::DuplicateDialogueScene(
                "demo_morning".to_string()
            ))
        );
    }

    #[test]
    fn missing_content_package_collections_deserialize_as_empty_lists() {
        let value = serde_json::json!({
            "manifest": {
                "schema_version": CONTENT_SCHEMA_VERSION,
                "namespace": "core",
                "package_id": "core.empty",
                "version": "0.1.0",
                "dependencies": []
            }
        });

        let package: ContentPackage = serde_json::from_value(value).unwrap();

        assert!(package.locations.is_empty());
        assert!(package.characters.is_empty());
        assert!(package.relationships.is_empty());
        assert!(package.resources.is_empty());
        assert!(package.dialogue_scenes.is_empty());
        assert!(package.scheduled_events.is_empty());
    }

    #[test]
    fn world_entity_validation_reports_invalid_package_entities() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.entities"),
            locations: vec![
                Location {
                    id: "new_place".to_string(),
                    name: "".to_string(),
                    ascii_symbol: '新',
                    terrain: "".to_string(),
                },
                Location {
                    id: "new_place".to_string(),
                    name: "重复地点".to_string(),
                    ascii_symbol: '重',
                    terrain: "interior".to_string(),
                },
            ],
            characters: vec![
                Character {
                    id: "new_character".to_string(),
                    display_name: "".to_string(),
                    location_id: "".to_string(),
                    state: character_state(),
                },
                Character {
                    id: "new_character".to_string(),
                    display_name: "重复角色".to_string(),
                    location_id: "new_place".to_string(),
                    state: character_state(),
                },
            ],
            relationships: vec![
                Relationship {
                    source_character_id: "player".to_string(),
                    target_character_id: "".to_string(),
                    affinity: 0,
                    trust: 0,
                },
                Relationship {
                    source_character_id: "player".to_string(),
                    target_character_id: "new_character".to_string(),
                    affinity: 0,
                    trust: 0,
                },
                Relationship {
                    source_character_id: "player".to_string(),
                    target_character_id: "new_character".to_string(),
                    affinity: 1,
                    trust: 1,
                },
            ],
            resources: Vec::new(),
            dialogue_scenes: Vec::new(),
            scheduled_events: Vec::new(),
        };

        let report = package.validate().unwrap();

        assert_eq!(
            issue_codes(&report),
            vec![
                ContentIssueCode::EmptyLocationName,
                ContentIssueCode::EmptyLocationTerrain,
                ContentIssueCode::DuplicateLocationId,
                ContentIssueCode::EmptyCharacterName,
                ContentIssueCode::EmptyCharacterLocation,
                ContentIssueCode::DuplicateCharacterId,
                ContentIssueCode::EmptyRelationshipReference,
                ContentIssueCode::DuplicateRelationship,
            ]
        );
    }

    #[test]
    fn resource_validation_reports_invalid_assets_and_refs() {
        let mut node = node_with_choice("entry", None);
        node.resource_refs = vec!["".to_string()];
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.assets"),
            locations: Vec::new(),
            characters: Vec::new(),
            relationships: Vec::new(),
            resources: vec![
                ResourceAsset {
                    resource_id: "portrait".to_string(),
                    source_path: "".to_string(),
                    media_type: eratw_engine::ResourceMediaType::Image,
                    license: "unknown".to_string(),
                    author: "".to_string(),
                    usage: Vec::new(),
                    character_bindings: Vec::new(),
                    tags: Vec::new(),
                    sha256: None,
                },
                resource_asset("portrait"),
            ],
            dialogue_scenes: vec![scene_with_nodes(vec![node])],
            scheduled_events: Vec::new(),
        };

        let report = package.validate().unwrap();

        assert_eq!(
            issue_codes(&report),
            vec![
                ContentIssueCode::EmptyResourcePath,
                ContentIssueCode::EmptyResourceLicense,
                ContentIssueCode::EmptyResourceAuthor,
                ContentIssueCode::DuplicateResourceId,
                ContentIssueCode::EmptyDialogueResourceRef,
            ]
        );
    }

    #[test]
    fn scheduled_event_validation_reports_invalid_events() {
        let mut duplicate = scheduled_weather_event("duplicate", 1, 8, 10, 0);
        duplicate.repeat = Some(ScheduledRepeat {
            every_minutes: 0,
            remaining_runs: None,
        });
        duplicate.conditions = vec![DialogueCondition::TimeAtLeast {
            hour: 24,
            minute: 0,
        }];
        let mut invalid_time = scheduled_weather_event("invalid_time", 1, 25, 0, 0);
        invalid_time.kind = ScheduledEventKind::AdjustCharacterState {
            character_id: "".to_string(),
            energy_delta: 0,
            mood_delta: 1,
        };
        let package = package_with(
            "core.events",
            Vec::new(),
            vec![duplicate.clone(), duplicate, invalid_time],
        );

        let report = package.validate().unwrap();

        assert_eq!(
            issue_codes(&report),
            vec![
                ContentIssueCode::InvalidScheduledRepeat,
                ContentIssueCode::InvalidConditionTime,
                ContentIssueCode::DuplicateScheduledEventId,
                ContentIssueCode::InvalidScheduledRepeat,
                ContentIssueCode::InvalidConditionTime,
                ContentIssueCode::InvalidScheduledEventTime,
                ContentIssueCode::EmptyScheduledEventReference,
            ]
        );
    }

    #[test]
    fn clean_package_installs_scheduled_events_in_runtime_order() {
        let package = package_with(
            "core.events",
            vec![DialogueScene {
                id: "scene.extra".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node_with_choice("entry", None)],
            }],
            vec![
                ScheduledEvent {
                    id: "low".to_string(),
                    due: ScheduledTime::new(1, 8, 10),
                    priority: 0,
                    repeat: None,
                    conditions: Vec::new(),
                    kind: ScheduledEventKind::ChangeWeather {
                        weather: Weather::Cloudy,
                    },
                },
                ScheduledEvent {
                    id: "dialogue".to_string(),
                    due: ScheduledTime::new(1, 8, 10),
                    priority: 10,
                    repeat: Some(ScheduledRepeat {
                        every_minutes: 30,
                        remaining_runs: Some(2),
                    }),
                    conditions: Vec::new(),
                    kind: ScheduledEventKind::StartDialogue {
                        scene_id: "scene.extra".to_string(),
                    },
                },
            ],
        );
        let mut world = WorldState::bootstrap_demo();
        world.scheduled_events.clear();

        let installed = package.install_into_world(world).unwrap();

        assert_eq!(installed.scheduled_events[0].id, "dialogue");
        assert_eq!(installed.scheduled_events[1].id, "low");
        assert!(installed
            .dialogue_scenes
            .iter()
            .any(|scene| scene.id == "scene.extra"));
    }

    #[test]
    fn clean_package_installs_resources_and_dialogue_resource_refs() {
        let mut node = node_with_choice("entry", None);
        node.resource_refs = vec!["core.assets.heroine.smile".to_string()];
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.assets"),
            locations: Vec::new(),
            characters: Vec::new(),
            relationships: Vec::new(),
            resources: vec![resource_asset("core.assets.heroine.smile")],
            dialogue_scenes: vec![DialogueScene {
                id: "scene.asset".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node],
            }],
            scheduled_events: Vec::new(),
        };
        let world = WorldState::bootstrap_demo();

        let installed = package.install_into_world(world).unwrap();

        assert!(installed
            .resources
            .iter()
            .any(|resource| resource.resource_id == "core.assets.heroine.smile"));
        let scene = installed
            .dialogue_scenes
            .iter()
            .find(|scene| scene.id == "scene.asset")
            .unwrap();
        assert_eq!(
            scene.nodes[0].resource_refs,
            vec!["core.assets.heroine.smile".to_string()]
        );
    }

    #[test]
    fn clean_package_installs_locations_characters_and_relationships() {
        let mut node = node_with_choice("entry", None);
        node.speaker_id = "sample_character".to_string();
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("sample", "sample.character"),
            locations: vec![location("sample_room")],
            characters: vec![character("sample_character", "sample_room")],
            relationships: vec![Relationship {
                source_character_id: "player".to_string(),
                target_character_id: "sample_character".to_string(),
                affinity: 3,
                trust: 1,
            }],
            resources: Vec::new(),
            dialogue_scenes: vec![DialogueScene {
                id: "sample_character_intro".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node],
            }],
            scheduled_events: Vec::new(),
        };
        let world = WorldState::bootstrap_demo();

        let installed = package.install_into_world(world).unwrap();

        assert!(installed
            .locations
            .iter()
            .any(|location| location.id == "sample_room"));
        assert!(installed
            .characters
            .iter()
            .any(|character| character.id == "sample_character"));
        assert!(installed.relationships.iter().any(|relationship| {
            relationship.source_character_id == "player"
                && relationship.target_character_id == "sample_character"
        }));
        assert!(installed
            .dialogue_scenes
            .iter()
            .any(|scene| scene.id == "sample_character_intro"));
    }

    #[test]
    fn install_rejects_character_missing_location_transactionally() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("sample", "sample.bad-character"),
            locations: Vec::new(),
            characters: vec![character("sample_character", "missing_room")],
            relationships: Vec::new(),
            resources: Vec::new(),
            dialogue_scenes: Vec::new(),
            scheduled_events: Vec::new(),
        };
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world.clone());

        assert_eq!(
            result,
            Err(ContentInstallError::MissingLocationReference {
                target: "character:sample_character".to_string(),
                location_id: "missing_room".to_string(),
            })
        );
        assert_eq!(
            world.characters.len(),
            WorldState::bootstrap_demo().characters.len()
        );
    }

    #[test]
    fn install_rejects_duplicate_world_entity_ids() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("sample", "sample.duplicates"),
            locations: vec![location("school_gate")],
            characters: Vec::new(),
            relationships: Vec::new(),
            resources: Vec::new(),
            dialogue_scenes: Vec::new(),
            scheduled_events: Vec::new(),
        };
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world);

        assert_eq!(
            result,
            Err(ContentInstallError::DuplicateLocation(
                "school_gate".to_string()
            ))
        );
    }

    #[test]
    fn install_rejects_dialogue_effect_missing_relationship() {
        let mut node = node_with_choice("entry", None);
        node.choices = vec![DialogueChoice {
            id: "missing_relationship".to_string(),
            label: "缺少关系".to_string(),
            next_node_id: None,
            conditions: Vec::new(),
            effects: vec![DialogueEffect::AdjustRelationship {
                source_character_id: "player".to_string(),
                target_character_id: "sample_character".to_string(),
                affinity_delta: 1,
                trust_delta: 1,
            }],
        }];
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("sample", "sample.missing-relationship"),
            locations: vec![location("sample_room")],
            characters: vec![character("sample_character", "sample_room")],
            relationships: Vec::new(),
            resources: Vec::new(),
            dialogue_scenes: vec![DialogueScene {
                id: "sample_relationship_scene".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node],
            }],
            scheduled_events: Vec::new(),
        };
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world);

        assert_eq!(
            result,
            Err(ContentInstallError::MissingRelationshipReference {
                target: "dialogue:sample_relationship_scene:entry:missing_relationship".to_string(),
                source_character_id: "player".to_string(),
                target_character_id: "sample_character".to_string(),
            })
        );
    }

    #[test]
    fn install_rejects_missing_dialogue_resource_ref_transactionally() {
        let mut node = node_with_choice("entry", None);
        node.resource_refs = vec!["missing.resource".to_string()];
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.missing-asset"),
            locations: Vec::new(),
            characters: Vec::new(),
            relationships: Vec::new(),
            resources: Vec::new(),
            dialogue_scenes: vec![DialogueScene {
                id: "scene.asset".to_string(),
                entry_node_id: "entry".to_string(),
                nodes: vec![node],
            }],
            scheduled_events: Vec::new(),
        };
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world.clone());

        assert_eq!(
            result,
            Err(ContentInstallError::MissingDialogueResource {
                node_id: "scene.asset:entry".to_string(),
                resource_id: "missing.resource".to_string(),
            })
        );
        assert_eq!(
            world.resources.len(),
            WorldState::bootstrap_demo().resources.len()
        );
    }

    #[test]
    fn install_rejects_existing_resource_id() {
        let package = ContentPackage {
            manifest: ContentPackageManifest::new("core", "core.duplicate-asset"),
            locations: Vec::new(),
            characters: Vec::new(),
            relationships: Vec::new(),
            resources: vec![resource_asset("core.demo.heroine.neutral")],
            dialogue_scenes: Vec::new(),
            scheduled_events: Vec::new(),
        };
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world);

        assert_eq!(
            result,
            Err(ContentInstallError::DuplicateResource(
                "core.demo.heroine.neutral".to_string()
            ))
        );
    }

    #[test]
    fn install_rejects_existing_scheduled_event_id() {
        let package = package_with(
            "core.duplicate-event",
            Vec::new(),
            vec![scheduled_weather_event("demo_clouds_at_gate", 1, 10, 0, 0)],
        );
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world);

        assert_eq!(
            result,
            Err(ContentInstallError::DuplicateScheduledEvent(
                "demo_clouds_at_gate".to_string()
            ))
        );
    }

    #[test]
    fn install_rejects_scheduled_event_missing_dialogue_scene() {
        let package = package_with(
            "core.missing-scene",
            Vec::new(),
            vec![ScheduledEvent {
                id: "start_missing".to_string(),
                due: ScheduledTime::new(1, 8, 10),
                priority: 0,
                repeat: None,
                conditions: Vec::new(),
                kind: ScheduledEventKind::StartDialogue {
                    scene_id: "missing_scene".to_string(),
                },
            }],
        );
        let world = WorldState::bootstrap_demo();

        let result = package.install_into_world(world);

        assert_eq!(
            result,
            Err(ContentInstallError::MissingScheduledEventScene {
                event_id: "start_missing".to_string(),
                scene_id: "missing_scene".to_string(),
            })
        );
    }

    fn package_with(
        package_id: &str,
        dialogue_scenes: Vec<DialogueScene>,
        scheduled_events: Vec<ScheduledEvent>,
    ) -> ContentPackage {
        ContentPackage {
            manifest: ContentPackageManifest::new("core", package_id),
            locations: Vec::new(),
            characters: Vec::new(),
            relationships: Vec::new(),
            resources: Vec::new(),
            dialogue_scenes,
            scheduled_events,
        }
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
            resource_refs: Vec::new(),
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

    fn scheduled_weather_event(
        id: &str,
        day: u32,
        hour: u8,
        minute: u8,
        priority: i16,
    ) -> ScheduledEvent {
        ScheduledEvent {
            id: id.to_string(),
            due: ScheduledTime::new(day, hour, minute),
            priority,
            repeat: None,
            conditions: Vec::new(),
            kind: ScheduledEventKind::ChangeWeather {
                weather: Weather::Rain,
            },
        }
    }

    fn issue_codes(report: &ContentValidationReport) -> Vec<ContentIssueCode> {
        report
            .issues
            .iter()
            .map(|issue| issue.code.clone())
            .collect()
    }

    fn resource_asset(resource_id: &str) -> ResourceAsset {
        ResourceAsset {
            resource_id: resource_id.to_string(),
            source_path: "assets/demo/heroine-smile.webp".to_string(),
            media_type: eratw_engine::ResourceMediaType::Image,
            license: "project-demo".to_string(),
            author: "ERAtw-NEXT".to_string(),
            usage: vec!["portrait".to_string()],
            character_bindings: vec!["demo_heroine".to_string()],
            tags: vec!["smile".to_string()],
            sha256: None,
        }
    }

    fn location(id: &str) -> Location {
        Location {
            id: id.to_string(),
            name: "新增地点".to_string(),
            ascii_symbol: '新',
            terrain: "interior".to_string(),
        }
    }

    fn character(id: &str, location_id: &str) -> Character {
        Character {
            id: id.to_string(),
            display_name: "新增角色".to_string(),
            location_id: location_id.to_string(),
            state: character_state(),
        }
    }

    fn character_state() -> CharacterState {
        CharacterState {
            energy: 70,
            mood: 0,
        }
    }
}
