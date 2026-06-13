use eratw_next_content_audit::{default_out_dir, run_audit, AuditError, AuditOptions};
use std::path::PathBuf;

fn main() {
    match parse_args().and_then(|options| run_audit(&options).map(|report| (options, report))) {
        Ok((options, report)) => {
            println!("content audit complete");
            println!("source: {}", report.summary.source_root);
            println!("out: {}", options.out_dir.display());
            println!("files: {}", report.summary.totals.files);
            println!("risks: {}", report.summary.risks.len());
        }
        Err(err) => {
            eprintln!("content audit failed: {err}");
            std::process::exit(2);
        }
    }
}

fn parse_args() -> Result<AuditOptions, AuditError> {
    let mut source = PathBuf::from(eratw_next_content_audit::DEFAULT_ALLOWED_SOURCE);
    let mut out_dir = default_out_dir();
    let mut profile = eratw_next_content_audit::DEFAULT_PROFILE.to_string();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => {
                source = PathBuf::from(next_value(&mut args, "--source")?);
            }
            "--out" => {
                out_dir = PathBuf::from(next_value(&mut args, "--out")?);
            }
            "--profile" => {
                profile = next_value(&mut args, "--profile")?;
            }
            "--no-network" | "--no-execute" => {}
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(AuditError::InvalidArgs(format!(
                    "unknown argument '{other}'"
                )));
            }
        }
    }

    let mut options = AuditOptions::new(source, out_dir);
    options.profile = profile;
    Ok(options)
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, AuditError> {
    args.next()
        .ok_or_else(|| AuditError::InvalidArgs(format!("{flag} requires a value")))
}

fn print_help() {
    println!(
        "ERAtw-NEXT M1 readonly content audit\n\
\n\
Usage:\n\
  cargo run -p eratw_next_content_audit -- \\\n\
    --source D:\\AICODE\\eratw-content \\\n\
    --out reports\\content-audit\\manual \\\n\
    --profile m1-readonly \\\n\
    --no-network --no-execute\n\
\n\
Safety:\n\
  - only the allowlisted eratw-content source is accepted\n\
  - .git directories and reparse points are skipped\n\
  - output must be new and outside the readonly source\n\
  - text/resource bodies are read only for aggregate statistics and hashes\n\
  - source content is never emitted, modified, or executed"
    );
}
