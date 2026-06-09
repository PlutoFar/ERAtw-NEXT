use eratw_mod_runtime::{
    check_mod_package_for_engine_with_policy, install_mod_package_for_engine_with_policy,
    package_mod_project_for_engine_with_policy, parse_mod_capability,
    preflight_mod_package_install_for_engine_with_policy, scaffold_mod_template,
    validate_mod_project_for_engine_with_policy, ModCapability, ModDiscoveryError,
    ModInstallPreflightIssueSeverity, ModInstallPreflightReport, ModSecurityPolicy,
    ModTemplateOptions,
};
use std::{env, path::PathBuf, process::ExitCode};

enum Command {
    New {
        root: PathBuf,
        namespace: String,
        name: Option<String>,
        version: String,
        engine_version: String,
    },
    Validate {
        root: PathBuf,
        engine_version: Option<String>,
        allowed_capabilities: Vec<ModCapability>,
    },
    Pack {
        source_root: PathBuf,
        output_root: PathBuf,
        engine_version: Option<String>,
        allowed_capabilities: Vec<ModCapability>,
    },
    CheckPackage {
        package_root: PathBuf,
        engine_version: Option<String>,
        allowed_capabilities: Vec<ModCapability>,
    },
    InstallPackage {
        package_root: PathBuf,
        install_root: PathBuf,
        engine_version: Option<String>,
        allowed_capabilities: Vec<ModCapability>,
    },
    PreflightInstallPackage {
        package_root: PathBuf,
        install_root: PathBuf,
        engine_version: Option<String>,
        allowed_capabilities: Vec<ModCapability>,
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
        Command::New {
            root,
            namespace,
            name,
            version,
            engine_version,
        } => scaffold_mod_template(
            &root,
            ModTemplateOptions {
                namespace,
                name: name.unwrap_or_else(|| {
                    root.file_name()
                        .and_then(|file_name| file_name.to_str())
                        .unwrap_or("ERAtw-NEXT Mod")
                        .to_string()
                }),
                version,
                engine_version,
            },
        )
        .map(|report| {
            format!(
                "created mod template {} {} at {}",
                report.manifest.namespace,
                report.manifest.version,
                report.root_path.display()
            )
        })
        .map_err(format_mod_error),
        Command::Validate {
            root,
            engine_version,
            allowed_capabilities,
        } => validate_mod_project_for_engine_with_policy(
            &root,
            engine_version.as_deref(),
            &security_policy(allowed_capabilities),
        )
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
            allowed_capabilities,
        } => package_mod_project_for_engine_with_policy(
            &source_root,
            &output_root,
            engine_version.as_deref(),
            &security_policy(allowed_capabilities),
        )
        .map(|report| {
            format!(
                "packed mod {} {} to {}",
                report.manifest.namespace,
                report.manifest.version,
                report.package_root.display()
            )
        })
        .map_err(format_mod_error),
        Command::CheckPackage {
            package_root,
            engine_version,
            allowed_capabilities,
        } => check_mod_package_for_engine_with_policy(
            &package_root,
            engine_version.as_deref(),
            &security_policy(allowed_capabilities),
        )
        .map(|report| {
            format!(
                "checked mod package {} {} at {} ({} resource errors, {} resource warnings)",
                report.manifest.namespace,
                report.manifest.version,
                report.package_root.display(),
                report.resource_report.error_count,
                report.resource_report.warning_count
            )
        })
        .map_err(format_mod_error),
        Command::InstallPackage {
            package_root,
            install_root,
            engine_version,
            allowed_capabilities,
        } => install_mod_package_for_engine_with_policy(
            &package_root,
            &install_root,
            engine_version.as_deref(),
            &security_policy(allowed_capabilities),
        )
        .map(|report| {
            format!(
                "installed mod package {} {} to {}",
                report.manifest.namespace,
                report.manifest.version,
                report.target_root.display()
            )
        })
        .map_err(format_mod_error),
        Command::PreflightInstallPackage {
            package_root,
            install_root,
            engine_version,
            allowed_capabilities,
        } => Ok(format_preflight_report(
            preflight_mod_package_install_for_engine_with_policy(
                &package_root,
                &install_root,
                engine_version.as_deref(),
                &security_policy(allowed_capabilities),
            ),
        )),
        Command::Help => Ok(usage()),
    }
}

fn parse_command(args: Vec<String>) -> Result<Command, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::Help);
    };

    match command {
        "new" => {
            if includes_help_option(&args[1..]) {
                return Ok(Command::Help);
            }
            let (positionals, options) = parse_options(&args[1..])?;
            if positionals.len() != 1 {
                return Err(format!("{}\n\nnew expects exactly one root path", usage()));
            }
            let Some(namespace) = options.namespace else {
                return Err(format!("{}\n\nnew requires --namespace", usage()));
            };
            Ok(Command::New {
                root: PathBuf::from(&positionals[0]),
                namespace,
                name: options.name,
                version: options.version.unwrap_or_else(|| "0.1.0".to_string()),
                engine_version: options
                    .engine_version
                    .unwrap_or_else(|| "0.1.0-m0".to_string()),
            })
        }
        "validate" => {
            if includes_help_option(&args[1..]) {
                return Ok(Command::Help);
            }
            let (positionals, options) = parse_options(&args[1..])?;
            if positionals.len() != 1 {
                return Err(format!(
                    "{}\n\nvalidate expects exactly one root path",
                    usage()
                ));
            }
            Ok(Command::Validate {
                root: PathBuf::from(&positionals[0]),
                engine_version: options.engine_version,
                allowed_capabilities: options.allowed_capabilities,
            })
        }
        "pack" => {
            if includes_help_option(&args[1..]) {
                return Ok(Command::Help);
            }
            let (positionals, options) = parse_options(&args[1..])?;
            if positionals.len() != 2 {
                return Err(format!(
                    "{}\n\npack expects source root and output root paths",
                    usage()
                ));
            }
            Ok(Command::Pack {
                source_root: PathBuf::from(&positionals[0]),
                output_root: PathBuf::from(&positionals[1]),
                engine_version: options.engine_version,
                allowed_capabilities: options.allowed_capabilities,
            })
        }
        "check-package" => {
            if includes_help_option(&args[1..]) {
                return Ok(Command::Help);
            }
            let (positionals, options) = parse_options(&args[1..])?;
            if positionals.len() != 1 {
                return Err(format!(
                    "{}\n\ncheck-package expects exactly one package root path",
                    usage()
                ));
            }
            Ok(Command::CheckPackage {
                package_root: PathBuf::from(&positionals[0]),
                engine_version: options.engine_version,
                allowed_capabilities: options.allowed_capabilities,
            })
        }
        "install-package" => {
            if includes_help_option(&args[1..]) {
                return Ok(Command::Help);
            }
            let (positionals, options) = parse_options(&args[1..])?;
            if positionals.len() != 2 {
                return Err(format!(
                    "{}\n\ninstall-package expects package root and install root paths",
                    usage()
                ));
            }
            Ok(Command::InstallPackage {
                package_root: PathBuf::from(&positionals[0]),
                install_root: PathBuf::from(&positionals[1]),
                engine_version: options.engine_version,
                allowed_capabilities: options.allowed_capabilities,
            })
        }
        "preflight-install-package" => {
            if includes_help_option(&args[1..]) {
                return Ok(Command::Help);
            }
            let (positionals, options) = parse_options(&args[1..])?;
            if positionals.len() != 2 {
                return Err(format!(
                    "{}\n\npreflight-install-package expects package root and install root paths",
                    usage()
                ));
            }
            Ok(Command::PreflightInstallPackage {
                package_root: PathBuf::from(&positionals[0]),
                install_root: PathBuf::from(&positionals[1]),
                engine_version: options.engine_version,
                allowed_capabilities: options.allowed_capabilities,
            })
        }
        "-h" | "--help" | "help" => Ok(Command::Help),
        unknown => Err(format!("{}\n\nunknown command: {unknown}", usage())),
    }
}

#[derive(Default)]
struct ParsedOptions {
    namespace: Option<String>,
    name: Option<String>,
    version: Option<String>,
    engine_version: Option<String>,
    allowed_capabilities: Vec<ModCapability>,
}

fn parse_options(args: &[String]) -> Result<(Vec<String>, ParsedOptions), String> {
    let mut positionals = Vec::new();
    let mut options = ParsedOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--namespace" => {
                index += 1;
                let Some(namespace) = args.get(index) else {
                    return Err("--namespace requires a value".to_string());
                };
                options.namespace = Some(namespace.clone());
            }
            "--name" => {
                index += 1;
                let Some(name) = args.get(index) else {
                    return Err("--name requires a value".to_string());
                };
                options.name = Some(name.clone());
            }
            "--version" => {
                index += 1;
                let Some(version) = args.get(index) else {
                    return Err("--version requires a value".to_string());
                };
                options.version = Some(version.clone());
            }
            "--engine-version" => {
                index += 1;
                let Some(version) = args.get(index) else {
                    return Err("--engine-version requires a value".to_string());
                };
                options.engine_version = Some(version.clone());
            }
            "--allow-capability" => {
                index += 1;
                let Some(capability) = args.get(index) else {
                    return Err("--allow-capability requires a value".to_string());
                };
                let Some(capability) = parse_mod_capability(capability) else {
                    return Err(format!("unknown mod capability: {capability}"));
                };
                options.allowed_capabilities.push(capability);
            }
            option if option.starts_with('-') => {
                return Err(format!("unknown option: {option}"));
            }
            positional => positionals.push(positional.to_string()),
        }
        index += 1;
    }

    Ok((positionals, options))
}

fn includes_help_option(args: &[String]) -> bool {
    args.iter()
        .any(|argument| argument == "-h" || argument == "--help")
}

fn format_mod_error(error: ModDiscoveryError) -> String {
    format!("mod command failed: {error}")
}

fn security_policy(allowed_capabilities: Vec<ModCapability>) -> ModSecurityPolicy {
    ModSecurityPolicy::with_authorized_unsafe_capabilities(allowed_capabilities)
}

fn format_preflight_report(report: ModInstallPreflightReport) -> String {
    let namespace = report
        .manifest
        .as_ref()
        .map(|manifest| manifest.namespace.as_str())
        .unwrap_or("unknown");
    let target = report
        .target_root
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| report.install_root.display().to_string());
    let errors = report
        .issues
        .iter()
        .filter(|issue| issue.severity == ModInstallPreflightIssueSeverity::Error)
        .count();
    let warnings = report
        .issues
        .iter()
        .filter(|issue| issue.severity == ModInstallPreflightIssueSeverity::Warning)
        .count();
    let status = if errors == 0 { "ready" } else { "blocked" };
    let mut lines = vec![format!(
        "preflight {status}: {namespace} to {target} ({errors} errors, {warnings} warnings)"
    )];
    for issue in report.issues {
        let severity = match issue.severity {
            ModInstallPreflightIssueSeverity::Error => "error",
            ModInstallPreflightIssueSeverity::Warning => "warning",
        };
        lines.push(format!(
            "{severity}: {}: {}",
            issue.path.display(),
            issue.error
        ));
    }
    lines.join("\n")
}

fn usage() -> String {
    [
        "ERAtw-NEXT Mod CLI",
        "",
        "Usage:",
        "  eratw-mod new <mod-root> --namespace <namespace> [--name <name>] [--version <version>] [--engine-version <version>]",
        "  eratw-mod validate <mod-root> [--engine-version <version>] [--allow-capability <capability>]",
        "  eratw-mod pack <mod-root> <output-root> [--engine-version <version>] [--allow-capability <capability>]",
        "  eratw-mod check-package <package-root> [--engine-version <version>] [--allow-capability <capability>]",
        "  eratw-mod preflight-install-package <package-root> <install-root> [--engine-version <version>] [--allow-capability <capability>]",
        "  eratw-mod install-package <package-root> <install-root> [--engine-version <version>] [--allow-capability <capability>]",
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
    fn new_command_creates_valid_project() {
        let root = temp_dir("new_command");

        let output = run(vec![
            "new".to_string(),
            root.to_string_lossy().to_string(),
            "--namespace".to_string(),
            "example.new_cli".to_string(),
            "--name".to_string(),
            "CLI 模板".to_string(),
        ])
        .unwrap();

        assert!(output.contains("created mod template example.new_cli 0.1.0"));
        assert!(root.join("manifest.json").exists());
        assert!(root.join("content/character.json").exists());

        let validate_output = run(vec![
            "validate".to_string(),
            root.to_string_lossy().to_string(),
            "--engine-version".to_string(),
            "0.1.0-m0".to_string(),
        ])
        .unwrap();
        assert!(validate_output.contains("valid mod example.new_cli 0.1.0"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn new_command_requires_namespace() {
        let root = temp_dir("new_command_missing_namespace");

        let error = run(vec!["new".to_string(), root.to_string_lossy().to_string()]).unwrap_err();

        assert!(error.contains("new requires --namespace"));
        assert!(!root.exists());
    }

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
    fn validate_command_requires_explicit_capability_authorization() {
        let root = temp_dir("validate_command_policy");
        write_manifest_with_capabilities(&root, "example.cli", &["network_access"]);

        let denied = run(vec![
            "validate".to_string(),
            root.to_string_lossy().to_string(),
        ])
        .unwrap_err();
        let allowed = run(vec![
            "validate".to_string(),
            root.to_string_lossy().to_string(),
            "--allow-capability".to_string(),
            "network_access".to_string(),
        ])
        .unwrap();

        assert!(denied.contains("unsafe capability"));
        assert!(allowed.contains("valid mod example.cli 0.1.0"));

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
    fn check_package_command_reports_valid_package() {
        let source_root = temp_dir("check_package_command_source");
        let output_root = temp_dir("check_package_command_output");
        write_manifest(&source_root, "example.cli");
        run(vec![
            "pack".to_string(),
            source_root.to_string_lossy().to_string(),
            output_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        let output = run(vec![
            "check-package".to_string(),
            output_root
                .join("example.cli-0.1.0")
                .to_string_lossy()
                .to_string(),
            "--engine-version".to_string(),
            "0.1.0-m0".to_string(),
        ])
        .unwrap();

        assert!(output.contains("checked mod package example.cli 0.1.0"));
        assert!(output.contains("0 resource errors, 0 resource warnings"));

        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn check_package_command_reports_resource_warnings() {
        let source_root = temp_dir("check_package_command_resource_source");
        let output_root = temp_dir("check_package_command_resource_output");
        write_manifest_with_resource(&source_root, "example.cli");
        fs::write(source_root.join("assets/readme.txt"), "publishable").unwrap();
        run(vec![
            "pack".to_string(),
            source_root.to_string_lossy().to_string(),
            output_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        let output = run(vec![
            "check-package".to_string(),
            output_root
                .join("example.cli-0.1.0")
                .to_string_lossy()
                .to_string(),
        ])
        .unwrap();

        assert!(output.contains("0 resource errors, 1 resource warnings"));

        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn preflight_install_package_command_reports_ready_package() {
        let source_root = temp_dir("preflight_package_command_source");
        let output_root = temp_dir("preflight_package_command_output");
        let install_root = temp_dir("preflight_package_command_root");
        write_manifest(&source_root, "example.cli");
        run(vec![
            "pack".to_string(),
            source_root.to_string_lossy().to_string(),
            output_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        let output = run(vec![
            "preflight-install-package".to_string(),
            output_root
                .join("example.cli-0.1.0")
                .to_string_lossy()
                .to_string(),
            install_root.to_string_lossy().to_string(),
            "--engine-version".to_string(),
            "0.1.0-m0".to_string(),
        ])
        .unwrap();

        assert!(output.contains("preflight ready: example.cli"));
        assert!(output.contains("0 errors, 0 warnings"));
        assert!(!install_root.exists());

        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn preflight_install_package_command_reports_resource_warnings() {
        let source_root = temp_dir("preflight_package_command_resource_source");
        let output_root = temp_dir("preflight_package_command_resource_output");
        let install_root = temp_dir("preflight_package_command_resource_root");
        write_manifest_with_resource(&source_root, "example.cli");
        fs::write(source_root.join("assets/readme.txt"), "publishable").unwrap();
        run(vec![
            "pack".to_string(),
            source_root.to_string_lossy().to_string(),
            output_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        let output = run(vec![
            "preflight-install-package".to_string(),
            output_root
                .join("example.cli-0.1.0")
                .to_string_lossy()
                .to_string(),
            install_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        assert!(output.contains("preflight ready: example.cli"));
        assert!(output.contains("0 errors, 1 warnings"));
        assert!(output.contains("mod package resource publication warning"));

        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn preflight_install_package_command_reports_existing_target() {
        let source_root = temp_dir("preflight_package_command_existing_source");
        let output_root = temp_dir("preflight_package_command_existing_output");
        let install_root = temp_dir("preflight_package_command_existing_root");
        fs::create_dir_all(install_root.join("example.cli")).unwrap();
        write_manifest(&source_root, "example.cli");
        run(vec![
            "pack".to_string(),
            source_root.to_string_lossy().to_string(),
            output_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        let output = run(vec![
            "preflight-install-package".to_string(),
            output_root
                .join("example.cli-0.1.0")
                .to_string_lossy()
                .to_string(),
            install_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        assert!(output.contains("preflight blocked: example.cli"));
        assert!(output.contains("install target already exists"));

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn install_package_command_installs_checked_package() {
        let source_root = temp_dir("install_package_command_source");
        let output_root = temp_dir("install_package_command_output");
        let install_root = temp_dir("install_package_command_root");
        write_manifest(&source_root, "example.cli");
        run(vec![
            "pack".to_string(),
            source_root.to_string_lossy().to_string(),
            output_root.to_string_lossy().to_string(),
        ])
        .unwrap();

        let output = run(vec![
            "install-package".to_string(),
            output_root
                .join("example.cli-0.1.0")
                .to_string_lossy()
                .to_string(),
            install_root.to_string_lossy().to_string(),
            "--engine-version".to_string(),
            "0.1.0-m0".to_string(),
        ])
        .unwrap();

        assert!(output.contains("installed mod package example.cli 0.1.0"));
        assert!(install_root.join("example.cli/manifest.json").exists());

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn install_package_command_uses_explicit_capability_authorization() {
        let source_root = temp_dir("install_package_command_policy_source");
        let output_root = temp_dir("install_package_command_policy_output");
        let install_root = temp_dir("install_package_command_policy_root");
        write_manifest_with_capabilities(&source_root, "example.cli", &["system_command"]);
        run(vec![
            "pack".to_string(),
            source_root.to_string_lossy().to_string(),
            output_root.to_string_lossy().to_string(),
            "--allow-capability".to_string(),
            "system_command".to_string(),
        ])
        .unwrap();

        let denied = run(vec![
            "install-package".to_string(),
            output_root
                .join("example.cli-0.1.0")
                .to_string_lossy()
                .to_string(),
            install_root.to_string_lossy().to_string(),
        ])
        .unwrap_err();
        let allowed = run(vec![
            "install-package".to_string(),
            output_root
                .join("example.cli-0.1.0")
                .to_string_lossy()
                .to_string(),
            install_root.to_string_lossy().to_string(),
            "--allow-capability".to_string(),
            "system_command".to_string(),
        ])
        .unwrap();

        assert!(denied.contains("unsafe capability"));
        assert!(allowed.contains("installed mod package example.cli 0.1.0"));
        assert!(install_root.join("example.cli/manifest.json").exists());

        let _ = fs::remove_dir_all(install_root);
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
        write_manifest_with_capabilities(root, namespace, &["content"]);
    }

    fn write_manifest_with_capabilities(root: &PathBuf, namespace: &str, capabilities: &[&str]) {
        fs::create_dir_all(root).unwrap();
        let capabilities = capabilities
            .iter()
            .map(|capability| format!(r#""{capability}""#))
            .collect::<Vec<_>>()
            .join(", ");
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
  "capabilities": [{capabilities}]
}}"#
            ),
        )
        .unwrap();
    }

    fn write_manifest_with_resource(root: &PathBuf, namespace: &str) {
        fs::create_dir_all(root.join("assets")).unwrap();
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
  "capabilities": ["content"],
  "resources": [
    {{
      "resource_id": "{namespace}.assets.readme",
      "source_path": "assets/readme.txt",
      "media_type": "other",
      "license": "CC-BY-4.0",
      "author": "ERAtw-NEXT",
      "usage": [],
      "character_bindings": [],
      "tags": [],
      "sha256": null
    }}
  ]
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
