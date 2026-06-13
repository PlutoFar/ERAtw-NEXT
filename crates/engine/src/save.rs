use crate::{
    replay_commands, EngineError, GameCommand, GameSession, GameState, LoadedContentPackage,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

pub const SAVE_ENVELOPE_SCHEMA_VERSION: &str = "save-envelope/v1";
pub const SAVE_REPORT_SCHEMA_VERSION: &str = "save-report/v1";
const SAVE_FORMAT_VERSION: u32 = 1;
const MAX_SAVE_BYTES: u64 = 16 * 1024 * 1024;
#[cfg(test)]
const SAVE_REPORT_SCHEMA: &str = include_str!("../../../schemas/save-report.schema.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SaveDependency {
    pub package_id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SaveEnvelope {
    pub schema_version: String,
    pub save_version: u32,
    pub engine_version: String,
    pub dependencies: Vec<SaveDependency>,
    pub initial_state: GameState,
    pub state: GameState,
    pub command_log: Vec<GameCommand>,
    pub state_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SaveReport {
    pub schema_version: String,
    pub path: String,
    pub package_id: String,
    pub turn: u64,
    pub bytes: u64,
    pub state_hash: String,
}

pub fn save_game_file(
    path: impl AsRef<Path>,
    session: &GameSession,
) -> Result<SaveReport, EngineError> {
    let path = validate_new_save_path(path.as_ref())?;
    validate_session_for_save(session)?;
    let envelope = build_envelope(session)?;
    let encoded = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        save_error(
            "SAVE_SERIALIZE_FAILED",
            "Game save could not be serialized.",
            json!({ "error": error.to_string() }),
        )
    })?;
    if encoded.len() as u64 > MAX_SAVE_BYTES {
        return Err(save_error(
            "SAVE_TOO_LARGE",
            "Game save exceeds the size limit.",
            json!({ "bytes": encoded.len(), "limit": MAX_SAVE_BYTES }),
        ));
    }
    write_atomic_new(&path, &encoded)?;
    Ok(SaveReport {
        schema_version: SAVE_REPORT_SCHEMA_VERSION.to_string(),
        path: normalize_path(&path),
        package_id: session.state.package.package_id.clone(),
        turn: session.state.turn,
        bytes: encoded.len() as u64,
        state_hash: envelope.state_hash,
    })
}

pub fn load_save_file(
    path: impl AsRef<Path>,
    package: &LoadedContentPackage,
) -> Result<GameSession, EngineError> {
    let path = validate_existing_save_path(path.as_ref())?;
    let metadata = fs::metadata(&path).map_err(|error| {
        save_error(
            "SAVE_UNREADABLE",
            "Game save metadata could not be read.",
            json!({ "path": normalize_path(&path), "error": error.to_string() }),
        )
    })?;
    if metadata.len() > MAX_SAVE_BYTES {
        return Err(save_error(
            "SAVE_TOO_LARGE",
            "Game save exceeds the size limit.",
            json!({ "bytes": metadata.len(), "limit": MAX_SAVE_BYTES }),
        ));
    }
    let envelope: SaveEnvelope =
        serde_json::from_reader(BufReader::new(File::open(&path).map_err(|error| {
            save_error(
                "SAVE_UNREADABLE",
                "Game save file could not be opened.",
                json!({ "path": normalize_path(&path), "error": error.to_string() }),
            )
        })?))
        .map_err(|error| {
            save_error(
                "SAVE_JSON_INVALID",
                "Game save JSON is invalid.",
                json!({ "path": normalize_path(&path), "error": error.to_string() }),
            )
        })?;
    validate_envelope(&envelope, package)?;
    let context = crate::GameContext::from_package(package);
    validate_saved_state(&context, &envelope.initial_state, "initialState")?;
    validate_saved_state(&context, &envelope.state, "state")?;
    let replayed = replay_saved_commands(&context, &envelope.initial_state, &envelope.command_log)?;
    if replayed != envelope.state {
        return Err(save_error(
            "SAVE_REPLAY_MISMATCH",
            "Game save state does not match deterministic command replay.",
            json!({
                "savedTurn": envelope.state.turn,
                "replayedTurn": replayed.turn
            }),
        ));
    }
    Ok(GameSession {
        context,
        initial_state: envelope.initial_state,
        state: envelope.state,
        command_log: envelope.command_log,
    })
}

fn validate_session_for_save(session: &GameSession) -> Result<(), EngineError> {
    validate_saved_state(&session.context, &session.initial_state, "initialState")?;
    validate_saved_state(&session.context, &session.state, "state")?;
    let replayed = replay_saved_commands(
        &session.context,
        &session.initial_state,
        &session.command_log,
    )?;
    if replayed != session.state {
        return Err(save_error(
            "SAVE_REPLAY_MISMATCH",
            "Game session state does not match deterministic command replay.",
            json!({
                "savedTurn": session.state.turn,
                "replayedTurn": replayed.turn
            }),
        ));
    }
    Ok(())
}

fn validate_saved_state(
    context: &crate::GameContext,
    state: &GameState,
    field: &str,
) -> Result<(), EngineError> {
    crate::game::validate_game_state(context, state).map_err(|error| {
        save_error(
            "SAVE_STATE_INVALID",
            "Game save contains an invalid game state.",
            json!({
                "field": field,
                "sourceCode": error.code,
                "sourceMessage": error.message,
                "sourceDetails": error.details
            }),
        )
    })
}

fn replay_saved_commands(
    context: &crate::GameContext,
    initial_state: &GameState,
    command_log: &[GameCommand],
) -> Result<GameState, EngineError> {
    replay_commands(context, initial_state, command_log).map_err(|error| {
        save_error(
            "SAVE_REPLAY_MISMATCH",
            "Game save command log could not be replayed.",
            json!({
                "sourceCode": error.code,
                "sourceMessage": error.message,
                "sourceDetails": error.details
            }),
        )
    })
}

fn build_envelope(session: &GameSession) -> Result<SaveEnvelope, EngineError> {
    let state_hash = hash_state(&session.state)?;
    Ok(SaveEnvelope {
        schema_version: SAVE_ENVELOPE_SCHEMA_VERSION.to_string(),
        save_version: SAVE_FORMAT_VERSION,
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        dependencies: vec![SaveDependency {
            package_id: session.state.package.package_id.clone(),
            version: session.state.package.version.clone(),
        }],
        initial_state: session.initial_state.clone(),
        state: session.state.clone(),
        command_log: session.command_log.clone(),
        state_hash,
    })
}

fn validate_envelope(
    envelope: &SaveEnvelope,
    package: &LoadedContentPackage,
) -> Result<(), EngineError> {
    if envelope.schema_version != SAVE_ENVELOPE_SCHEMA_VERSION
        || envelope.save_version != SAVE_FORMAT_VERSION
    {
        return Err(save_error(
            "SAVE_VERSION_UNSUPPORTED",
            "Game save version is not supported.",
            json!({
                "schemaVersion": envelope.schema_version,
                "saveVersion": envelope.save_version,
                "expectedSchemaVersion": SAVE_ENVELOPE_SCHEMA_VERSION,
                "expectedSaveVersion": SAVE_FORMAT_VERSION,
                "migrationAvailable": false
            }),
        ));
    }
    let save_engine_version = Version::parse(&envelope.engine_version).map_err(|error| {
        save_error(
            "SAVE_ENGINE_VERSION_INVALID",
            "Game save engine version is invalid.",
            json!({ "version": envelope.engine_version, "error": error.to_string() }),
        )
    })?;
    let current_engine_version =
        Version::parse(env!("CARGO_PKG_VERSION")).expect("engine package version is valid semver");
    if save_engine_version > current_engine_version {
        return Err(save_error(
            "SAVE_ENGINE_VERSION_NEWER",
            "Game save was created by a newer engine version.",
            json!({
                "saveEngineVersion": save_engine_version.to_string(),
                "currentEngineVersion": current_engine_version.to_string()
            }),
        ));
    }
    let expected = SaveDependency {
        package_id: package.manifest.package_id.clone(),
        version: package.manifest.version.clone(),
    };
    if !envelope.dependencies.contains(&expected) {
        return Err(save_error(
            "SAVE_DEPENDENCY_MISSING",
            "Required content package dependency is not loaded.",
            json!({
                "required": envelope.dependencies,
                "loaded": expected
            }),
        ));
    }
    if envelope.initial_state.package != package.index.package
        || envelope.state.package != package.index.package
    {
        return Err(save_error(
            "SAVE_PACKAGE_MISMATCH",
            "Game save state belongs to a different content package.",
            json!({
                "loaded": package.index.package,
                "initialState": envelope.initial_state.package,
                "state": envelope.state.package
            }),
        ));
    }
    let actual_hash = hash_state(&envelope.state)?;
    if actual_hash != envelope.state_hash {
        return Err(save_error(
            "SAVE_HASH_MISMATCH",
            "Game save state hash does not match its payload.",
            json!({
                "expected": envelope.state_hash,
                "actual": actual_hash
            }),
        ));
    }
    Ok(())
}

fn hash_state(state: &GameState) -> Result<String, EngineError> {
    let bytes = serde_json::to_vec(state).map_err(|error| {
        save_error(
            "SAVE_SERIALIZE_FAILED",
            "Game state could not be serialized for hashing.",
            json!({ "error": error.to_string() }),
        )
    })?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn validate_new_save_path(path: &Path) -> Result<PathBuf, EngineError> {
    let absolute = validate_save_path_shape(path)?;
    if absolute.exists() {
        return Err(save_error(
            "SAVE_ALREADY_EXISTS",
            "Game save path already exists.",
            json!({ "path": normalize_path(&absolute) }),
        ));
    }
    let parent = absolute.parent().ok_or_else(|| {
        save_error(
            "SAVE_PARENT_INVALID",
            "Game save path has no parent directory.",
            json!({ "path": normalize_path(&absolute) }),
        )
    })?;
    let parent = parent.canonicalize().map_err(|error| {
        save_error(
            "SAVE_PARENT_UNAVAILABLE",
            "Game save parent directory cannot be resolved.",
            json!({ "path": normalize_path(parent), "error": error.to_string() }),
        )
    })?;
    Ok(parent.join(
        absolute
            .file_name()
            .expect("validated save path has a file name"),
    ))
}

fn validate_existing_save_path(path: &Path) -> Result<PathBuf, EngineError> {
    let absolute = validate_save_path_shape(path)?;
    let metadata = fs::symlink_metadata(&absolute).map_err(|error| {
        save_error(
            "SAVE_UNREADABLE",
            "Game save file cannot be resolved.",
            json!({ "path": normalize_path(&absolute), "error": error.to_string() }),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(save_error(
            "SAVE_PATH_UNSAFE",
            "Game save path must be a regular file.",
            json!({ "path": normalize_path(&absolute) }),
        ));
    }
    absolute.canonicalize().map_err(|error| {
        save_error(
            "SAVE_UNREADABLE",
            "Game save file cannot be resolved.",
            json!({ "path": normalize_path(&absolute), "error": error.to_string() }),
        )
    })
}

fn validate_save_path_shape(path: &Path) -> Result<PathBuf, EngineError> {
    if !path.is_absolute() {
        return Err(save_error(
            "SAVE_PATH_NOT_ABSOLUTE",
            "Game save path must be absolute.",
            json!({ "path": normalize_path(path) }),
        ));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(save_error(
            "SAVE_PATH_TRAVERSAL",
            "Game save path must not contain parent traversal.",
            json!({ "path": normalize_path(path) }),
        ));
    }
    if looks_like_unc_path(path) {
        return Err(save_error(
            "SAVE_PATH_NETWORK",
            "Network save paths are not allowed.",
            json!({ "path": normalize_path(path) }),
        ));
    }
    if !path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        return Err(save_error(
            "SAVE_EXTENSION_INVALID",
            "Game save path must use the .json extension.",
            json!({ "path": normalize_path(path) }),
        ));
    }
    Ok(path.to_path_buf())
}

fn write_atomic_new(path: &Path, encoded: &[u8]) -> Result<(), EngineError> {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .expect("validated save path has a UTF-8 file name");
    let temp_path = path.with_file_name(format!(".{file_name}.{}.tmp", std::process::id()));
    if temp_path.exists() {
        return Err(save_error(
            "SAVE_TEMP_EXISTS",
            "Temporary save path already exists.",
            json!({ "path": normalize_path(&temp_path) }),
        ));
    }

    let result = (|| {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(encoded)?;
        writer.flush()?;
        writer.get_ref().sync_all().map_err(std::io::Error::other)?;
        drop(writer);
        fs::rename(&temp_path, path)?;
        Ok::<(), std::io::Error>(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temp_path);
        return Err(save_error(
            "SAVE_WRITE_FAILED",
            "Game save could not be written atomically.",
            json!({ "path": normalize_path(path), "error": error.to_string() }),
        ));
    }
    Ok(())
}

fn looks_like_unc_path(path: &Path) -> bool {
    let value = path.as_os_str().to_string_lossy();
    value.starts_with(r"\\") || value.starts_with("//")
}

fn save_error(code: &str, message: &str, details: serde_json::Value) -> EngineError {
    EngineError::new(code, message, details)
}

fn normalize_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    normalized
        .strip_prefix("//?/")
        .unwrap_or(&normalized)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::tests::create_playable_package;
    use crate::{load_content_package, GameCommand};

    #[test]
    fn save_round_trip_preserves_state_and_log() {
        let root = create_playable_package("save-round-trip");
        let package = load_content_package(&root).unwrap();
        let mut session = GameSession::from_package(&package).unwrap();
        session.apply(GameCommand::Wait { minutes: 120 }).unwrap();
        session
            .apply(GameCommand::Move {
                location_id: "core.location.square".to_string(),
                minutes: 10,
            })
            .unwrap();
        let save_path = root.parent().unwrap().join(format!(
            "eratw_next_save_{}_roundtrip.json",
            std::process::id()
        ));
        if save_path.exists() {
            fs::remove_file(&save_path).unwrap();
        }
        let report = save_game_file(&save_path, &session).unwrap();
        let loaded = load_save_file(&save_path, &package).unwrap();
        assert_eq!(loaded.state, session.state);
        assert_eq!(loaded.command_log, session.command_log);
        assert_eq!(report.turn, 2);
        let schema: serde_json::Value = serde_json::from_str(SAVE_REPORT_SCHEMA).unwrap();
        let compiled = jsonschema::JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .compile(&schema)
            .unwrap();
        assert!(compiled
            .validate(&serde_json::to_value(&report).unwrap())
            .is_ok());
        fs::remove_file(save_path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_invalid_json_and_dependency_mismatch() {
        let root = create_playable_package("save-errors");
        let package = load_content_package(&root).unwrap();
        let bad_path = root
            .parent()
            .unwrap()
            .join(format!("eratw_next_save_{}_bad.json", std::process::id()));
        fs::write(&bad_path, "{").unwrap();
        let error = load_save_file(&bad_path, &package).unwrap_err();
        assert_eq!(error.code, "SAVE_JSON_INVALID");
        fs::remove_file(&bad_path).unwrap();

        let session = GameSession::from_package(&package).unwrap();
        let dependency_path = root.parent().unwrap().join(format!(
            "eratw_next_save_{}_dependency.json",
            std::process::id()
        ));
        let mut envelope = build_envelope(&session).unwrap();
        envelope.dependencies[0].package_id = "other.package".to_string();
        fs::write(
            &dependency_path,
            serde_json::to_vec_pretty(&envelope).unwrap(),
        )
        .unwrap();
        let error = load_save_file(&dependency_path, &package).unwrap_err();
        assert_eq!(error.code, "SAVE_DEPENDENCY_MISSING");
        fs::remove_file(dependency_path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_future_version_hash_mismatch_and_replay_mismatch() {
        let root = create_playable_package("save-integrity");
        let package = load_content_package(&root).unwrap();
        let mut session = GameSession::from_package(&package).unwrap();
        session.apply(GameCommand::Wait { minutes: 30 }).unwrap();
        let base = build_envelope(&session).unwrap();

        let save_version_path = temp_save_path("save-version");
        let mut unsupported = base.clone();
        unsupported.save_version = SAVE_FORMAT_VERSION + 1;
        write_envelope(&save_version_path, &unsupported);
        let version_error = load_save_file(&save_version_path, &package).unwrap_err();
        assert_eq!(version_error.code, "SAVE_VERSION_UNSUPPORTED");
        assert_eq!(version_error.details["migrationAvailable"], false);
        fs::remove_file(save_version_path).unwrap();

        let version_path = temp_save_path("future");
        let mut future = base.clone();
        future.engine_version = "99.0.0".to_string();
        write_envelope(&version_path, &future);
        assert_eq!(
            load_save_file(&version_path, &package).unwrap_err().code,
            "SAVE_ENGINE_VERSION_NEWER"
        );
        fs::remove_file(version_path).unwrap();

        let hash_path = temp_save_path("hash");
        let mut bad_hash = base.clone();
        bad_hash.state_hash =
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
        write_envelope(&hash_path, &bad_hash);
        assert_eq!(
            load_save_file(&hash_path, &package).unwrap_err().code,
            "SAVE_HASH_MISMATCH"
        );
        fs::remove_file(hash_path).unwrap();

        let replay_path = temp_save_path("replay");
        let mut bad_replay = base;
        bad_replay.state.player.money += 1;
        bad_replay.state_hash = hash_state(&bad_replay.state).unwrap();
        write_envelope(&replay_path, &bad_replay);
        assert_eq!(
            load_save_file(&replay_path, &package).unwrap_err().code,
            "SAVE_REPLAY_MISMATCH"
        );
        fs::remove_file(replay_path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_state_when_command_log_is_empty() {
        let root = create_playable_package("save-invalid-state");
        let package = load_content_package(&root).unwrap();
        let session = GameSession::from_package(&package).unwrap();
        let mut envelope = build_envelope(&session).unwrap();
        envelope.initial_state.player.max_energy = 0;
        envelope.initial_state.player.energy = 0;
        envelope.state = envelope.initial_state.clone();
        envelope.state_hash = hash_state(&envelope.state).unwrap();
        let path = temp_save_path("invalid-state");
        write_envelope(&path, &envelope);

        let error = load_save_file(&path, &package).unwrap_err();
        assert_eq!(error.code, "SAVE_STATE_INVALID");
        fs::remove_file(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_unknown_fields_inside_package_identity() {
        let root = create_playable_package("save-unknown-field");
        let package = load_content_package(&root).unwrap();
        let session = GameSession::from_package(&package).unwrap();
        let envelope = build_envelope(&session).unwrap();
        let mut value = serde_json::to_value(envelope).unwrap();
        value["state"]["package"]["unexpected"] = json!(true);
        let path = temp_save_path("unknown-field");
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

        let error = load_save_file(&path, &package).unwrap_err();
        assert_eq!(error.code, "SAVE_JSON_INVALID");
        fs::remove_file(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn save_rejects_invalid_or_unreplayable_session() {
        let root = create_playable_package("save-invalid-session");
        let package = load_content_package(&root).unwrap();
        let mut invalid = GameSession::from_package(&package).unwrap();
        invalid.state.player.max_energy = 0;
        invalid.state.player.energy = 0;
        let invalid_path = temp_save_path("write-invalid-state");
        let state_error = save_game_file(&invalid_path, &invalid).unwrap_err();
        assert_eq!(state_error.code, "SAVE_STATE_INVALID");
        assert!(!invalid_path.exists());

        let mut unreplayable = GameSession::from_package(&package).unwrap();
        unreplayable.state.player.money += 1;
        let replay_path = temp_save_path("write-replay");
        let replay_error = save_game_file(&replay_path, &unreplayable).unwrap_err();
        assert_eq!(replay_error.code, "SAVE_REPLAY_MISMATCH");
        assert!(!replay_path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    fn temp_save_path(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "eratw_next_save_{}_{}.json",
            std::process::id(),
            label
        ));
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        path
    }

    fn write_envelope(path: &Path, envelope: &SaveEnvelope) {
        fs::write(path, serde_json::to_vec_pretty(envelope).unwrap()).unwrap();
    }
}
