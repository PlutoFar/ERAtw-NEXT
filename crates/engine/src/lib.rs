use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod save;

pub const ENGINE_VERSION: &str = "0.1.0-m0";

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
pub struct DialogueNode {
    pub id: String,
    pub speaker_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub id: String,
    pub due: ScheduledTime,
    pub kind: ScheduledEventKind,
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
    AdjustCharacterState {
        character_id: String,
        energy_delta: i16,
        mood_delta: i16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    pub engine_version: String,
    pub clock: WorldClock,
    pub locations: Vec<Location>,
    pub characters: Vec<Character>,
    pub active_dialogue: Vec<DialogueNode>,
    pub scheduled_events: Vec<ScheduledEvent>,
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
    StartDialogue {
        scene_id: String,
    },
    ScheduleEvent {
        event: ScheduledEvent,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EngineError {
    #[error("character not found: {0}")]
    CharacterNotFound(String),
    #[error("location not found: {0}")]
    LocationNotFound(String),
    #[error("scene not found: {0}")]
    SceneNotFound(String),
    #[error("scheduled event id is required")]
    ScheduledEventIdRequired,
    #[error("duplicate scheduled event: {0}")]
    DuplicateScheduledEvent(String),
    #[error("scheduled event has invalid due time: {0}")]
    InvalidScheduledTime(String),
}

impl WorldState {
    pub fn bootstrap_demo() -> Self {
        Self {
            engine_version: ENGINE_VERSION.to_string(),
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
            active_dialogue: Vec::new(),
            scheduled_events: vec![
                ScheduledEvent {
                    id: "demo_clouds_at_gate".to_string(),
                    due: ScheduledTime::new(1, 8, 30),
                    kind: ScheduledEventKind::ChangeWeather {
                        weather: Weather::Cloudy,
                    },
                },
                ScheduledEvent {
                    id: "demo_morning_mood".to_string(),
                    due: ScheduledTime::new(1, 9, 0),
                    kind: ScheduledEventKind::AdjustCharacterState {
                        character_id: "demo_heroine".to_string(),
                        energy_delta: -3,
                        mood_delta: 5,
                    },
                },
            ],
            event_log: vec!["ERAtw-NEXT M0 engine ready.".to_string()],
        }
    }

    pub fn apply_command(&mut self, command: EngineCommand) -> Result<(), EngineError> {
        let mut next = self.clone();
        next.apply_command_inner(command)?;
        *self = next;
        Ok(())
    }

    fn apply_command_inner(&mut self, command: EngineCommand) -> Result<(), EngineError> {
        match command {
            EngineCommand::AdvanceTime { minutes } => self.advance_time(minutes),
            EngineCommand::MoveCharacter {
                character_id,
                location_id,
            } => self.move_character(&character_id, &location_id),
            EngineCommand::StartDialogue { scene_id } => self.start_dialogue(&scene_id),
            EngineCommand::ScheduleEvent { event } => self.schedule_event(event),
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
        if scene_id != "demo_morning" {
            return Err(EngineError::SceneNotFound(scene_id.to_string()));
        }

        self.active_dialogue = vec![
            DialogueNode {
                id: "demo_morning_001".to_string(),
                speaker_id: "demo_heroine".to_string(),
                text: "早上好。今天先从一个干净的新世界开始。".to_string(),
            },
            DialogueNode {
                id: "demo_morning_002".to_string(),
                speaker_id: "system".to_string(),
                text: "该对话来自版本化 DialogueNode，不执行旧 ERB。".to_string(),
            },
        ];
        self.event_log.push("播放场景 demo_morning。".to_string());
        Ok(())
    }

    fn schedule_event(&mut self, event: ScheduledEvent) -> Result<(), EngineError> {
        if event.id.trim().is_empty() {
            return Err(EngineError::ScheduledEventIdRequired);
        }

        if !event.due.is_valid() {
            return Err(EngineError::InvalidScheduledTime(event.id));
        }

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

        self.scheduled_events = pending_events;
        due_events.sort_by(|left, right| {
            left.due
                .absolute_minute()
                .cmp(&right.due.absolute_minute())
                .then_with(|| left.id.cmp(&right.id))
        });

        for event in due_events {
            self.execute_scheduled_event(&event)?;
        }

        Ok(())
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
            ScheduledEventKind::AdjustCharacterState {
                character_id,
                energy_delta,
                mood_delta,
            } => {
                let character = self
                    .characters
                    .iter_mut()
                    .find(|character| character.id == *character_id)
                    .ok_or_else(|| EngineError::CharacterNotFound(character_id.clone()))?;

                character.state.energy =
                    bounded_delta(character.state.energy, *energy_delta, 0, 100);
                character.state.mood = bounded_delta(character.state.mood, *mood_delta, -100, 100);
                self.event_log.push(format!(
                    "事件 {} 触发：{} 状态更新。",
                    event.id, character.display_name
                ));
                Ok(())
            }
        }
    }

    fn sort_scheduled_events(&mut self) {
        self.scheduled_events.sort_by(|left, right| {
            left.due
                .absolute_minute()
                .cmp(&right.due.absolute_minute())
                .then_with(|| left.id.cmp(&right.id))
        });
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

fn bounded_delta(value: i16, delta: i16, min: i16, max: i16) -> i16 {
    value.saturating_add(delta).clamp(min, max)
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
    fn schedule_event_rejects_duplicate_ids_transactionally() {
        let mut world = WorldState::bootstrap_demo();
        let original = world.clone();

        let result = world.apply_command(EngineCommand::ScheduleEvent {
            event: ScheduledEvent {
                id: "demo_clouds_at_gate".to_string(),
                due: ScheduledTime::new(1, 10, 0),
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
}
