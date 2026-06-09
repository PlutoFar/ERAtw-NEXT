use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldClock {
    pub day: u32,
    pub hour: u8,
    pub minute: u8,
    pub season: Season,
    pub weather: Weather,
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
pub struct WorldState {
    pub engine_version: String,
    pub clock: WorldClock,
    pub locations: Vec<Location>,
    pub characters: Vec<Character>,
    pub active_dialogue: Vec<DialogueNode>,
    pub event_log: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineCommand {
    AdvanceTime { minutes: u16 },
    MoveCharacter {
        character_id: String,
        location_id: String,
    },
    StartDialogue { scene_id: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EngineError {
    #[error("character not found: {0}")]
    CharacterNotFound(String),
    #[error("location not found: {0}")]
    LocationNotFound(String),
    #[error("scene not found: {0}")]
    SceneNotFound(String),
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
            event_log: vec!["ERAtw-NEXT M0 engine ready.".to_string()],
        }
    }

    pub fn apply_command(&mut self, command: EngineCommand) -> Result<(), EngineError> {
        match command {
            EngineCommand::AdvanceTime { minutes } => {
                self.advance_time(minutes);
                Ok(())
            }
            EngineCommand::MoveCharacter {
                character_id,
                location_id,
            } => self.move_character(&character_id, &location_id),
            EngineCommand::StartDialogue { scene_id } => self.start_dialogue(&scene_id),
        }
    }

    fn advance_time(&mut self, minutes: u16) {
        let total = u16::from(self.clock.hour) * 60 + u16::from(self.clock.minute) + minutes;
        let days = total / (24 * 60);
        let minute_of_day = total % (24 * 60);
        self.clock.day += u32::from(days);
        self.clock.hour = (minute_of_day / 60) as u8;
        self.clock.minute = (minute_of_day % 60) as u8;
        self.event_log.push(format!("时间推进 {} 分钟。", minutes));
    }

    fn move_character(
        &mut self,
        character_id: &str,
        location_id: &str,
    ) -> Result<(), EngineError> {
        if !self.locations.iter().any(|location| location.id == location_id) {
            return Err(EngineError::LocationNotFound(location_id.to_string()));
        }

        let character = self
            .characters
            .iter_mut()
            .find(|character| character.id == character_id)
            .ok_or_else(|| EngineError::CharacterNotFound(character_id.to_string()))?;

        character.location_id = location_id.to_string();
        self.event_log
            .push(format!("{} 移动到 {}。", character.display_name, location_id));
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
    fn move_character_rejects_missing_location() {
        let mut world = WorldState::bootstrap_demo();

        let result = world.apply_command(EngineCommand::MoveCharacter {
            character_id: "demo_heroine".to_string(),
            location_id: "missing".to_string(),
        });

        assert_eq!(result, Err(EngineError::LocationNotFound("missing".to_string())));
    }
}
