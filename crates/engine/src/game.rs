use crate::{EngineError, LoadedContentPackage, PackageIdentity};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

pub const GAME_STATE_SCHEMA_VERSION: &str = "game-state/v1";
const MAX_COMMAND_MINUTES: u16 = 1440;
const MAX_TOTAL_MINUTES: u64 = u32::MAX as u64 * 1440 - 1;
#[cfg(test)]
const GAME_STATE_SCHEMA: &str = include_str!("../../../schemas/game-state.schema.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct GameClock {
    pub day: u32,
    pub minute_of_day: u16,
    pub total_minutes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PlayerState {
    pub energy: u16,
    pub max_energy: u16,
    pub money: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ScheduledEvent {
    pub id: String,
    pub due_at: u64,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EventRecord {
    pub id: String,
    pub occurred_at: u64,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct GameState {
    pub schema_version: String,
    pub package: PackageIdentity,
    pub turn: u64,
    pub clock: GameClock,
    pub current_location_id: String,
    pub player: PlayerState,
    pub flags: BTreeMap<String, i64>,
    pub event_queue: Vec<ScheduledEvent>,
    pub recent_events: Vec<EventRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum GameCommand {
    Wait {
        minutes: u16,
    },
    Rest {
        minutes: u16,
    },
    Move {
        location_id: String,
        minutes: u16,
    },
    SetFlag {
        key: String,
        value: i64,
    },
    ScheduleEvent {
        event_id: String,
        due_in_minutes: u16,
        kind: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub state: GameState,
    pub emitted_events: Vec<EventRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameContext {
    pub package: PackageIdentity,
    pub locations: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub context: GameContext,
    pub initial_state: GameState,
    pub state: GameState,
    pub command_log: Vec<GameCommand>,
}

impl GameContext {
    pub fn from_package(package: &LoadedContentPackage) -> Self {
        Self {
            package: package.index.package.clone(),
            locations: package
                .runtime_locations
                .iter()
                .map(|location| (location.id.clone(), location.connections.clone()))
                .collect(),
        }
    }
}

impl GameSession {
    pub fn from_package(package: &LoadedContentPackage) -> Result<Self, EngineError> {
        let context = GameContext::from_package(package);
        let initial_state = new_game(package)?;
        Ok(Self {
            context,
            state: initial_state.clone(),
            initial_state,
            command_log: Vec::new(),
        })
    }

    pub fn apply(&mut self, command: GameCommand) -> Result<CommandResult, EngineError> {
        let result = apply_command(&self.context, &self.state, &command)?;
        self.state = result.state.clone();
        self.command_log.push(command);
        Ok(result)
    }
}

pub fn new_game(package: &LoadedContentPackage) -> Result<GameState, EngineError> {
    if !package.index.playable {
        return Err(game_error(
            "GAME_PACKAGE_NOT_PLAYABLE",
            "Loaded content package is not marked playable.",
            json!({ "packageId": package.manifest.package_id }),
        ));
    }
    let start = package
        .runtime_locations
        .first()
        .ok_or_else(|| {
            game_error(
                "GAME_START_LOCATION_MISSING",
                "Playable content package has no start location.",
                json!({ "packageId": package.manifest.package_id }),
            )
        })?
        .id
        .clone();
    Ok(GameState {
        schema_version: GAME_STATE_SCHEMA_VERSION.to_string(),
        package: package.index.package.clone(),
        turn: 0,
        clock: GameClock {
            day: 1,
            minute_of_day: 360,
            total_minutes: 360,
        },
        current_location_id: start,
        player: PlayerState {
            energy: 100,
            max_energy: 100,
            money: 100,
        },
        flags: BTreeMap::new(),
        event_queue: vec![ScheduledEvent {
            id: "system.daybreak.1".to_string(),
            due_at: 480,
            kind: "daybreak".to_string(),
        }],
        recent_events: Vec::new(),
    })
}

pub fn apply_command(
    context: &GameContext,
    state: &GameState,
    command: &GameCommand,
) -> Result<CommandResult, EngineError> {
    validate_game_state(context, state)?;
    if state.turn == u64::MAX {
        return Err(game_error(
            "GAME_TURN_OVERFLOW",
            "Game turn cannot be advanced further.",
            json!({ "turn": state.turn }),
        ));
    }
    let mut next = state.clone();
    next.recent_events.clear();

    match command {
        GameCommand::Wait { minutes } => {
            validate_minutes(*minutes)?;
            advance_time(&mut next, *minutes)?;
        }
        GameCommand::Rest { minutes } => {
            validate_minutes(*minutes)?;
            advance_time(&mut next, *minutes)?;
            let recovered = minutes.saturating_div(5).max(1);
            next.player.energy = next
                .player
                .energy
                .saturating_add(recovered)
                .min(next.player.max_energy);
        }
        GameCommand::Move {
            location_id,
            minutes,
        } => {
            validate_minutes(*minutes)?;
            let connections = context
                .locations
                .get(&state.current_location_id)
                .ok_or_else(|| {
                    game_error(
                        "GAME_LOCATION_UNKNOWN",
                        "Current location is absent from the loaded package.",
                        json!({ "locationId": state.current_location_id }),
                    )
                })?;
            if !connections.contains(location_id) {
                return Err(game_error(
                    "GAME_LOCATION_NOT_CONNECTED",
                    "Target location is not connected to the current location.",
                    json!({
                        "from": state.current_location_id,
                        "to": location_id
                    }),
                ));
            }
            let energy_cost = minutes.saturating_add(9) / 10;
            if next.player.energy < energy_cost {
                return Err(game_error(
                    "GAME_ENERGY_INSUFFICIENT",
                    "Player does not have enough energy to move.",
                    json!({
                        "required": energy_cost,
                        "available": next.player.energy
                    }),
                ));
            }
            next.player.energy -= energy_cost;
            next.current_location_id = location_id.clone();
            advance_time(&mut next, *minutes)?;
        }
        GameCommand::SetFlag { key, value } => {
            if key.trim().is_empty() || key.len() > 128 {
                return Err(game_error(
                    "GAME_FLAG_INVALID",
                    "Flag key must contain 1 to 128 characters.",
                    json!({ "key": key }),
                ));
            }
            next.flags.insert(key.clone(), *value);
        }
        GameCommand::ScheduleEvent {
            event_id,
            due_in_minutes,
            kind,
        } => {
            validate_minutes(*due_in_minutes)?;
            if event_id.trim().is_empty() || kind.trim().is_empty() {
                return Err(game_error(
                    "GAME_EVENT_INVALID",
                    "Event ID and kind must not be empty.",
                    json!({ "eventId": event_id, "kind": kind }),
                ));
            }
            if next.event_queue.iter().any(|event| event.id == *event_id) {
                return Err(game_error(
                    "GAME_EVENT_DUPLICATE",
                    "Event queue already contains this event ID.",
                    json!({ "eventId": event_id }),
                ));
            }
            let due_at = next
                .clock
                .total_minutes
                .checked_add(u64::from(*due_in_minutes))
                .filter(|value| *value <= MAX_TOTAL_MINUTES)
                .ok_or_else(|| {
                    game_error(
                        "GAME_TIME_OVERFLOW",
                        "Scheduled event time exceeds the supported game clock.",
                        json!({
                            "totalMinutes": next.clock.total_minutes,
                            "dueInMinutes": due_in_minutes
                        }),
                    )
                })?;
            next.event_queue.push(ScheduledEvent {
                id: event_id.clone(),
                due_at,
                kind: kind.clone(),
            });
            sort_events(&mut next.event_queue);
        }
    }

    next.turn += 1;
    process_due_events(&mut next);
    Ok(CommandResult {
        emitted_events: next.recent_events.clone(),
        state: next,
    })
}

pub fn replay_commands(
    context: &GameContext,
    initial_state: &GameState,
    commands: &[GameCommand],
) -> Result<GameState, EngineError> {
    validate_game_state(context, initial_state)?;
    let mut state = initial_state.clone();
    for command in commands {
        state = apply_command(context, &state, command)?.state;
    }
    Ok(state)
}

pub(crate) fn validate_game_state(
    context: &GameContext,
    state: &GameState,
) -> Result<(), EngineError> {
    if state.schema_version != GAME_STATE_SCHEMA_VERSION {
        return Err(game_error(
            "GAME_STATE_VERSION_UNSUPPORTED",
            "Game state schema version is not supported.",
            json!({
                "expected": GAME_STATE_SCHEMA_VERSION,
                "actual": state.schema_version
            }),
        ));
    }
    if state.package != context.package {
        return Err(game_error(
            "GAME_PACKAGE_MISMATCH",
            "Game state belongs to a different content package.",
            json!({
                "expected": context.package.package_id,
                "actual": state.package.package_id
            }),
        ));
    }
    if !context.locations.contains_key(&state.current_location_id) {
        return Err(game_error(
            "GAME_LOCATION_UNKNOWN",
            "Game state current location is absent from the loaded package.",
            json!({ "locationId": state.current_location_id }),
        ));
    }
    let expected_day = state.clock.total_minutes / 1440 + 1;
    let expected_minute = state.clock.total_minutes % 1440;
    if state.clock.total_minutes > MAX_TOTAL_MINUTES
        || u64::from(state.clock.day) != expected_day
        || u64::from(state.clock.minute_of_day) != expected_minute
    {
        return Err(game_error(
            "GAME_STATE_INVALID",
            "Game clock fields are inconsistent.",
            json!({
                "clock": state.clock,
                "maxTotalMinutes": MAX_TOTAL_MINUTES
            }),
        ));
    }
    if state.player.max_energy == 0 || state.player.energy > state.player.max_energy {
        return Err(game_error(
            "GAME_STATE_INVALID",
            "Player energy bounds are invalid.",
            json!({
                "energy": state.player.energy,
                "maxEnergy": state.player.max_energy
            }),
        ));
    }
    if let Some(key) = state
        .flags
        .keys()
        .find(|key| key.trim().is_empty() || key.len() > 128)
    {
        return Err(game_error(
            "GAME_STATE_INVALID",
            "Game state contains an invalid flag key.",
            json!({ "key": key }),
        ));
    }

    let mut event_ids = BTreeSet::new();
    for (index, event) in state.event_queue.iter().enumerate() {
        if event.id.trim().is_empty()
            || event.kind.trim().is_empty()
            || event.due_at <= state.clock.total_minutes
            || event.due_at > MAX_TOTAL_MINUTES
            || !event_ids.insert(event.id.clone())
        {
            return Err(game_error(
                "GAME_STATE_INVALID",
                "Game state contains an invalid scheduled event.",
                json!({ "index": index, "event": event }),
            ));
        }
    }
    if state
        .event_queue
        .windows(2)
        .any(|pair| event_order(&pair[0], &pair[1]).is_gt())
    {
        return Err(game_error(
            "GAME_STATE_INVALID",
            "Scheduled events are not in deterministic order.",
            json!({}),
        ));
    }
    if let Some(event) = state.recent_events.iter().find(|event| {
        event.id.trim().is_empty()
            || event.kind.trim().is_empty()
            || event.occurred_at > state.clock.total_minutes
    }) {
        return Err(game_error(
            "GAME_STATE_INVALID",
            "Game state contains an invalid recent event.",
            json!({ "event": event }),
        ));
    }
    Ok(())
}

fn validate_minutes(minutes: u16) -> Result<(), EngineError> {
    if minutes == 0 || minutes > MAX_COMMAND_MINUTES {
        return Err(game_error(
            "GAME_MINUTES_INVALID",
            "Command minutes must be between 1 and 1440.",
            json!({ "minutes": minutes }),
        ));
    }
    Ok(())
}

fn advance_time(state: &mut GameState, minutes: u16) -> Result<(), EngineError> {
    let total_minutes = state
        .clock
        .total_minutes
        .checked_add(u64::from(minutes))
        .filter(|value| *value <= MAX_TOTAL_MINUTES)
        .ok_or_else(|| {
            game_error(
                "GAME_TIME_OVERFLOW",
                "Game clock cannot be advanced further.",
                json!({
                    "totalMinutes": state.clock.total_minutes,
                    "minutes": minutes,
                    "maxTotalMinutes": MAX_TOTAL_MINUTES
                }),
            )
        })?;
    state.clock.total_minutes = total_minutes;
    state.clock.day = (total_minutes / 1440 + 1) as u32;
    state.clock.minute_of_day = (total_minutes % 1440) as u16;
    Ok(())
}

fn process_due_events(state: &mut GameState) {
    sort_events(&mut state.event_queue);
    let split = state
        .event_queue
        .partition_point(|event| event.due_at <= state.clock.total_minutes);
    let due: Vec<_> = state.event_queue.drain(..split).collect();
    state.recent_events = due
        .into_iter()
        .map(|event| EventRecord {
            id: event.id,
            occurred_at: event.due_at,
            kind: event.kind,
        })
        .collect();
}

fn sort_events(events: &mut [ScheduledEvent]) {
    events.sort_by(event_order);
}

fn event_order(left: &ScheduledEvent, right: &ScheduledEvent) -> std::cmp::Ordering {
    left.due_at
        .cmp(&right.due_at)
        .then_with(|| left.id.cmp(&right.id))
}

fn game_error(code: &str, message: &str, details: serde_json::Value) -> EngineError {
    EngineError::new(code, message, details)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::tests::create_playable_package;
    use crate::load_content_package;
    use std::fs;

    #[test]
    fn replay_is_deterministic() {
        let root = create_playable_package("replay");
        let package = load_content_package(&root).unwrap();
        let context = GameContext::from_package(&package);
        let initial = new_game(&package).unwrap();
        let commands = vec![
            GameCommand::Wait { minutes: 30 },
            GameCommand::Move {
                location_id: "core.location.square".to_string(),
                minutes: 10,
            },
            GameCommand::SetFlag {
                key: "story.intro".to_string(),
                value: 1,
            },
            GameCommand::Rest { minutes: 90 },
        ];
        let first = replay_commands(&context, &initial, &commands).unwrap();
        let second = replay_commands(&context, &initial, &commands).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.current_location_id, "core.location.square");
        assert_eq!(first.clock.total_minutes, 490);
        assert_eq!(first.recent_events[0].kind, "daybreak");
        let schema: serde_json::Value = serde_json::from_str(GAME_STATE_SCHEMA).unwrap();
        let compiled = jsonschema::JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .compile(&schema)
            .unwrap();
        assert!(compiled
            .validate(&serde_json::to_value(&first).unwrap())
            .is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reducer_rejects_disconnected_move() {
        let root = create_playable_package("bad-move");
        let package = load_content_package(&root).unwrap();
        let context = GameContext::from_package(&package);
        let initial = new_game(&package).unwrap();
        let error = apply_command(
            &context,
            &initial,
            &GameCommand::Move {
                location_id: "core.location.missing".to_string(),
                minutes: 10,
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "GAME_LOCATION_NOT_CONNECTED");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reducer_rejects_inconsistent_clock_and_zero_max_energy() {
        let root = create_playable_package("bad-state");
        let package = load_content_package(&root).unwrap();
        let context = GameContext::from_package(&package);
        let mut state = new_game(&package).unwrap();

        state.clock.day = 2;
        let clock_error =
            apply_command(&context, &state, &GameCommand::Wait { minutes: 10 }).unwrap_err();
        assert_eq!(clock_error.code, "GAME_STATE_INVALID");

        state.clock.day = 1;
        state.player.energy = 0;
        state.player.max_energy = 0;
        let energy_error =
            apply_command(&context, &state, &GameCommand::Wait { minutes: 10 }).unwrap_err();
        assert_eq!(energy_error.code, "GAME_STATE_INVALID");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reducer_reports_time_and_turn_overflow() {
        let root = create_playable_package("overflow");
        let package = load_content_package(&root).unwrap();
        let context = GameContext::from_package(&package);
        let mut state = new_game(&package).unwrap();
        state.event_queue.clear();
        state.clock.total_minutes = u64::from(u32::MAX) * 1440 - 1;
        state.clock.day = u32::MAX;
        state.clock.minute_of_day = 1439;

        let time_error =
            apply_command(&context, &state, &GameCommand::Wait { minutes: 1 }).unwrap_err();
        assert_eq!(time_error.code, "GAME_TIME_OVERFLOW");

        state.clock = GameClock {
            day: 1,
            minute_of_day: 360,
            total_minutes: 360,
        };
        state.turn = u64::MAX;
        let turn_error =
            apply_command(&context, &state, &GameCommand::Wait { minutes: 1 }).unwrap_err();
        assert_eq!(turn_error.code, "GAME_TURN_OVERFLOW");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reducer_rejects_invalid_event_queue() {
        let root = create_playable_package("bad-events");
        let package = load_content_package(&root).unwrap();
        let context = GameContext::from_package(&package);
        let mut state = new_game(&package).unwrap();
        state.event_queue.push(ScheduledEvent {
            id: "system.daybreak.1".to_string(),
            due_at: 300,
            kind: "duplicate".to_string(),
        });

        let error =
            apply_command(&context, &state, &GameCommand::Wait { minutes: 10 }).unwrap_err();
        assert_eq!(error.code, "GAME_STATE_INVALID");
        fs::remove_dir_all(root).unwrap();
    }
}
