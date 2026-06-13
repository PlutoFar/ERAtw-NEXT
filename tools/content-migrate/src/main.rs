use eratw_next_content_migrate::{run_migration, MigrationError, MigrationOptions};
use std::path::PathBuf;

fn main() {
    match parse_args().and_then(|options| run_migration(&options).map(|package| (options, package)))
    {
        Ok((options, package)) => {
            println!("draft content package generated");
            println!("out: {}", options.out_dir.display());
            println!(
                "objects: {}",
                package.migration_report.summary.objects_generated
            );
            println!(
                "unmapped: {}",
                package.migration_report.summary.unmapped_items
            );
            println!("valid: {}", package.validation_report.valid);
        }
        Err(err) => {
            eprintln!("content migration failed: {err}");
            std::process::exit(2);
        }
    }
}

fn parse_args() -> Result<MigrationOptions, MigrationError> {
    let mut audit_dir = None;
    let mut out_dir = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--audit" => audit_dir = Some(PathBuf::from(next_value(&mut args, "--audit")?)),
            "--out" => out_dir = Some(PathBuf::from(next_value(&mut args, "--out")?)),
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(MigrationError::InvalidInput(format!(
                    "unknown argument '{other}'"
                )));
            }
        }
    }
    let audit_dir =
        audit_dir.ok_or_else(|| MigrationError::InvalidInput("--audit is required".to_string()))?;
    let out_dir =
        out_dir.ok_or_else(|| MigrationError::InvalidInput("--out is required".to_string()))?;
    Ok(MigrationOptions::new(audit_dir, out_dir))
}

fn next_value(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, MigrationError> {
    args.next()
        .ok_or_else(|| MigrationError::InvalidInput(format!("{flag} requires a value")))
}

fn print_help() {
    println!(
        "ERAtw-NEXT M2 draft content package generator\n\
\n\
Usage:\n\
  cargo run -p eratw_next_content_migrate -- \\\n\
    --audit D:\\path\\to\\content-audit-report \\\n\
    --out D:\\path\\outside\\ERAtw-NEXT\\content-package\n\
\n\
The generator consumes M1 reports only. It does not read legacy source bodies,\n\
copy assets, execute scripts, or write output inside the engine repository.\n\
The output path must be absolute, outside the audit input, and not already exist."
    );
}
