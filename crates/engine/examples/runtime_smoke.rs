use eratw_next_engine::{load_content_package, GameCommand, GameSession};
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&error).unwrap_or_else(|_| error.to_string())
        );
        std::process::exit(2);
    }
}

fn run() -> Result<(), eratw_next_engine::EngineError> {
    let mut args = std::env::args().skip(1);
    let first = args.next().ok_or_else(|| {
        eratw_next_engine::EngineError::unavailable(
            "SMOKE_PACKAGE_REQUIRED",
            "Usage: runtime_smoke [--create-demo] <absolute-package-path> [new-save-json-path]",
        )
    })?;
    let package_path = if first == "--create-demo" {
        let path = PathBuf::from(args.next().ok_or_else(|| {
            eratw_next_engine::EngineError::unavailable(
                "SMOKE_PACKAGE_REQUIRED",
                "--create-demo requires an absolute output path.",
            )
        })?);
        create_demo_package(&path)?;
        path
    } else {
        PathBuf::from(first)
    };
    let save_path = args.next().map(PathBuf::from);
    let package = load_content_package(package_path)?;

    let mut output = json!({
        "package": package.index.package,
        "playable": package.index.playable,
        "counts": package.index.counts,
        "warnings": package.index.warnings,
        "game": null
    });

    if package.index.playable {
        let mut session = GameSession::from_package(&package)?;
        session.apply(GameCommand::Wait { minutes: 120 })?;
        if let Some(target) = session
            .context
            .locations
            .get(&session.state.current_location_id)
            .and_then(|connections| connections.first())
            .cloned()
        {
            session.apply(GameCommand::Move {
                location_id: target,
                minutes: 10,
            })?;
        }
        if let Some(save_path) = save_path {
            let report = eratw_next_engine::save_game_file(&save_path, &session)?;
            let loaded = eratw_next_engine::load_save_file(&save_path, &package)?;
            output["game"] = json!({
                "turn": loaded.state.turn,
                "clock": loaded.state.clock,
                "locationId": loaded.state.current_location_id,
                "save": report
            });
        } else {
            output["game"] = json!({
                "turn": session.state.turn,
                "clock": session.state.clock,
                "locationId": session.state.current_location_id
            });
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("smoke output is serializable")
    );
    Ok(())
}

fn create_demo_package(path: &Path) -> Result<(), eratw_next_engine::EngineError> {
    if !path.is_absolute() || path.exists() {
        return Err(eratw_next_engine::EngineError::new(
            "SMOKE_DEMO_PATH_INVALID",
            "Demo package output must be a new absolute path.",
            json!({ "path": path.to_string_lossy() }),
        ));
    }
    for directory in [
        "dictionaries",
        "characters",
        "locations",
        "resources",
        "dialogue",
    ] {
        fs::create_dir_all(path.join(directory)).map_err(smoke_io_error)?;
    }
    fs::write(
        path.join("manifest.json"),
        r#"{
  "schemaVersion":"content-package/v1-draft",
  "packageId":"demo.playable",
  "displayName":"Playable Demo",
  "version":"1.0.0",
  "engineVersion":">=0.4.0",
  "source":{"kind":"legacy-eratw","sourceRootId":"eratw-content","sourceRootHint":"self-owned-demo","generatedFromAudit":"content-audit-summary/v1"},
  "dependencies":[],
  "conflicts":[],
  "capabilities":["playable.core"],
  "review":{"status":"accepted","notes":["Self-owned runtime smoke package."],"blockingIssues":[]}
}"#,
    )
    .map_err(smoke_io_error)?;
    fs::write(
        path.join("characters/characters.jsonl"),
        r#"{"id":"core.character.001","legacy":{"numericId":1,"rawKeys":["1"]},"displayName":{"primary":"Demo Character","ja":null,"zhHans":"示例角色","aliases":[]},"profile":{"species":null,"title":null,"description":null},"dictionaryRefs":{"talents":[],"baseStats":[],"abilities":[]},"resourceRefs":[],"dialogueSourceRefs":[],"sourceTrace":{"sourceRootId":"eratw-content","relativePath":"self-owned-demo","legacyId":"1","lineRange":null,"contentHash":null,"conversion":{"tool":"runtime-smoke","version":"1","confidence":"high","requiresReview":false}},"review":{"status":"accepted","notes":[],"blockingIssues":[]}}"#,
    )
    .map_err(smoke_io_error)?;
    let mut locations =
        File::create(path.join("locations/locations.jsonl")).map_err(smoke_io_error)?;
    writeln!(locations, r#"{{"id":"core.location.home","displayName":{{"primary":"Home","zhHans":"居所","ja":null}},"kind":"home","tags":["safe"],"connections":["core.location.square"],"sourceTrace":{{}},"review":{{"status":"accepted"}}}}"#).map_err(smoke_io_error)?;
    writeln!(locations, r#"{{"id":"core.location.square","displayName":{{"primary":"Square","zhHans":"广场","ja":null}},"kind":"public","tags":[],"connections":["core.location.home"],"sourceTrace":{{}},"review":{{"status":"accepted"}}}}"#).map_err(smoke_io_error)?;
    for relative in [
        "dictionaries/legacy-csv-dictionaries.jsonl",
        "resources/resources.jsonl",
        "dialogue/dialogue-sources.jsonl",
        "dialogue/dialogue-scenes.jsonl",
    ] {
        fs::write(path.join(relative), "").map_err(smoke_io_error)?;
    }
    Ok(())
}

fn smoke_io_error(error: std::io::Error) -> eratw_next_engine::EngineError {
    eratw_next_engine::EngineError::new(
        "SMOKE_DEMO_WRITE_FAILED",
        "Demo package could not be created.",
        json!({ "error": error.to_string() }),
    )
}
