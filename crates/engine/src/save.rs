use crate::{EngineReplayLog, InstalledContentPackage, WorldState, ENGINE_VERSION};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};
use thiserror::Error;

pub const SAVE_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_SAVE_BACKUP_LIMIT: usize = 10;
pub const DEFAULT_FAILED_SAVE_BACKUP_LIMIT: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveEnvelope {
    pub schema_version: u32,
    pub engine_version: String,
    pub saved_at_unix_ms: u64,
    pub slot_id: String,
    pub mod_dependencies: Vec<SaveModDependency>,
    #[serde(default)]
    pub replay_log: Option<EngineReplayLog>,
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
pub struct SaveReadReport {
    pub save: SaveEnvelope,
    pub validation: SaveValidationReport,
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
pub struct SaveRecoveryReport {
    pub path: PathBuf,
    pub recovered_from: PathBuf,
    pub failed_primary_backup_path: Option<PathBuf>,
    pub save: SaveEnvelope,
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
    #[error("save has no recoverable backup")]
    MissingRecoverableBackup,
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
        let replay_log = Some(world.replay_log());

        Self {
            schema_version: SAVE_SCHEMA_VERSION,
            engine_version: ENGINE_VERSION.to_string(),
            saved_at_unix_ms,
            slot_id: slot_id.into(),
            mod_dependencies,
            replay_log,
            world,
        }
    }

    pub fn validate(
        &self,
        enabled_mods: &[SaveModDependency],
    ) -> Result<SaveValidationReport, SaveError> {
        self.validate_with_options(enabled_mods, false)
    }

    pub fn validate_against_registry(
        &self,
        enabled_mods: &[SaveModDependency],
    ) -> Result<SaveValidationReport, SaveError> {
        self.validate_with_options(enabled_mods, true)
    }

    fn validate_with_options(
        &self,
        enabled_mods: &[SaveModDependency],
        require_external_registry: bool,
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
                    && (require_external_registry
                        || !mod_dependency_is_available(dependency, &world_mod_dependencies))
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
        if let Some(replay_log) = &self.replay_log {
            if self.world.command_log_initial_random.is_none()
                && !replay_log.commands.is_empty()
                && self.world.command_log == replay_log.commands
            {
                self.world.command_log_initial_random = Some(replay_log.initial_random.clone());
            }
        }
        self.replay_log = Some(self.world.replay_log());
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
    write_save_atomic_with_backup_limit(path, save, timestamp_unix_ms, DEFAULT_SAVE_BACKUP_LIMIT)
}

pub fn write_save_atomic_with_backup_limit(
    path: impl AsRef<Path>,
    save: &SaveEnvelope,
    timestamp_unix_ms: u64,
    backup_limit: usize,
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
    prune_backup_files(path, BackupFileKind::Normal, backup_limit)?;

    Ok(SaveWriteReport {
        path: path.to_path_buf(),
        backup_path,
    })
}

pub fn read_save(
    path: impl AsRef<Path>,
    enabled_mods: &[SaveModDependency],
) -> Result<SaveEnvelope, SaveError> {
    Ok(read_save_report(path, enabled_mods)?.save)
}

pub fn read_save_report(
    path: impl AsRef<Path>,
    enabled_mods: &[SaveModDependency],
) -> Result<SaveReadReport, SaveError> {
    let encoded = fs::read(path)?;
    let save: SaveEnvelope = serde_json::from_slice(&encoded)?;
    let save = save.migrate_to_current()?;
    let validation = save.validate(enabled_mods)?;
    Ok(SaveReadReport { save, validation })
}

pub fn preflight_save_against_registry(
    path: impl AsRef<Path>,
    enabled_mods: &[SaveModDependency],
) -> Result<SaveReadReport, SaveError> {
    let encoded = fs::read(path)?;
    let save: SaveEnvelope = serde_json::from_slice(&encoded)?;
    let save = save.migrate_to_current()?;
    let validation = save.validate_against_registry(enabled_mods)?;
    Ok(SaveReadReport { save, validation })
}

pub fn recover_save_from_latest_backup(
    path: impl AsRef<Path>,
    enabled_mods: &[SaveModDependency],
    timestamp_unix_ms: u64,
) -> Result<SaveRecoveryReport, SaveError> {
    let path = path.as_ref();
    let recovered_from = latest_recoverable_backup_path_for(path, enabled_mods)?
        .ok_or(SaveError::MissingRecoverableBackup)?;
    let failed_primary_backup_path = if path.exists() {
        let backup_path = failed_primary_backup_path_for(path, timestamp_unix_ms);
        fs::copy(path, &backup_path)?;
        Some(backup_path)
    } else {
        None
    };

    fs::copy(&recovered_from, path)?;
    prune_backup_files(
        path,
        BackupFileKind::FailedPrimary,
        DEFAULT_FAILED_SAVE_BACKUP_LIMIT,
    )?;
    let save = read_save(path, enabled_mods)?;

    Ok(SaveRecoveryReport {
        path: path.to_path_buf(),
        recovered_from,
        failed_primary_backup_path,
        save,
    })
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

fn failed_primary_backup_path_for(path: &Path, timestamp_unix_ms: u64) -> PathBuf {
    let mut backup = PathBuf::from(path);
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_string());
    backup.set_extension(match extension {
        Some(extension) if !extension.is_empty() => {
            format!("{extension}.failed.{timestamp_unix_ms}.bak")
        }
        _ => format!("failed.{timestamp_unix_ms}.bak"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackupFileKind {
    Normal,
    FailedPrimary,
}

fn latest_recoverable_backup_path_for(
    path: &Path,
    enabled_mods: &[SaveModDependency],
) -> Result<Option<PathBuf>, SaveError> {
    for (_, _, candidate) in backup_file_candidates(path, BackupFileKind::Normal)?
        .into_iter()
        .rev()
    {
        if read_save(&candidate, enabled_mods).is_ok() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

fn prune_backup_files(
    path: &Path,
    kind: BackupFileKind,
    backup_limit: usize,
) -> Result<(), SaveError> {
    let candidates = backup_file_candidates(path, kind)?;
    let prune_count = candidates.len().saturating_sub(backup_limit);
    for (_, _, backup_path) in candidates.into_iter().take(prune_count) {
        fs::remove_file(backup_path)?;
    }

    Ok(())
}

fn backup_file_candidates(
    path: &Path,
    kind: BackupFileKind,
) -> Result<Vec<(SystemTime, String, PathBuf)>, SaveError> {
    let parent = path.parent().ok_or(SaveError::MissingParentDirectory)?;
    if !parent.exists() {
        return Ok(Vec::new());
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Ok(Vec::new());
    };
    let backup_prefix = format!("{file_name}.");
    let failed_prefix = format!("{file_name}.failed.");

    let mut candidates = Vec::<(SystemTime, String, PathBuf)>::new();
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let Some(entry_name) = entry_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !backup_file_name_matches(entry_name, &backup_prefix, &failed_prefix, kind) {
            continue;
        }
        let modified = entry
            .metadata()?
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH);
        candidates.push((modified, entry_name.to_string(), entry_path));
    }

    candidates.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    Ok(candidates)
}

fn backup_file_name_matches(
    entry_name: &str,
    backup_prefix: &str,
    failed_prefix: &str,
    kind: BackupFileKind,
) -> bool {
    match kind {
        BackupFileKind::Normal => {
            entry_name.starts_with(backup_prefix)
                && !entry_name.starts_with(failed_prefix)
                && entry_name.ends_with(".bak")
        }
        BackupFileKind::FailedPrimary => {
            entry_name.starts_with(failed_prefix) && entry_name.ends_with(".bak")
        }
    }
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
                dependencies: Vec::new(),
                conflicts: Vec::new(),
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
    fn save_envelope_embeds_replay_log_with_initial_random() {
        let mut world = WorldState::bootstrap_demo();
        world.random = crate::WorldRandom {
            seed: 42,
            cursor: 3,
        };
        world
            .apply_command(crate::EngineCommand::RollCharacterMood {
                character_id: "demo_heroine".to_string(),
                min_delta: -2,
                max_delta: 2,
            })
            .unwrap();

        let save = SaveEnvelope::new("slot-1", world, 123);
        let replay_log = save.replay_log.as_ref().unwrap();

        assert_eq!(
            replay_log.initial_random,
            crate::WorldRandom {
                seed: 42,
                cursor: 3,
            }
        );
        assert_eq!(replay_log.commands, save.world.command_log);
    }

    #[test]
    fn migration_backfills_missing_replay_log() {
        let mut save = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 123);
        save.replay_log = None;

        let migrated = save.migrate_to_current().unwrap();

        assert!(migrated.replay_log.is_some());
        assert_eq!(
            migrated.replay_log.as_ref().unwrap().commands,
            migrated.world.command_log
        );
    }

    #[test]
    fn save_preflight_registry_requires_external_enabled_mods() {
        let mut world = WorldState::bootstrap_demo();
        world
            .installed_content_packages
            .push(InstalledContentPackage {
                namespace: "sample".to_string(),
                package_id: "sample.event_pack".to_string(),
                version: "0.1.0".to_string(),
                dependencies: Vec::new(),
                conflicts: Vec::new(),
            });
        let save = SaveEnvelope::new("slot-1", world, 123);

        let compatibility_report = save.validate(&[]).unwrap();
        let registry_report = save.validate_against_registry(&[]).unwrap();
        let ready_report = save
            .validate_against_registry(&[SaveModDependency {
                namespace: "sample.event_pack".to_string(),
                version: "0.1.0".to_string(),
                required: true,
            }])
            .unwrap();

        assert!(compatibility_report.missing_required_mods.is_empty());
        assert_eq!(
            registry_report.missing_required_mods,
            vec![SaveModDependency {
                namespace: "sample.event_pack".to_string(),
                version: "0.1.0".to_string(),
                required: true,
            }]
        );
        assert!(ready_report.missing_required_mods.is_empty());
    }

    #[test]
    fn save_preflight_registry_reports_version_mismatch_as_missing_dependency() {
        let mut save = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 123);
        save.mod_dependencies.push(SaveModDependency {
            namespace: "example.required".to_string(),
            version: "1.0.0".to_string(),
            required: true,
        });

        let report = save
            .validate_against_registry(&[SaveModDependency {
                namespace: "example.required".to_string(),
                version: "2.0.0".to_string(),
                required: true,
            }])
            .unwrap();

        assert_eq!(report.missing_required_mods, save.mod_dependencies);
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
    fn save_file_prunes_old_backups_after_overwrite() {
        let dir = temp_save_dir("backup_prune");
        let path = dir.join("slot-1.json");

        for saved_at in 100..105 {
            let save = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), saved_at);
            write_save_atomic_with_backup_limit(&path, &save, saved_at, 2).unwrap();
        }

        let backup_names = backup_file_candidates(&path, BackupFileKind::Normal)
            .unwrap()
            .into_iter()
            .map(|(_, name, _)| name)
            .collect::<Vec<_>>();

        assert_eq!(backup_names.len(), 2);
        assert!(backup_names.iter().any(|name| name.contains(".103.bak")));
        assert!(backup_names.iter().any(|name| name.contains(".104.bak")));
        assert!(!backup_names.iter().any(|name| name.contains(".102.bak")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn recover_save_restores_latest_backup_and_preserves_failed_primary() {
        let dir = temp_save_dir("recover_latest");
        let path = dir.join("slot-1.json");
        let first = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 100);
        let mut second_world = WorldState::bootstrap_demo();
        second_world.clock.minute = 30;
        let second = SaveEnvelope::new("slot-1", second_world, 200);

        write_save_atomic(&path, &first, 100).unwrap();
        write_save_atomic(&path, &second, 200).unwrap();
        fs::write(&path, b"{broken").unwrap();

        let report = recover_save_from_latest_backup(&path, &[], 300).unwrap();
        let recovered = read_save(&path, &[]).unwrap();

        assert_eq!(report.save.saved_at_unix_ms, 100);
        assert_eq!(recovered.saved_at_unix_ms, 100);
        assert!(report
            .recovered_from
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains(".200.bak"));
        let failed_primary_backup_path = report.failed_primary_backup_path.unwrap();
        assert!(failed_primary_backup_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains(".failed.300.bak"));
        assert_eq!(
            fs::read_to_string(failed_primary_backup_path).unwrap(),
            "{broken"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn recover_save_skips_invalid_and_failed_primary_backups() {
        let dir = temp_save_dir("recover_skips_bad_backups");
        let path = dir.join("slot-1.json");
        let first = SaveEnvelope::new("slot-1", WorldState::bootstrap_demo(), 100);
        let mut second_world = WorldState::bootstrap_demo();
        second_world.clock.minute = 30;
        let second = SaveEnvelope::new("slot-1", second_world, 200);

        write_save_atomic(&path, &first, 100).unwrap();
        write_save_atomic(&path, &second, 200).unwrap();
        fs::write(backup_path_for(&path, 300), b"{broken backup").unwrap();
        fs::write(
            failed_primary_backup_path_for(&path, 400),
            b"{failed primary",
        )
        .unwrap();
        fs::write(&path, b"{broken primary").unwrap();

        let report = recover_save_from_latest_backup(&path, &[], 500).unwrap();

        assert_eq!(report.save.saved_at_unix_ms, 100);
        assert!(report
            .recovered_from
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains(".200.bak"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn recover_save_reports_missing_backup() {
        let dir = temp_save_dir("recover_missing");
        let path = dir.join("slot-1.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, b"{broken").unwrap();

        let result = recover_save_from_latest_backup(&path, &[], 300);

        assert_eq!(result, Err(SaveError::MissingRecoverableBackup));

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
