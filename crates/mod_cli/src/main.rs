use eratw_mod_runtime::{
    package_mod_project_for_engine, validate_mod_project_for_engine, ModDiscoveryError,
};
use std::{env, path::PathBuf, process::ExitCode};

enum Command {
    Validate {
        root: PathBuf,
        engine_version: Option<String>,
    },
    Pack {
        source_root: PathBuf,
        output_root: PathBuf,
        engine_version: Option<String>,
    },
    Help,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(message) => {
            if !message.is_empty() {
                println!("{message}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    match parse_command(args)? {
        Command::Validate {
            root,
            engine_version,
        } => validate_mod_project_for_engine(&root, engine_version.as_deref())
            .map(|report| {
                format!(
                    "valid mod {} {} at {}",
                    report.manifest.namespace,
                    report.manifest.version,
                    report.root_path.display()
                )
            })
            .map_err(format_mod_error),
        Command::Pack {
            source_root,
            output_root,
            engine_version,
        } => package_mod_project_for_engine(&source_root, &output_root, engine_version.as_deref())
            .map(|report| {
                format!(
                    "packed mod {} {} to {}",
                    report.manifest.namespace,
                    report.manifest.version,
                    report.package_root.display()
                )
            })
            .map_err(format_mod_error),
        Command::Help => Ok(usage()),
    }
}

fn parse_command(args: Vec<String>) -> Result<Command, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::Help);
    };

    match command {
        "validate" => {
            if includes_help_option(&args[1..]) {
                return Ok(Command::Help);
            }
            let (positionals, engine_version) = parse_options(&args[1..])?;
            if positionals.len() != 1 {
                return Err(format!(
                    "{}\n\nvalidate expects exactly one root path",
                    usage()
                ));
            }
            Ok(Command::Validate {
                root: PathBuf::from(&positionals[0]),
                engine_version,
            })
        }
        "pack" => {
            if includes_help_option(&args[1..]) {
                return Ok(Command::Help);
            }
            let (positionals, engine_version) = parse_options(&args[1..])?;
            if positionals.len() != 2 {
                return Err(format!(
                    "{}\n\npack expects source root and output root paths",
                    usage()
                ));
            }
            Ok(Command::Pack {
                source_root: PathBuf::from(&positionals[0]),
                output_root: PathBuf::from(&positionals[1]),
                engine_version,
            })
        }
        "-h" | "--help" | "help" => Ok(Command::Help),
        unknown => Err(format!("{}\n\nunknown command: {unknown}", usage())),
    }
}

fn parse_options(args: &[String]) -> Result<(Vec<String>, Option<String>), String> {
    let mut positionals = Vec::new();
    let mut engine_version = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--engine-version" => {
                index += 1;
                let Some(version) = args.get(index) else {
                    return Err("--engine-version requires a value".to_string());
                };
                engine_version = Some(version.clone());
            }
            option if option.starts_with('-') => {
                return Err(format!("unknown option: {option}"));
            }
            positional => positionals.push(positional.to_string()),
        }
        index += 1;
    }

    Ok((positionals, engine_version))
}

fn includes_help_option(args: &[String]) -> bool {
    args.iter()
        .any(|argument| argument == "-h" || argument == "--help")
}

fn format_mod_error(error: ModDiscoveryError) -> String {
    format!("mod command failed: {error}")
}

fn usage() -> String {
    [
        "ERAtw-NEXT Mod CLI",
        "",
        "Usage:",
        "  eratw-mod validate <mod-root> [--engine-version <version>]",
        "  eratw-mod pack <mod-root> <output-root> [--engine-version <version>]",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn validate_command_reports_valid_project() {
        let root = temp_dir("validate_command");
        write_manifest(&root, "example.cli");

        let output = run(vec![
            "validate".to_string(),
            root.to_string_lossy().to_string(),
            "--engine-version".to_string(),
            "0.1.0-m0".to_string(),
        ])
        .unwrap();

        assert!(output.contains("valid mod example.cli 0.1.0"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pack_command_writes_package_directory() {
        let source_root = temp_dir("pack_command_source");
        let output_root = temp_dir("pack_command_output");
        write_manifest(&source_root, "example.cli");
        fs::write(source_root.join("readme.txt"), "packed").unwrap();

        let output = run(vec![
            "pack".to_string(),
            source_root.to_string_lossy().to_string(),
            output_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        assert!(output.contains("packed mod example.cli 0.1.0"));
        assert!(output_root
            .join("example.cli-0.1.0/content/manifest.json")
            .exists());
        assert_eq!(
            fs::read_to_string(output_root.join("example.cli-0.1.0/content/readme.txt")).unwrap(),
            "packed"
        );

        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn unknown_command_returns_usage_error() {
        let error = run(vec!["wat".to_string()]).unwrap_err();

        assert!(error.contains("unknown command: wat"));
        assert!(error.contains("eratw-mod validate"));
    }

    #[test]
    fn command_help_returns_usage() {
        let output = run(vec!["pack".to_string(), "--help".to_string()]).unwrap();

        assert!(output.contains("ERAtw-NEXT Mod CLI"));
        assert!(output.contains("eratw-mod pack"));
    }

    fn write_manifest(root: &PathBuf, namespace: &str) {
        fs::create_dir_all(root).unwrap();
        fs::write(
            root.join("manifest.json"),
            format!(
                r#"{{
  "namespace": "{namespace}",
  "name": "{namespace}",
  "version": "0.1.0",
  "engine_version": "0.1.0-m0",
  "load_order": 0,
  "dependencies": [],
  "conflicts": [],
  "capabilities": ["content"]
}}"#
            ),
        )
        .unwrap();
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("eratw_next_mod_cli_{label}_{nonce}"))
    }
}
