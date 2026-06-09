use crate::{WorldState, ENGINE_VERSION};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SAVE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveEnvelope {
    pub schema_version: u32,
    pub engine_version: String,
    pub saved_at_unix_ms: u64,
    pub slot_id: String,
    pub mod_dependencies: Vec<SaveModDependency>,
    pub world: WorldState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveModDependency {
    pub namespace: String,
    pub version: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveValidationReport {
    pub missing_required_mods: Vec<SaveModDependency>,
    pub incompatible_schema: Option<u32>,
    pub engine_version_mismatch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveBackupPlan {
    pub primary_path: String,
    pub backup_path: String,
    pub reason: SaveBackupReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveBackupReason {
    BeforeOverwrite,
    BeforeMigration,
    BeforeRecovery,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SaveError {
    #[error("save schema {0} is newer than supported schema 1")]
    UnsupportedFutureSchema(u32),
    #[error("save slot id is required")]
    MissingSlotId,
    #[error("save world has no locations")]
    EmptyLocations,
    #[error("save world has no characters")]
    EmptyCharacters,
}

impl SaveEnvelope {
    pub fn new(slot_id: impl Into<String>, world: WorldState, saved_at_unix_ms: u64) -> Self {
        Self {
            schema_version: SAVE_SCHEMA_VERSION,
            engine_version: ENGINE_VERSION.to_string(),
            saved_at_unix_ms,
            slot_id: slot_id.into(),
            mod_dependencies: Vec::new(),
            world,
        }
    }

    pub fn validate(
        &self,
        enabled_mods: &[SaveModDependency],
    ) -> Result<SaveValidationReport, SaveError> {
        if self.schema_version > SAVE_SCHEMA_VERSION {
            return Err(SaveError::UnsupportedFutureSchema(self.schema_version));
        }
        if self.slot_id.trim().is_empty() {
            return Err(SaveError::MissingSlotId);
        }
        if self.world.locations.is_empty() {
            return Err(SaveError::EmptyLocations);
        }
        if self.world.characters.is_empty() {
            return Err(SaveError::EmptyCharacters);
        }

        let missing_required_mods = self
            .mod_dependencies
            .iter()
            .filter(|dependency| {
                dependency.required
                    && !enabled_mods.iter().any(|enabled| {
                        enabled.namespace == dependency.namespace
                            && enabled.version == dependency.version
                    })
            })
            .cloned()
            .collect();

        Ok(SaveValidationReport {
            missing_required_mods,
            incompatible_schema: (self.schema_version != SAVE_SCHEMA_VERSION)
                .then_some(self.schema_version),
            engine_version_mismatch: self.engine_version != ENGINE_VERSION,
        })
    }

    pub fn migrate_to_current(mut self) -> Result<Self, SaveError> {
        if self.schema_version > SAVE_SCHEMA_VERSION {
            return Err(SaveError::UnsupportedFutureSchema(self.schema_version));
        }

        self.schema_version = SAVE_SCHEMA_VERSION;
        self.engine_version = ENGINE_VERSION.to_string();
        Ok(self)
    }
}

pub fn backup_plan(
    primary_path: impl Into<String>,
    timestamp_unix_ms: u64,
    reason: SaveBackupReason,
) -> SaveBackupPlan {
    let primary_path = primary_path.into();
    SaveBackupPlan {
        backup_path: format!("{primary_path}.{timestamp_unix_ms}.bak"),
        primary_path,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorldState;

    #[test]
    fn save_envelope_round_trips_as_json() {
        let save = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 123);

        let encoded = serde_json::to_string(&save).unwrap();
        let decoded: SaveEnvelope = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, save);
    }

    #[test]
    fn validation_reports_missing_required_mods() {
        let mut save = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 123);
        save.mod_dependencies.push(SaveModDependency {
            namespace: "example.required".to_string(),
            version: "1.0.0".to_string(),
            required: true,
        });

        let report = save.validate(&[]).unwrap();

        assert_eq!(report.missing_required_mods.len(), 1);
        assert_eq!(
            report.missing_required_mods[0].namespace,
            "example.required"
        );
    }

    #[test]
    fn migration_rejects_future_schema() {
        let mut save = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 123);
        save.schema_version = SAVE_SCHEMA_VERSION + 1;

        let result = save.migrate_to_current();

        assert_eq!(
            result,
            Err(SaveError::UnsupportedFutureSchema(SAVE_SCHEMA_VERSION + 1))
        );
    }

    #[test]
    fn backup_plan_is_deterministic() {
        let plan = backup_plan("saves/slot-1.json", 42, SaveBackupReason::BeforeOverwrite);

        assert_eq!(plan.backup_path, "saves/slot-1.json.42.bak");
        assert_eq!(plan.reason, SaveBackupReason::BeforeOverwrite);
    }
}
