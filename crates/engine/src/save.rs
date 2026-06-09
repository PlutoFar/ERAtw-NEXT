use crate::{InstalledContentPackage, WorldState, ENGINE_VERSION};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
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
pub struct SaveWriteReport {
    pub path: PathBuf,
    pub backup_path: Option<PathBuf>,
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
    #[error("save path has no parent directory")]
    MissingParentDirectory,
    #[error("save io error: {0}")]
    Io(String),
    #[error("save json error: {0}")]
    Json(String),
}

impl From<io::Error> for SaveError {
    fn from(error: io::Error) -> Self {
        SaveError::Io(error.to_string())
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(error: serde_json::Error) -> Self {
        SaveError::Json(error.to_string())
    }
}

impl SaveEnvelope {
    pub fn new(slot_id: impl Into<String>, world: WorldState, saved_at_unix_ms: u64) -> Self {
        let mod_dependencies = mod_dependencies_for_world(&world);

        Self {
            schema_version: SAVE_SCHEMA_VERSION,
            engine_version: ENGINE_VERSION.to_string(),
            saved_at_unix_ms,
            slot_id: slot_id.into(),
            mod_dependencies,
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

        let world_mod_dependencies = mod_dependencies_for_world(&self.world);
        let missing_required_mods = self
            .mod_dependencies
            .iter()
            .filter(|dependency| {
                dependency.required
                    && !mod_dependency_is_available(dependency, enabled_mods)
                    && !mod_dependency_is_available(dependency, &world_mod_dependencies)
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

pub fn mod_dependencies_for_world(world: &WorldState) -> Vec<SaveModDependency> {
    let mut dependencies = world
        .installed_content_packages
        .iter()
        .map(SaveModDependency::from)
        .collect::<Vec<_>>();
    dependencies.sort_by(|left, right| {
        left.namespace
            .cmp(&right.namespace)
            .then_with(|| left.version.cmp(&right.version))
    });
    dependencies
}

fn mod_dependency_is_available(
    dependency: &SaveModDependency,
    enabled_mods: &[SaveModDependency],
) -> bool {
    enabled_mods.iter().any(|enabled| {
        enabled.namespace == dependency.namespace && enabled.version == dependency.version
    })
}

impl From<&InstalledContentPackage> for SaveModDependency {
    fn from(package: &InstalledContentPackage) -> Self {
        Self {
            namespace: package.package_id.clone(),
            version: package.version.clone(),
            required: true,
        }
    }
}

pub fn write_save_atomic(
    path: impl AsRef<Path>,
    save: &SaveEnvelope,
    timestamp_unix_ms: u64,
) -> Result<SaveWriteReport, SaveError> {
    save.validate(&[])?;

    let path = path.as_ref();
    let parent = path.parent().ok_or(SaveError::MissingParentDirectory)?;
    fs::create_dir_all(parent)?;

    let backup_path = if path.exists() {
        let backup_path = backup_path_for(path, timestamp_unix_ms);
        fs::copy(path, &backup_path)?;
        Some(backup_path)
    } else {
        None
    };

    let temp_path = temp_path_for(path, timestamp_unix_ms);
    let encoded = serde_json::to_vec_pretty(save)?;
    fs::write(&temp_path, encoded)?;
    fs::rename(&temp_path, path).or_else(|error| {
        if path.exists() {
            fs::remove_file(path)?;
            fs::rename(&temp_path, path)?;
            Ok(())
        } else {
            Err(error)
        }
    })?;

    Ok(SaveWriteReport {
        path: path.to_path_buf(),
        backup_path,
    })
}

pub fn read_save(
    path: impl AsRef<Path>,
    enabled_mods: &[SaveModDependency],
) -> Result<SaveEnvelope, SaveError> {
    let encoded = fs::read(path)?;
    let save: SaveEnvelope = serde_json::from_slice(&encoded)?;
    let save = save.migrate_to_current()?;
    save.validate(enabled_mods)?;
    Ok(save)
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

fn backup_path_for(path: &Path, timestamp_unix_ms: u64) -> PathBuf {
    let mut backup = PathBuf::from(path);
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_string());
    backup.set_extension(match extension {
        Some(extension) if !extension.is_empty() => format!("{extension}.{timestamp_unix_ms}.bak"),
        _ => format!("{timestamp_unix_ms}.bak"),
    });
    backup
}

fn temp_path_for(path: &Path, timestamp_unix_ms: u64) -> PathBuf {
    let mut temp = PathBuf::from(path);
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_string());
    temp.set_extension(match extension {
        Some(extension) if !extension.is_empty() => format!("{extension}.{timestamp_unix_ms}.tmp"),
        _ => format!("{timestamp_unix_ms}.tmp"),
    });
    temp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorldState;
    use std::time::{SystemTime, UNIX_EPOCH};

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
    fn save_envelope_derives_installed_content_package_dependencies() {
        let mut world = WorldState::bootstrap_demo();
        world
            .installed_content_packages
            .push(InstalledContentPackage {
                namespace: "sample".to_string(),
                package_id: "sample.event_pack".to_string(),
                version: "0.1.0".to_string(),
            });

        let save = SaveEnvelope::new("slot-1", world, 123);
        let report = save.validate(&[]).unwrap();

        assert_eq!(
            save.mod_dependencies,
            vec![SaveModDependency {
                namespace: "sample.event_pack".to_string(),
                version: "0.1.0".to_string(),
                required: true,
            }]
        );
        assert!(report.missing_required_mods.is_empty());
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

    #[test]
    fn save_file_round_trips_through_atomic_writer() {
        let dir = temp_save_dir("round_trip");
        let path = dir.join("slot-1.json");
        let save = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 123);

        let report = write_save_atomic(&path, &save, 123).unwrap();
        let decoded = read_save(&path, &[]).unwrap();

        assert_eq!(report.path, path);
        assert_eq!(report.backup_path, None);
        assert_eq!(decoded, save);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_file_creates_backup_before_overwrite() {
        let dir = temp_save_dir("backup");
        let path = dir.join("slot-1.json");
        let first = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 100);
        let second = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 200);

        write_save_atomic(&path, &first, 100).unwrap();
        let report = write_save_atomic(&path, &second, 200).unwrap();

        let backup_path = report.backup_path.unwrap();
        let backup = read_save(&backup_path, &[]).unwrap();
        let current = read_save(&path, &[]).unwrap();

        assert_eq!(backup.saved_at_unix_ms, 100);
        assert_eq!(current.saved_at_unix_ms, 200);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_save_rejects_invalid_json() {
        let dir = temp_save_dir("invalid_json");
        let path = dir.join("slot-1.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, b"{broken").unwrap();

        let result = read_save(&path, &[]);

        assert!(matches!(result, Err(SaveError::Json(_))));

        let _ = fs::remove_dir_all(dir);
    }

    fn temp_save_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("eratw_next_save_{label}_{nonce}"))
    }
}
