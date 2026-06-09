use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod resource;
pub mod save;

pub const ENGINE_VERSION: &str = "0.1.0-m0";
pub const DEMO_RNG_SEED: u64 = 0x4552_4174;
pub const REPLAY_LOG_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weather {
    Clear,
    Cloudy,
    Rain,
    Snow,
}

impl Weather {
    fn label(&self) -> &'static str {
        match self {
            Weather::Clear => "晴",
            Weather::Cloudy => "阴",
            Weather::Rain => "雨",
            Weather::Snow => "雪",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldClock {
    pub day: u32,
    pub hour: u8,
    pub minute: u8,
    pub season: Season,
    pub weather: Weather,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledTime {
    pub day: u32,
    pub hour: u8,
    pub minute: u8,
}

impl ScheduledTime {
    pub fn new(day: u32, hour: u8, minute: u8) -> Self {
        Self { day, hour, minute }
    }

    pub fn is_valid(&self) -> bool {
        self.day > 0 && self.hour < 24 && self.minute < 60
    }

    fn absolute_minute(&self) -> u64 {
        u64::from(self.day.saturating_sub(1)) * 24 * 60
            + u64::from(self.hour) * 60
            + u64::from(self.minute)
    }

    fn from_absolute_minute(absolute_minute: u64) -> Self {
        let minute_of_day = absolute_minute % (24 * 60);
        Self {
            day: (absolute_minute / (24 * 60) + 1) as u32,
            hour: (minute_of_day / 60) as u8,
            minute: (minute_of_day % 60) as u8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub ascii_symbol: char,
    pub terrain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterState {
    pub energy: i16,
    pub mood: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub display_name: String,
    pub location_id: String,
    pub state: CharacterState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub source_character_id: String,
    pub target_character_id: String,
    pub affinity: i16,
    pub trust: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldRandom {
    #[serde(with = "u64_string")]
    pub seed: u64,
    #[serde(with = "u64_string")]
    pub cursor: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineReplayLog {
    pub schema_version: u32,
    pub engine_version: String,
    pub initial_random: WorldRandom,
    pub commands: Vec<EngineCommand>,
}

impl Default for WorldRandom {
    fn default() -> Self {
        Self {
            seed: DEMO_RNG_SEED,
            cursor: 0,
        }
    }
}

impl WorldRandom {
    fn next_u64(&mut self) -> u64 {
        let value = splitmix64(self.seed.wrapping_add(self.cursor));
        self.cursor = self.cursor.wrapping_add(1);
        value
    }

    fn next_bounded_u64(&mut self, upper_exclusive: u64) -> u64 {
        debug_assert!(upper_exclusive > 0);
        let sample_space = u128::from(u64::MAX) + 1;
        let zone = sample_space / u128::from(upper_exclusive) * u128::from(upper_exclusive);

        loop {
            let value = u128::from(self.next_u64());
            if value < zone {
                return (value % u128::from(upper_exclusive)) as u64;
            }
        }
    }

    fn roll_i16_inclusive(&mut self, min: i16, max: i16) -> i16 {
        let span = i64::from(max) - i64::from(min) + 1;
        let offset = self.next_bounded_u64(span as u64) as i64;
        (i64::from(min) + offset) as i16
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: String,
    pub speaker_id: String,
    pub text: String,
    #[serde(default)]
    pub resource_refs: Vec<String>,
    pub choices: Vec<DialogueChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceMediaType {
    Image,
    Audio,
    Font,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAsset {
    pub resource_id: String,
    pub source_path: String,
    pub media_type: ResourceMediaType,
    pub license: String,
    pub author: String,
    #[serde(default)]
    pub usage: Vec<String>,
    #[serde(default)]
    pub character_bindings: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "ContentPackageDependencyWire")]
pub struct ContentPackageDependency {
    pub package_id: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default = "default_required")]
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum ContentPackageDependencyWire {
    PackageId(String),
    Object {
        #[serde(alias = "packageId")]
        package_id: String,
        #[serde(default)]
        version: Option<String>,
        #[serde(default = "default_required")]
        required: bool,
    },
}

impl From<ContentPackageDependencyWire> for ContentPackageDependency {
    fn from(value: ContentPackageDependencyWire) -> Self {
        match value {
            ContentPackageDependencyWire::PackageId(package_id) => Self {
                package_id,
                version: None,
                required: true,
            },
            ContentPackageDependencyWire::Object {
                package_id,
                version,
                required,
            } => Self {
                package_id,
                version,
                required,
            },
        }
    }
}

fn default_required() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledContentPackage {
    pub namespace: String,
    pub package_id: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Vec<ContentPackageDependency>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub id: String,
    pub label: String,
    pub next_node_id: Option<String>,
    #[serde(default)]
    pub conditions: Vec<DialogueCondition>,
    pub effects: Vec<DialogueEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DialogueCondition {
    CharacterAtLocation {
        character_id: String,
        location_id: String,
    },
    CharacterMoodAtLeast {
        character_id: String,
        value: i16,
    },
    RelationshipAffinityAtLeast {
        source_character_id: String,
        target_character_id: String,
        value: i16,
    },
    WeatherIs {
        weather: Weather,
    },
    TimeAtLeast {
        hour: u8,
        minute: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DialogueEffect {
    AdjustCharacterState {
        character_id: String,
        energy_delta: i16,
        mood_delta: i16,
    },
    AdjustRelationship {
        source_character_id: String,
        target_character_id: String,
        affinity_delta: i16,
        trust_delta: i16,
    },
    RollCharacterState {
        character_id: String,
        energy_min_delta: i16,
        energy_max_delta: i16,
        mood_min_delta: i16,
        mood_max_delta: i16,
    },
    ChangeWeather {
        weather: Weather,
    },
    AddLog {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueScene {
    pub id: String,
    pub entry_node_id: String,
    pub nodes: Vec<DialogueNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub id: String,
    pub due: ScheduledTime,
    #[serde(default)]
    pub priority: i16,
    #[serde(default)]
    pub repeat: Option<ScheduledRepeat>,
    #[serde(default)]
    pub conditions: Vec<DialogueCondition>,
    pub kind: ScheduledEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledRepeat {
    pub every_minutes: u16,
    pub remaining_runs: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScheduledEventKind {
    ChangeWeather {
        weather: Weather,
    },
    StartDialogue {
        scene_id: String,
    },
    AdjustRelationship {
        source_character_id: String,
        target_character_id: String,
        affinity_delta: i16,
        trust_delta: i16,
    },
    AdjustCharacterState {
        character_id: String,
        energy_delta: i16,
        mood_delta: i16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    pub engine_version: String,
    #[serde(default)]
    pub installed_content_packages: Vec<InstalledContentPackage>,
    pub clock: WorldClock,
    pub locations: Vec<Location>,
    pub characters: Vec<Character>,
    #[serde(default)]
    pub resources: Vec<ResourceAsset>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    pub dialogue_scenes: Vec<DialogueScene>,
    pub active_dialogue_scene_id: Option<String>,
    pub active_dialogue: Vec<DialogueNode>,
    pub scheduled_events: Vec<ScheduledEvent>,
    #[serde(default)]
    pub random: WorldRandom,
    #[serde(default)]
    pub command_log_initial_random: Option<WorldRandom>,
    pub command_log: Vec<EngineCommand>,
    pub event_log: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineCommand {
    AdvanceTime {
        minutes: u16,
    },
    MoveCharacter {
        character_id: String,
        location_id: String,
    },
    AdjustRelationship {
        source_character_id: String,
        target_character_id: String,
        affinity_delta: i16,
        trust_delta: i16,
    },
    StartDialogue {
        scene_id: String,
    },
    ChooseDialogue {
        node_id: String,
        choice_id: String,
    },
    RollCharacterMood {
        character_id: String,
        min_delta: i16,
        max_delta: i16,
    },
    ScheduleEvent {
        event: ScheduledEvent,
    },
    CancelEvent {
        event_id: String,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EngineError {
    #[error("character not found: {0}")]
    CharacterNotFound(String),
    #[error("relationship not found: {source_character_id} -> {target_character_id}")]
    RelationshipNotFound {
        source_character_id: String,
        target_character_id: String,
    },
    #[error("location not found: {0}")]
    LocationNotFound(String),
    #[error("scene not found: {0}")]
    SceneNotFound(String),
    #[error("dialogue is not active")]
    DialogueNotActive,
    #[error("dialogue node is not active: {0}")]
    DialogueNodeNotActive(String),
    #[error("dialogue node not found: {0}")]
    DialogueNodeNotFound(String),
    #[error("dialogue choice not found: {0}")]
    DialogueChoiceNotFound(String),
    #[error("dialogue choice condition not met: {0}")]
    DialogueChoiceConditionNotMet(String),
    #[error("scheduled event id is required")]
    ScheduledEventIdRequired,
    #[error("duplicate scheduled event: {0}")]
    DuplicateScheduledEvent(String),
    #[error("scheduled event not found: {0}")]
    ScheduledEventNotFound(String),
    #[error("scheduled event has invalid due time: {0}")]
    InvalidScheduledTime(String),
    #[error("scheduled event has invalid repeat: {0}")]
    InvalidScheduledRepeat(String),
    #[error("invalid random range: {min_delta}..={max_delta}")]
    InvalidRandomRange { min_delta: i16, max_delta: i16 },
}

impl WorldState {
    pub fn bootstrap_demo() -> Self {
        Self {
            engine_version: ENGINE_VERSION.to_string(),
            installed_content_packages: Vec::new(),
            clock: WorldClock {
                day: 1,
                hour: 8,
                minute: 0,
                season: Season::Spring,
                weather: Weather::Clear,
            },
            locations: vec![
                Location {
                    id: "school_gate".to_string(),
                    name: "校门".to_string(),
                    ascii_symbol: '門',
                    terrain: "street".to_string(),
                },
                Location {
                    id: "club_room".to_string(),
                    name: "社团室".to_string(),
                    ascii_symbol: '部',
                    terrain: "interior".to_string(),
                },
                Location {
                    id: "garden".to_string(),
                    name: "庭园".to_string(),
                    ascii_symbol: '庭',
                    terrain: "grass".to_string(),
                },
            ],
            characters: vec![Character {
                id: "demo_heroine".to_string(),
                display_name: "示例角色".to_string(),
                location_id: "school_gate".to_string(),
                state: CharacterState {
                    energy: 80,
                    mood: 10,
                },
            }],
            resources: vec![ResourceAsset {
                resource_id: "core.demo.heroine.neutral".to_string(),
                source_path: "assets/demo/heroine-neutral.webp".to_string(),
                media_type: ResourceMediaType::Image,
                license: "project-demo".to_string(),
                author: "ERAtw-NEXT".to_string(),
                usage: vec!["portrait".to_string()],
                character_bindings: vec!["demo_heroine".to_string()],
                tags: vec!["neutral".to_string()],
                sha256: None,
            }],
            relationships: vec![Relationship {
                source_character_id: "player".to_string(),
                target_character_id: "demo_heroine".to_string(),
                affinity: 5,
                trust: 0,
            }],
            dialogue_scenes: demo_dialogue_scenes(),
            active_dialogue_scene_id: None,
            active_dialogue: Vec::new(),
            scheduled_events: vec![
                ScheduledEvent {
                    id: "demo_clouds_at_gate".to_string(),
                    due: ScheduledTime::new(1, 8, 30),
                    priority: 0,
                    repeat: None,
                    conditions: Vec::new(),
                    kind: ScheduledEventKind::ChangeWeather {
                        weather: Weather::Cloudy,
                    },
                },
                ScheduledEvent {
                    id: "demo_morning_mood".to_string(),
                    due: ScheduledTime::new(1, 9, 0),
                    priority: 0,
                    repeat: None,
                    conditions: Vec::new(),
                    kind: ScheduledEventKind::AdjustCharacterState {
                        character_id: "demo_heroine".to_string(),
                        energy_delta: -3,
                        mood_delta: 5,
                    },
                },
            ],
            random: WorldRandom::default(),
            command_log_initial_random: None,
            command_log: Vec::new(),
            event_log: vec!["ERAtw-NEXT M0 engine ready.".to_string()],
        }
    }

    pub fn apply_command(&mut self, command: EngineCommand) -> Result<(), EngineError> {
        let mut next = self.clone();
        let initial_random = next
            .command_log_initial_random
            .clone()
            .unwrap_or_else(|| next.random.clone());
        next.apply_command_inner(command.clone())?;
        if next.command_log.is_empty() {
            next.command_log_initial_random = Some(initial_random);
        }
        next.command_log.push(command);
        *self = next;
        Ok(())
    }

    pub fn replay_log(&self) -> EngineReplayLog {
        EngineReplayLog {
            schema_version: REPLAY_LOG_SCHEMA_VERSION,
            engine_version: ENGINE_VERSION.to_string(),
            initial_random: self.replay_initial_random(),
            commands: self.command_log.clone(),
        }
    }

    fn replay_initial_random(&self) -> WorldRandom {
        match (
            &self.command_log_initial_random,
            self.command_log.is_empty(),
        ) {
            (Some(initial_random), _) => initial_random.clone(),
            (None, true) => self.random.clone(),
            (None, false) => WorldRandom::default(),
        }
    }

    fn apply_command_inner(&mut self, command: EngineCommand) -> Result<(), EngineError> {
        match command {
            EngineCommand::AdvanceTime { minutes } => self.advance_time(minutes),
            EngineCommand::MoveCharacter {
                character_id,
                location_id,
            } => self.move_character(&character_id, &location_id),
            EngineCommand::AdjustRelationship {
                source_character_id,
                target_character_id,
                affinity_delta,
                trust_delta,
            } => self.adjust_relationship_command(
                &source_character_id,
                &target_character_id,
                affinity_delta,
                trust_delta,
            ),
            EngineCommand::StartDialogue { scene_id } => self.start_dialogue(&scene_id),
            EngineCommand::ChooseDialogue { node_id, choice_id } => {
                self.choose_dialogue(&node_id, &choice_id)
            }
            EngineCommand::RollCharacterMood {
                character_id,
                min_delta,
                max_delta,
            } => self.roll_character_mood(&character_id, min_delta, max_delta),
            EngineCommand::ScheduleEvent { event } => self.schedule_event(event),
            EngineCommand::CancelEvent { event_id } => self.cancel_event(&event_id),
        }
    }

    fn advance_time(&mut self, minutes: u16) -> Result<(), EngineError> {
        let end = self.clock_absolute_minute() + u64::from(minutes);
        let minute_of_day = end % (24 * 60);
        self.clock.day = (end / (24 * 60) + 1) as u32;
        self.clock.hour = (minute_of_day / 60) as u8;
        self.clock.minute = (minute_of_day % 60) as u8;
        self.event_log.push(format!("时间推进 {} 分钟。", minutes));
        self.trigger_due_events(end)?;
        Ok(())
    }

    fn move_character(&mut self, character_id: &str, location_id: &str) -> Result<(), EngineError> {
        let location_name = self
            .locations
            .iter()
            .find(|location| location.id == location_id)
            .map(|location| location.name.clone())
            .ok_or_else(|| EngineError::LocationNotFound(location_id.to_string()))?;

        let character = self
            .characters
            .iter_mut()
            .find(|character| character.id == character_id)
            .ok_or_else(|| EngineError::CharacterNotFound(character_id.to_string()))?;

        character.location_id = location_id.to_string();
        self.event_log.push(format!(
            "{} 移动到 {}。",
            character.display_name, location_name
        ));
        Ok(())
    }

    fn start_dialogue(&mut self, scene_id: &str) -> Result<(), EngineError> {
        let scene = self
            .dialogue_scenes
            .iter()
            .find(|scene| scene.id == scene_id)
            .ok_or_else(|| EngineError::SceneNotFound(scene_id.to_string()))?;
        let entry = scene
            .nodes
            .iter()
            .find(|node| node.id == scene.entry_node_id)
            .ok_or_else(|| EngineError::DialogueNodeNotFound(scene.entry_node_id.clone()))?;

        self.active_dialogue_scene_id = Some(scene.id.clone());
        self.active_dialogue = vec![entry.clone()];
        self.event_log.push(format!("播放场景 {}。", scene_id));
        Ok(())
    }

    fn choose_dialogue(&mut self, node_id: &str, choice_id: &str) -> Result<(), EngineError> {
        let active_scene_id = self
            .active_dialogue_scene_id
            .clone()
            .ok_or(EngineError::DialogueNotActive)?;
        let active_node = self
            .active_dialogue
            .iter()
            .find(|node| node.id == node_id)
            .ok_or_else(|| EngineError::DialogueNodeNotActive(node_id.to_string()))?;
        let choice = active_node
            .choices
            .iter()
            .find(|choice| choice.id == choice_id)
            .cloned()
            .ok_or_else(|| EngineError::DialogueChoiceNotFound(choice_id.to_string()))?;

        if !self.dialogue_choice_conditions_met(&choice)? {
            return Err(EngineError::DialogueChoiceConditionNotMet(
                choice_id.to_string(),
            ));
        }

        for effect in &choice.effects {
            self.apply_dialogue_effect(effect)?;
        }

        if let Some(next_node_id) = choice.next_node_id {
            let next = self.dialogue_node(&active_scene_id, &next_node_id)?.clone();
            self.active_dialogue.push(next);
        } else {
            self.active_dialogue_scene_id = None;
        }

        self.event_log
            .push(format!("选择对话 {} / {}。", node_id, choice_id));
        Ok(())
    }

    fn apply_dialogue_effect(&mut self, effect: &DialogueEffect) -> Result<(), EngineError> {
        match effect {
            DialogueEffect::AdjustCharacterState {
                character_id,
                energy_delta,
                mood_delta,
            } => {
                self.adjust_character_state(character_id, *energy_delta, *mood_delta)?;
                Ok(())
            }
            DialogueEffect::AdjustRelationship {
                source_character_id,
                target_character_id,
                affinity_delta,
                trust_delta,
            } => {
                self.adjust_relationship(
                    source_character_id,
                    target_character_id,
                    *affinity_delta,
                    *trust_delta,
                )?;
                Ok(())
            }
            DialogueEffect::RollCharacterState {
                character_id,
                energy_min_delta,
                energy_max_delta,
                mood_min_delta,
                mood_max_delta,
            } => {
                self.roll_character_state(
                    character_id,
                    *energy_min_delta,
                    *energy_max_delta,
                    *mood_min_delta,
                    *mood_max_delta,
                )?;
                Ok(())
            }
            DialogueEffect::ChangeWeather { weather } => {
                self.clock.weather = weather.clone();
                Ok(())
            }
            DialogueEffect::AddLog { message } => {
                self.event_log.push(message.clone());
                Ok(())
            }
        }
    }

    pub fn visible_choices(&self, node: &DialogueNode) -> Result<Vec<DialogueChoice>, EngineError> {
        let mut choices = Vec::new();
        for choice in &node.choices {
            if self.dialogue_choice_conditions_met(choice)? {
                choices.push(choice.clone());
            }
        }
        Ok(choices)
    }

    fn dialogue_choice_conditions_met(&self, choice: &DialogueChoice) -> Result<bool, EngineError> {
        for condition in &choice.conditions {
            if !self.dialogue_condition_met(condition)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn dialogue_condition_met(&self, condition: &DialogueCondition) -> Result<bool, EngineError> {
        match condition {
            DialogueCondition::CharacterAtLocation {
                character_id,
                location_id,
            } => {
                let character = self.character(character_id)?;
                Ok(character.location_id == *location_id)
            }
            DialogueCondition::CharacterMoodAtLeast {
                character_id,
                value,
            } => {
                let character = self.character(character_id)?;
                Ok(character.state.mood >= *value)
            }
            DialogueCondition::RelationshipAffinityAtLeast {
                source_character_id,
                target_character_id,
                value,
            } => {
                self.ensure_character_exists(target_character_id)?;
                let relationship = self.relationship(source_character_id, target_character_id)?;
                Ok(relationship.affinity >= *value)
            }
            DialogueCondition::WeatherIs { weather } => Ok(self.clock.weather == *weather),
            DialogueCondition::TimeAtLeast { hour, minute } => {
                Ok((self.clock.hour, self.clock.minute) >= (*hour, *minute))
            }
        }
    }

    fn dialogue_node(&self, scene_id: &str, node_id: &str) -> Result<&DialogueNode, EngineError> {
        let scene = self
            .dialogue_scenes
            .iter()
            .find(|scene| scene.id == scene_id)
            .ok_or_else(|| EngineError::SceneNotFound(scene_id.to_string()))?;

        scene
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .ok_or_else(|| EngineError::DialogueNodeNotFound(node_id.to_string()))
    }

    fn schedule_event(&mut self, event: ScheduledEvent) -> Result<(), EngineError> {
        if event.id.trim().is_empty() {
            return Err(EngineError::ScheduledEventIdRequired);
        }

        if !event.due.is_valid() {
            return Err(EngineError::InvalidScheduledTime(event.id));
        }

        Self::validate_scheduled_repeat(&event)?;

        if self
            .scheduled_events
            .iter()
            .any(|existing| existing.id == event.id)
        {
            return Err(EngineError::DuplicateScheduledEvent(event.id));
        }

        self.scheduled_events.push(event);
        self.sort_scheduled_events();
        Ok(())
    }

    fn validate_scheduled_repeat(event: &ScheduledEvent) -> Result<(), EngineError> {
        if let Some(repeat) = &event.repeat {
            if repeat.every_minutes == 0 || repeat.remaining_runs == Some(0) {
                return Err(EngineError::InvalidScheduledRepeat(event.id.clone()));
            }
        }

        Ok(())
    }

    fn cancel_event(&mut self, event_id: &str) -> Result<(), EngineError> {
        if event_id.trim().is_empty() {
            return Err(EngineError::ScheduledEventIdRequired);
        }

        let original_len = self.scheduled_events.len();
        self.scheduled_events.retain(|event| event.id != event_id);
        if self.scheduled_events.len() == original_len {
            return Err(EngineError::ScheduledEventNotFound(event_id.to_string()));
        }

        self.event_log
            .push(format!("计划事件 {} 已取消。", event_id));
        Ok(())
    }

    fn roll_character_state(
        &mut self,
        character_id: &str,
        energy_min_delta: i16,
        energy_max_delta: i16,
        mood_min_delta: i16,
        mood_max_delta: i16,
    ) -> Result<(), EngineError> {
        Self::validate_random_range(energy_min_delta, energy_max_delta)?;
        Self::validate_random_range(mood_min_delta, mood_max_delta)?;

        let energy_delta = self
            .random
            .roll_i16_inclusive(energy_min_delta, energy_max_delta);
        let mood_delta = self
            .random
            .roll_i16_inclusive(mood_min_delta, mood_max_delta);
        let display_name = self.adjust_character_state(character_id, energy_delta, mood_delta)?;
        self.event_log.push(format!(
            "{} 状态随机变化：体力 {:+}（范围 {:+}..={:+}），心情 {:+}（范围 {:+}..={:+}）。",
            display_name,
            energy_delta,
            energy_min_delta,
            energy_max_delta,
            mood_delta,
            mood_min_delta,
            mood_max_delta
        ));
        Ok(())
    }

    fn validate_random_range(min_delta: i16, max_delta: i16) -> Result<(), EngineError> {
        if min_delta > max_delta {
            return Err(EngineError::InvalidRandomRange {
                min_delta,
                max_delta,
            });
        }

        Ok(())
    }

    fn roll_character_mood(
        &mut self,
        character_id: &str,
        min_delta: i16,
        max_delta: i16,
    ) -> Result<(), EngineError> {
        Self::validate_random_range(min_delta, max_delta)?;

        let delta = self.random.roll_i16_inclusive(min_delta, max_delta);
        let display_name = self.adjust_character_state(character_id, 0, delta)?;
        self.event_log.push(format!(
            "{} 心情随机变化 {:+}（范围 {:+}..={:+}）。",
            display_name, delta, min_delta, max_delta
        ));
        Ok(())
    }

    fn trigger_due_events(&mut self, end_minute: u64) -> Result<(), EngineError> {
        let mut due_events = Vec::new();
        let mut pending_events = Vec::new();

        for event in self.scheduled_events.drain(..) {
            if event.due.absolute_minute() <= end_minute {
                due_events.push(event);
            } else {
                pending_events.push(event);
            }
        }

        due_events.sort_by(compare_scheduled_events);
        due_events.reverse();

        while let Some(event) = due_events.pop() {
            if self.scheduled_event_conditions_met(&event)? {
                self.execute_scheduled_event(&event)?;
                if let Some(next_event) = Self::reschedule_repeating_event(event) {
                    if next_event.due.absolute_minute() <= end_minute {
                        due_events.push(next_event);
                        due_events.sort_by(compare_scheduled_events);
                        due_events.reverse();
                    } else {
                        pending_events.push(next_event);
                    }
                }
            } else {
                pending_events.push(event);
            }
        }

        self.scheduled_events = pending_events;
        self.sort_scheduled_events();
        Ok(())
    }

    fn scheduled_event_conditions_met(&self, event: &ScheduledEvent) -> Result<bool, EngineError> {
        for condition in &event.conditions {
            if !self.dialogue_condition_met(condition)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn reschedule_repeating_event(mut event: ScheduledEvent) -> Option<ScheduledEvent> {
        let repeat = event.repeat.as_mut()?;
        if let Some(remaining_runs) = repeat.remaining_runs.as_mut() {
            *remaining_runs = remaining_runs.saturating_sub(1);
            if *remaining_runs == 0 {
                return None;
            }
        }

        let next_due = event
            .due
            .absolute_minute()
            .saturating_add(u64::from(repeat.every_minutes));
        event.due = ScheduledTime::from_absolute_minute(next_due);
        Some(event)
    }

    fn execute_scheduled_event(&mut self, event: &ScheduledEvent) -> Result<(), EngineError> {
        match &event.kind {
            ScheduledEventKind::ChangeWeather { weather } => {
                self.clock.weather = weather.clone();
                self.event_log.push(format!(
                    "事件 {} 触发：天气变为{}。",
                    event.id,
                    weather.label()
                ));
                Ok(())
            }
            ScheduledEventKind::StartDialogue { scene_id } => self.start_dialogue(scene_id),
            ScheduledEventKind::AdjustRelationship {
                source_character_id,
                target_character_id,
                affinity_delta,
                trust_delta,
            } => {
                self.adjust_relationship(
                    source_character_id,
                    target_character_id,
                    *affinity_delta,
                    *trust_delta,
                )?;
                self.event_log.push(format!(
                    "事件 {} 触发：关系 {} -> {} 更新。",
                    event.id, source_character_id, target_character_id
                ));
                Ok(())
            }
            ScheduledEventKind::AdjustCharacterState {
                character_id,
                energy_delta,
                mood_delta,
            } => {
                let display_name =
                    self.adjust_character_state(character_id, *energy_delta, *mood_delta)?;
                self.event_log.push(format!(
                    "事件 {} 触发：{} 状态更新。",
                    event.id, display_name
                ));
                Ok(())
            }
        }
    }

    fn adjust_character_state(
        &mut self,
        character_id: &str,
        energy_delta: i16,
        mood_delta: i16,
    ) -> Result<String, EngineError> {
        let character = self
            .characters
            .iter_mut()
            .find(|character| character.id == character_id)
            .ok_or_else(|| EngineError::CharacterNotFound(character_id.to_string()))?;

        character.state.energy = bounded_delta(character.state.energy, energy_delta, 0, 100);
        character.state.mood = bounded_delta(character.state.mood, mood_delta, -100, 100);

        Ok(character.display_name.clone())
    }

    fn adjust_relationship_command(
        &mut self,
        source_character_id: &str,
        target_character_id: &str,
        affinity_delta: i16,
        trust_delta: i16,
    ) -> Result<(), EngineError> {
        self.ensure_character_exists(target_character_id)?;
        self.adjust_relationship(
            source_character_id,
            target_character_id,
            affinity_delta,
            trust_delta,
        )?;
        self.event_log.push(format!(
            "关系 {} -> {} 更新。",
            source_character_id, target_character_id
        ));
        Ok(())
    }

    fn adjust_relationship(
        &mut self,
        source_character_id: &str,
        target_character_id: &str,
        affinity_delta: i16,
        trust_delta: i16,
    ) -> Result<(), EngineError> {
        self.ensure_character_exists(target_character_id)?;

        let relationship = self.relationship_mut(source_character_id, target_character_id)?;

        relationship.affinity = bounded_delta(relationship.affinity, affinity_delta, -100, 100);
        relationship.trust = bounded_delta(relationship.trust, trust_delta, -100, 100);
        Ok(())
    }

    fn character(&self, character_id: &str) -> Result<&Character, EngineError> {
        self.characters
            .iter()
            .find(|character| character.id == character_id)
            .ok_or_else(|| EngineError::CharacterNotFound(character_id.to_string()))
    }

    fn relationship(
        &self,
        source_character_id: &str,
        target_character_id: &str,
    ) -> Result<&Relationship, EngineError> {
        self.relationships
            .iter()
            .find(|relationship| {
                relationship.source_character_id == source_character_id
                    && relationship.target_character_id == target_character_id
            })
            .ok_or_else(|| EngineError::RelationshipNotFound {
                source_character_id: source_character_id.to_string(),
                target_character_id: target_character_id.to_string(),
            })
    }

    fn relationship_mut(
        &mut self,
        source_character_id: &str,
        target_character_id: &str,
    ) -> Result<&mut Relationship, EngineError> {
        self.relationships
            .iter_mut()
            .find(|relationship| {
                relationship.source_character_id == source_character_id
                    && relationship.target_character_id == target_character_id
            })
            .ok_or_else(|| EngineError::RelationshipNotFound {
                source_character_id: source_character_id.to_string(),
                target_character_id: target_character_id.to_string(),
            })
    }

    fn ensure_character_exists(&self, character_id: &str) -> Result<(), EngineError> {
        self.characters
            .iter()
            .any(|character| character.id == character_id)
            .then_some(())
            .ok_or_else(|| EngineError::CharacterNotFound(character_id.to_string()))
    }

    fn sort_scheduled_events(&mut self) {
        self.scheduled_events.sort_by(compare_scheduled_events);
    }

    fn clock_absolute_minute(&self) -> u64 {
        ScheduledTime::new(self.clock.day, self.clock.hour, self.clock.minute).absolute_minute()
    }
}

pub fn replay_commands(
    mut world: WorldState,
    commands: &[EngineCommand],
) -> Result<WorldState, EngineError> {
    for command in commands {
        world.apply_command(command.clone())?;
    }

    Ok(world)
}

pub fn replay_command_log(
    mut world: WorldState,
    replay_log: &EngineReplayLog,
) -> Result<WorldState, EngineError> {
    world.random = replay_log.initial_random.clone();
    world.command_log_initial_random = None;
    world.command_log.clear();
    replay_commands(world, &replay_log.commands)
}

fn bounded_delta(value: i16, delta: i16, min: i16, max: i16) -> i16 {
    value.saturating_add(delta).clamp(min, max)
}

fn compare_scheduled_events(left: &ScheduledEvent, right: &ScheduledEvent) -> std::cmp::Ordering {
    left.due
        .absolute_minute()
        .cmp(&right.due.absolute_minute())
        .then_with(|| right.priority.cmp(&left.priority))
        .then_with(|| left.id.cmp(&right.id))
}

fn splitmix64(value: u64) -> u64 {
    let mut value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

mod u64_string {
    use serde::{de, Deserialize, Deserializer, Serializer};

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64Wire {
        String(String),
        Number(u64),
    }

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        match U64Wire::deserialize(deserializer)? {
            U64Wire::String(value) => value.parse().map_err(de::Error::custom),
            U64Wire::Number(value) => Ok(value),
        }
    }
}

fn demo_dialogue_scenes() -> Vec<DialogueScene> {
    vec![DialogueScene {
        id: "demo_morning".to_string(),
        entry_node_id: "demo_morning_001".to_string(),
        nodes: vec![
            DialogueNode {
                id: "demo_morning_001".to_string(),
                speaker_id: "demo_heroine".to_string(),
                text: "早上好。今天先从一个干净的新世界开始。".to_string(),
                resource_refs: vec!["core.demo.heroine.neutral".to_string()],
                choices: vec![
                    DialogueChoice {
                        id: "ask_about_engine".to_string(),
                        label: "询问新引擎".to_string(),
                        next_node_id: Some("demo_morning_002".to_string()),
                        conditions: Vec::new(),
                        effects: vec![DialogueEffect::AddLog {
                            message: "对话选择：询问新引擎。".to_string(),
                        }],
                    },
                    DialogueChoice {
                        id: "encourage".to_string(),
                        label: "鼓励她".to_string(),
                        next_node_id: Some("demo_morning_003".to_string()),
                        conditions: Vec::new(),
                        effects: vec![
                            DialogueEffect::AdjustCharacterState {
                                character_id: "demo_heroine".to_string(),
                                energy_delta: 0,
                                mood_delta: 3,
                            },
                            DialogueEffect::AdjustRelationship {
                                source_character_id: "player".to_string(),
                                target_character_id: "demo_heroine".to_string(),
                                affinity_delta: 2,
                                trust_delta: 1,
                            },
                        ],
                    },
                    DialogueChoice {
                        id: "talk_about_trust".to_string(),
                        label: "谈谈信任".to_string(),
                        next_node_id: Some("demo_morning_004".to_string()),
                        conditions: vec![DialogueCondition::RelationshipAffinityAtLeast {
                            source_character_id: "player".to_string(),
                            target_character_id: "demo_heroine".to_string(),
                            value: 7,
                        }],
                        effects: vec![DialogueEffect::AdjustRelationship {
                            source_character_id: "player".to_string(),
                            target_character_id: "demo_heroine".to_string(),
                            affinity_delta: 0,
                            trust_delta: 2,
                        }],
                    },
                ],
            },
            DialogueNode {
                id: "demo_morning_002".to_string(),
                speaker_id: "system".to_string(),
                text: "该对话来自版本化 DialogueScene，不执行旧 ERB。".to_string(),
                resource_refs: Vec::new(),
                choices: Vec::new(),
            },
            DialogueNode {
                id: "demo_morning_003".to_string(),
                speaker_id: "demo_heroine".to_string(),
                text: "嗯。先把能稳定重放的小循环做好。".to_string(),
                resource_refs: Vec::new(),
                choices: Vec::new(),
            },
            DialogueNode {
                id: "demo_morning_004".to_string(),
                speaker_id: "demo_heroine".to_string(),
                text: "信任会一点点积累。先从可验证的承诺开始。".to_string(),
                resource_refs: Vec::new(),
                choices: Vec::new(),
            },
        ],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_world_is_deterministic() {
        let left = serde_json::to_string(&WorldState::bootstrap_demo()).unwrap();
        let right = serde_json::to_string(&WorldState::bootstrap_demo()).unwrap();

        assert_eq!(left, right);
    }

    #[test]
    fn advance_time_rolls_day() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 17 * 60 })
            .unwrap();

        assert_eq!(world.clock.day, 2);
        assert_eq!(world.clock.hour, 1);
        assert_eq!(world.clock.minute, 0);
    }

    #[test]
    fn advance_time_triggers_due_events_once() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 60 })
            .unwrap();
        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 60 })
            .unwrap();

        assert_eq!(world.clock.weather, Weather::Cloudy);
        assert_eq!(world.scheduled_events.len(), 0);
        assert_eq!(world.characters[0].state.energy, 77);
        assert_eq!(world.characters[0].state.mood, 15);
        assert_eq!(
            world
                .event_log
                .iter()
                .filter(|entry| entry.contains("demo_clouds_at_gate"))
                .count(),
            1
        );
    }

    #[test]
    fn move_character_rejects_missing_location() {
        let mut world = WorldState::bootstrap_demo();

        let result = world.apply_command(EngineCommand::MoveCharacter {
            character_id: "demo_heroine".to_string(),
            location_id: "missing".to_string(),
        });

        assert_eq!(
            result,
            Err(EngineError::LocationNotFound("missing".to_string()))
        );
    }

    #[test]
    fn dialogue_scene_starts_at_entry_node() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::StartDialogue {
                scene_id: "demo_morning".to_string(),
            })
            .unwrap();

        assert_eq!(
            world.active_dialogue_scene_id,
            Some("demo_morning".to_string())
        );
        assert_eq!(world.active_dialogue.len(), 1);
        assert_eq!(world.active_dialogue[0].choices.len(), 3);
        let visible = world.visible_choices(&world.active_dialogue[0]).unwrap();
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn choosing_dialogue_advances_node_and_applies_effects() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::StartDialogue {
                scene_id: "demo_morning".to_string(),
            })
            .unwrap();
        world
            .apply_command(EngineCommand::ChooseDialogue {
                node_id: "demo_morning_001".to_string(),
                choice_id: "encourage".to_string(),
            })
            .unwrap();

        assert_eq!(world.active_dialogue.len(), 2);
        assert_eq!(world.active_dialogue[1].id, "demo_morning_003");
        assert_eq!(world.characters[0].state.mood, 13);
        assert_eq!(world.relationships[0].affinity, 7);
        assert_eq!(world.relationships[0].trust, 1);
    }

    #[test]
    fn dialogue_effect_rolls_character_state_deterministically() {
        let mut left = WorldState::bootstrap_demo();
        let mut right = WorldState::bootstrap_demo();
        left.dialogue_scenes[0].nodes[0]
            .choices
            .push(DialogueChoice {
                id: "random_state".to_string(),
                label: "随机状态".to_string(),
                next_node_id: None,
                conditions: Vec::new(),
                effects: vec![DialogueEffect::RollCharacterState {
                    character_id: "demo_heroine".to_string(),
                    energy_min_delta: -3,
                    energy_max_delta: 0,
                    mood_min_delta: -2,
                    mood_max_delta: 4,
                }],
            });
        right.dialogue_scenes = left.dialogue_scenes.clone();

        for world in [&mut left, &mut right] {
            world
                .apply_command(EngineCommand::StartDialogue {
                    scene_id: "demo_morning".to_string(),
                })
                .unwrap();
            world
                .apply_command(EngineCommand::ChooseDialogue {
                    node_id: "demo_morning_001".to_string(),
                    choice_id: "random_state".to_string(),
                })
                .unwrap();
        }

        assert_eq!(left, right);
        assert_eq!(left.random.cursor, 2);
        assert!((77..=80).contains(&left.characters[0].state.energy));
        assert!((8..=14).contains(&left.characters[0].state.mood));
    }

    #[test]
    fn invalid_dialogue_random_effect_is_transactional() {
        let mut world = WorldState::bootstrap_demo();
        world.dialogue_scenes[0].nodes[0]
            .choices
            .push(DialogueChoice {
                id: "invalid_random_state".to_string(),
                label: "非法随机状态".to_string(),
                next_node_id: None,
                conditions: Vec::new(),
                effects: vec![DialogueEffect::RollCharacterState {
                    character_id: "demo_heroine".to_string(),
                    energy_min_delta: 1,
                    energy_max_delta: -1,
                    mood_min_delta: 0,
                    mood_max_delta: 1,
                }],
            });
        world
            .apply_command(EngineCommand::StartDialogue {
                scene_id: "demo_morning".to_string(),
            })
            .unwrap();
        let original = world.clone();

        let result = world.apply_command(EngineCommand::ChooseDialogue {
            node_id: "demo_morning_001".to_string(),
            choice_id: "invalid_random_state".to_string(),
        });

        assert_eq!(
            result,
            Err(EngineError::InvalidRandomRange {
                min_delta: 1,
                max_delta: -1,
            })
        );
        assert_eq!(world, original);
    }

    #[test]
    fn dialogue_choice_conditions_gate_selection_transactionally() {
        let mut world = WorldState::bootstrap_demo();
        world
            .apply_command(EngineCommand::StartDialogue {
                scene_id: "demo_morning".to_string(),
            })
            .unwrap();
        let original = world.clone();

        let result = world.apply_command(EngineCommand::ChooseDialogue {
            node_id: "demo_morning_001".to_string(),
            choice_id: "talk_about_trust".to_string(),
        });

        assert_eq!(
            result,
            Err(EngineError::DialogueChoiceConditionNotMet(
                "talk_about_trust".to_string()
            ))
        );
        assert_eq!(world, original);
    }

    #[test]
    fn dialogue_choice_condition_allows_unlocked_selection() {
        let mut world = WorldState::bootstrap_demo();
        world
            .apply_command(EngineCommand::AdjustRelationship {
                source_character_id: "player".to_string(),
                target_character_id: "demo_heroine".to_string(),
                affinity_delta: 2,
                trust_delta: 0,
            })
            .unwrap();
        world
            .apply_command(EngineCommand::StartDialogue {
                scene_id: "demo_morning".to_string(),
            })
            .unwrap();

        let visible = world.visible_choices(&world.active_dialogue[0]).unwrap();
        assert!(visible.iter().any(|choice| choice.id == "talk_about_trust"));

        world
            .apply_command(EngineCommand::ChooseDialogue {
                node_id: "demo_morning_001".to_string(),
                choice_id: "talk_about_trust".to_string(),
            })
            .unwrap();

        assert_eq!(world.active_dialogue[1].id, "demo_morning_004");
        assert_eq!(world.relationships[0].trust, 2);
    }

    #[test]
    fn invalid_dialogue_choice_is_transactional() {
        let mut world = WorldState::bootstrap_demo();
        world
            .apply_command(EngineCommand::StartDialogue {
                scene_id: "demo_morning".to_string(),
            })
            .unwrap();
        let original = world.clone();

        let result = world.apply_command(EngineCommand::ChooseDialogue {
            node_id: "demo_morning_001".to_string(),
            choice_id: "missing".to_string(),
        });

        assert_eq!(
            result,
            Err(EngineError::DialogueChoiceNotFound("missing".to_string()))
        );
        assert_eq!(world, original);
    }

    #[test]
    fn command_log_records_successful_commands_only() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 30 })
            .unwrap();
        let result = world.apply_command(EngineCommand::MoveCharacter {
            character_id: "demo_heroine".to_string(),
            location_id: "missing".to_string(),
        });

        assert_eq!(
            result,
            Err(EngineError::LocationNotFound("missing".to_string()))
        );
        assert_eq!(world.command_log.len(), 1);
        assert_eq!(
            world.command_log[0],
            EngineCommand::AdvanceTime { minutes: 30 }
        );
        assert_eq!(
            world.command_log_initial_random,
            Some(WorldRandom::default())
        );
    }

    #[test]
    fn adjust_relationship_command_updates_bounded_relationship() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::AdjustRelationship {
                source_character_id: "player".to_string(),
                target_character_id: "demo_heroine".to_string(),
                affinity_delta: 120,
                trust_delta: 3,
            })
            .unwrap();

        assert_eq!(world.relationships[0].affinity, 100);
        assert_eq!(world.relationships[0].trust, 3);
        assert_eq!(
            world.command_log[0],
            EngineCommand::AdjustRelationship {
                source_character_id: "player".to_string(),
                target_character_id: "demo_heroine".to_string(),
                affinity_delta: 120,
                trust_delta: 3,
            }
        );
    }

    #[test]
    fn missing_relationship_is_transactional() {
        let mut world = WorldState::bootstrap_demo();
        let original = world.clone();

        let result = world.apply_command(EngineCommand::AdjustRelationship {
            source_character_id: "player".to_string(),
            target_character_id: "missing".to_string(),
            affinity_delta: 1,
            trust_delta: 1,
        });

        assert_eq!(
            result,
            Err(EngineError::CharacterNotFound("missing".to_string()))
        );
        assert_eq!(world, original);
    }

    #[test]
    fn random_command_consumes_explicit_rng_state() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::RollCharacterMood {
                character_id: "demo_heroine".to_string(),
                min_delta: -5,
                max_delta: 5,
            })
            .unwrap();

        let mood = world.characters[0].state.mood;
        assert!((5..=15).contains(&mood));
        assert_eq!(world.random.cursor, 1);

        let replayed = replay_commands(WorldState::bootstrap_demo(), &world.command_log).unwrap();
        assert_eq!(replayed, world);
    }

    #[test]
    fn replay_log_records_initial_rng_state_for_deterministic_replay() {
        let mut world = WorldState::bootstrap_demo();
        world.random = WorldRandom {
            seed: 987_654_321,
            cursor: 7,
        };

        world
            .apply_command(EngineCommand::RollCharacterMood {
                character_id: "demo_heroine".to_string(),
                min_delta: -5,
                max_delta: 5,
            })
            .unwrap();
        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 30 })
            .unwrap();

        let replay_log = world.replay_log();
        let replayed = replay_command_log(WorldState::bootstrap_demo(), &replay_log).unwrap();

        assert_eq!(replay_log.schema_version, REPLAY_LOG_SCHEMA_VERSION);
        assert_eq!(
            replay_log.initial_random,
            WorldRandom {
                seed: 987_654_321,
                cursor: 7,
            }
        );
        assert_eq!(replayed, world);
    }

    #[test]
    fn invalid_random_range_is_transactional() {
        let mut world = WorldState::bootstrap_demo();
        let original = world.clone();

        let result = world.apply_command(EngineCommand::RollCharacterMood {
            character_id: "demo_heroine".to_string(),
            min_delta: 5,
            max_delta: -5,
        });

        assert_eq!(
            result,
            Err(EngineError::InvalidRandomRange {
                min_delta: 5,
                max_delta: -5
            })
        );
        assert_eq!(world, original);
    }

    #[test]
    fn missing_random_state_deserializes_with_default_seed() {
        let mut value = serde_json::to_value(WorldState::bootstrap_demo()).unwrap();
        value.as_object_mut().unwrap().remove("random");

        let decoded: WorldState = serde_json::from_value(value).unwrap();

        assert_eq!(decoded.random, WorldRandom::default());
    }

    #[test]
    fn missing_command_log_initial_random_deserializes_as_none() {
        let mut value = serde_json::to_value(WorldState::bootstrap_demo()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("command_log_initial_random");

        let decoded: WorldState = serde_json::from_value(value).unwrap();

        assert_eq!(decoded.command_log_initial_random, None);
    }

    #[test]
    fn missing_relationships_deserializes_as_empty_list() {
        let mut value = serde_json::to_value(WorldState::bootstrap_demo()).unwrap();
        value.as_object_mut().unwrap().remove("relationships");

        let decoded: WorldState = serde_json::from_value(value).unwrap();

        assert!(decoded.relationships.is_empty());
    }

    #[test]
    fn missing_resources_deserializes_as_empty_list() {
        let mut value = serde_json::to_value(WorldState::bootstrap_demo()).unwrap();
        value.as_object_mut().unwrap().remove("resources");

        let decoded: WorldState = serde_json::from_value(value).unwrap();

        assert!(decoded.resources.is_empty());
    }

    #[test]
    fn missing_installed_content_packages_deserializes_as_empty_list() {
        let mut value = serde_json::to_value(WorldState::bootstrap_demo()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("installed_content_packages");

        let decoded: WorldState = serde_json::from_value(value).unwrap();

        assert!(decoded.installed_content_packages.is_empty());
    }

    #[test]
    fn installed_content_package_constraints_deserialize_with_defaults() {
        let mut value = serde_json::to_value(WorldState::bootstrap_demo()).unwrap();
        value["installed_content_packages"] = serde_json::json!([
            {
                "namespace": "sample",
                "package_id": "sample.event_pack",
                "version": "0.1.0"
            },
            {
                "namespace": "sample",
                "package_id": "sample.addon",
                "version": "0.1.0",
                "dependencies": [
                    "sample.event_pack",
                    {
                        "packageId": "sample.optional",
                        "version": "0.2.0",
                        "required": false
                    }
                ],
                "conflicts": ["sample.conflict"]
            }
        ]);

        let decoded: WorldState = serde_json::from_value(value).unwrap();

        assert!(decoded.installed_content_packages[0]
            .dependencies
            .is_empty());
        assert!(decoded.installed_content_packages[0].conflicts.is_empty());
        assert_eq!(
            decoded.installed_content_packages[1].dependencies,
            vec![
                ContentPackageDependency {
                    package_id: "sample.event_pack".to_string(),
                    version: None,
                    required: true,
                },
                ContentPackageDependency {
                    package_id: "sample.optional".to_string(),
                    version: Some("0.2.0".to_string()),
                    required: false,
                },
            ]
        );
        assert_eq!(
            decoded.installed_content_packages[1].conflicts,
            vec!["sample.conflict".to_string()]
        );
    }

    #[test]
    fn missing_dialogue_resource_refs_deserializes_as_empty_list() {
        let mut value = serde_json::to_value(WorldState::bootstrap_demo()).unwrap();
        let scenes = value
            .get_mut("dialogue_scenes")
            .unwrap()
            .as_array_mut()
            .unwrap();
        for scene in scenes {
            let nodes = scene.get_mut("nodes").unwrap().as_array_mut().unwrap();
            for node in nodes {
                node.as_object_mut().unwrap().remove("resource_refs");
            }
        }

        let decoded: WorldState = serde_json::from_value(value).unwrap();

        assert!(decoded
            .dialogue_scenes
            .iter()
            .flat_map(|scene| &scene.nodes)
            .all(|node| node.resource_refs.is_empty()));
    }

    #[test]
    fn missing_scheduled_event_priority_and_repeat_deserializes_defaults() {
        let mut value = serde_json::to_value(WorldState::bootstrap_demo()).unwrap();
        let scheduled_events = value
            .get_mut("scheduled_events")
            .unwrap()
            .as_array_mut()
            .unwrap();
        for event in scheduled_events {
            let object = event.as_object_mut().unwrap();
            object.remove("priority");
            object.remove("repeat");
        }

        let decoded: WorldState = serde_json::from_value(value).unwrap();

        assert!(decoded
            .scheduled_events
            .iter()
            .all(|event| event.priority == 0 && event.repeat.is_none()));
    }

    #[test]
    fn schedule_event_rejects_duplicate_ids_transactionally() {
        let mut world = WorldState::bootstrap_demo();
        let original = world.clone();

        let result = world.apply_command(EngineCommand::ScheduleEvent {
            event: ScheduledEvent {
                id: "demo_clouds_at_gate".to_string(),
                due: ScheduledTime::new(1, 10, 0),
                priority: 0,
                repeat: None,
                conditions: Vec::new(),
                kind: ScheduledEventKind::ChangeWeather {
                    weather: Weather::Rain,
                },
            },
        });

        assert_eq!(
            result,
            Err(EngineError::DuplicateScheduledEvent(
                "demo_clouds_at_gate".to_string()
            ))
        );
        assert_eq!(world, original);
    }

    #[test]
    fn replay_commands_is_deterministic() {
        let commands = vec![
            EngineCommand::AdvanceTime { minutes: 30 },
            EngineCommand::MoveCharacter {
                character_id: "demo_heroine".to_string(),
                location_id: "garden".to_string(),
            },
            EngineCommand::StartDialogue {
                scene_id: "demo_morning".to_string(),
            },
        ];

        let left = replay_commands(WorldState::bootstrap_demo(), &commands).unwrap();
        let right = replay_commands(WorldState::bootstrap_demo(), &commands).unwrap();

        assert_eq!(left, right);
        assert_eq!(left.clock.weather, Weather::Cloudy);
    }

    #[test]
    fn conditional_scheduled_event_waits_until_conditions_pass() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::ScheduleEvent {
                event: ScheduledEvent {
                    id: "trust_dialogue".to_string(),
                    due: ScheduledTime::new(1, 8, 10),
                    priority: 0,
                    repeat: None,
                    conditions: vec![DialogueCondition::RelationshipAffinityAtLeast {
                        source_character_id: "player".to_string(),
                        target_character_id: "demo_heroine".to_string(),
                        value: 7,
                    }],
                    kind: ScheduledEventKind::StartDialogue {
                        scene_id: "demo_morning".to_string(),
                    },
                },
            })
            .unwrap();
        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 10 })
            .unwrap();

        assert_eq!(world.active_dialogue_scene_id, None);
        assert!(world
            .scheduled_events
            .iter()
            .any(|event| event.id == "trust_dialogue"));

        world
            .apply_command(EngineCommand::AdjustRelationship {
                source_character_id: "player".to_string(),
                target_character_id: "demo_heroine".to_string(),
                affinity_delta: 2,
                trust_delta: 0,
            })
            .unwrap();
        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 1 })
            .unwrap();

        assert_eq!(
            world.active_dialogue_scene_id,
            Some("demo_morning".to_string())
        );
        assert!(!world
            .scheduled_events
            .iter()
            .any(|event| event.id == "trust_dialogue"));
    }

    #[test]
    fn scheduled_events_use_priority_for_same_due_time() {
        let mut world = WorldState::bootstrap_demo();
        world.scheduled_events.clear();

        world
            .apply_command(EngineCommand::ScheduleEvent {
                event: ScheduledEvent {
                    id: "low_priority".to_string(),
                    due: ScheduledTime::new(1, 8, 10),
                    priority: 0,
                    repeat: None,
                    conditions: Vec::new(),
                    kind: ScheduledEventKind::AdjustCharacterState {
                        character_id: "demo_heroine".to_string(),
                        energy_delta: 0,
                        mood_delta: 1,
                    },
                },
            })
            .unwrap();
        world
            .apply_command(EngineCommand::ScheduleEvent {
                event: ScheduledEvent {
                    id: "high_priority".to_string(),
                    due: ScheduledTime::new(1, 8, 10),
                    priority: 10,
                    repeat: None,
                    conditions: Vec::new(),
                    kind: ScheduledEventKind::AdjustCharacterState {
                        character_id: "demo_heroine".to_string(),
                        energy_delta: 0,
                        mood_delta: 1,
                    },
                },
            })
            .unwrap();

        assert_eq!(world.scheduled_events[0].id, "high_priority");

        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 10 })
            .unwrap();

        let high_index = world
            .event_log
            .iter()
            .position(|entry| entry.contains("high_priority"))
            .unwrap();
        let low_index = world
            .event_log
            .iter()
            .position(|entry| entry.contains("low_priority"))
            .unwrap();
        assert!(high_index < low_index);
    }

    #[test]
    fn cancel_event_removes_scheduled_event_transactionally() {
        let mut world = WorldState::bootstrap_demo();

        world
            .apply_command(EngineCommand::CancelEvent {
                event_id: "demo_clouds_at_gate".to_string(),
            })
            .unwrap();

        assert!(!world
            .scheduled_events
            .iter()
            .any(|event| event.id == "demo_clouds_at_gate"));
        assert_eq!(
            world.command_log[0],
            EngineCommand::CancelEvent {
                event_id: "demo_clouds_at_gate".to_string()
            }
        );

        let original = world.clone();
        let result = world.apply_command(EngineCommand::CancelEvent {
            event_id: "missing".to_string(),
        });

        assert_eq!(
            result,
            Err(EngineError::ScheduledEventNotFound("missing".to_string()))
        );
        assert_eq!(world, original);
    }

    #[test]
    fn repeating_event_catches_up_and_exhausts_remaining_runs() {
        let mut world = WorldState::bootstrap_demo();
        world.scheduled_events.clear();

        world
            .apply_command(EngineCommand::ScheduleEvent {
                event: ScheduledEvent {
                    id: "morning_tick".to_string(),
                    due: ScheduledTime::new(1, 8, 10),
                    priority: 0,
                    repeat: Some(ScheduledRepeat {
                        every_minutes: 10,
                        remaining_runs: Some(2),
                    }),
                    conditions: Vec::new(),
                    kind: ScheduledEventKind::AdjustCharacterState {
                        character_id: "demo_heroine".to_string(),
                        energy_delta: 0,
                        mood_delta: 2,
                    },
                },
            })
            .unwrap();

        world
            .apply_command(EngineCommand::AdvanceTime { minutes: 30 })
            .unwrap();

        assert_eq!(world.characters[0].state.mood, 14);
        assert!(!world
            .scheduled_events
            .iter()
            .any(|event| event.id == "morning_tick"));
        assert_eq!(
            world
                .event_log
                .iter()
                .filter(|entry| entry.contains("morning_tick"))
                .count(),
            2
        );
    }

    #[test]
    fn invalid_repeating_event_is_transactional() {
        let mut world = WorldState::bootstrap_demo();
        let original = world.clone();

        let result = world.apply_command(EngineCommand::ScheduleEvent {
            event: ScheduledEvent {
                id: "bad_repeat".to_string(),
                due: ScheduledTime::new(1, 8, 10),
                priority: 0,
                repeat: Some(ScheduledRepeat {
                    every_minutes: 0,
                    remaining_runs: None,
                }),
                conditions: Vec::new(),
                kind: ScheduledEventKind::ChangeWeather {
                    weather: Weather::Rain,
                },
            },
        });

        assert_eq!(
            result,
            Err(EngineError::InvalidScheduledRepeat(
                "bad_repeat".to_string()
            ))
        );
        assert_eq!(world, original);
    }
}
