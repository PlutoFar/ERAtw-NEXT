use encoding_rs::{SHIFT_JIS, UTF_16BE, UTF_16LE, UTF_8};
use regex::Regex;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug)]
pub(crate) struct DecodedText {
    pub encoding: String,
    pub had_errors: bool,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ErbFileAnalysis {
    pub encoding: String,
    pub decode_errors: bool,
    pub lines: u64,
    pub blank_lines: u64,
    pub comment_lines: u64,
    pub function_definitions: u64,
    pub calls: u64,
    pub conditionals: u64,
    pub select_cases: u64,
    pub print_commands: u64,
    pub variable_references: u64,
    pub resource_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CsvFileAnalysis {
    pub encoding: String,
    pub decode_errors: bool,
    pub delimiter: String,
    pub rows: u64,
    pub min_columns: u64,
    pub max_columns: u64,
    pub blank_rows: u64,
    pub duplicate_first_column_values: u64,
    pub parse_errors: u64,
}

pub(crate) fn decode_text_file(path: &Path) -> io::Result<DecodedText> {
    let bytes = fs::read(path)?;
    Ok(decode_bytes(&bytes))
}

fn decode_bytes(bytes: &[u8]) -> DecodedText {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return decode_with(UTF_8, &bytes[3..], "utf-8-bom");
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return decode_with(UTF_16LE, &bytes[2..], "utf-16le");
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return decode_with(UTF_16BE, &bytes[2..], "utf-16be");
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return DecodedText {
            encoding: "utf-8".to_string(),
            had_errors: false,
            text: text.to_string(),
        };
    }
    decode_with(SHIFT_JIS, bytes, "shift_jis")
}

fn decode_with(encoding: &'static encoding_rs::Encoding, bytes: &[u8], label: &str) -> DecodedText {
    let (text, _, had_errors) = encoding.decode(bytes);
    DecodedText {
        encoding: label.to_string(),
        had_errors,
        text: into_owned(text),
    }
}

fn into_owned(text: Cow<'_, str>) -> String {
    match text {
        Cow::Borrowed(value) => value.to_string(),
        Cow::Owned(value) => value,
    }
}

pub(crate) fn analyze_erb(path: &Path) -> io::Result<ErbFileAnalysis> {
    let decoded = decode_text_file(path)?;
    let mut lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;
    for line in decoded.text.lines() {
        lines += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_lines += 1;
        } else if trimmed.starts_with(';') {
            comment_lines += 1;
        }
    }

    let resource_references = resource_reference_regex()
        .find_iter(&decoded.text)
        .filter_map(|capture| {
            Path::new(capture.as_str())
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    Ok(ErbFileAnalysis {
        encoding: decoded.encoding,
        decode_errors: decoded.had_errors,
        lines,
        blank_lines,
        comment_lines,
        function_definitions: function_regex().find_iter(&decoded.text).count() as u64,
        calls: call_regex().find_iter(&decoded.text).count() as u64,
        conditionals: conditional_regex().find_iter(&decoded.text).count() as u64,
        select_cases: select_case_regex().find_iter(&decoded.text).count() as u64,
        print_commands: print_regex().find_iter(&decoded.text).count() as u64,
        variable_references: variable_regex().find_iter(&decoded.text).count() as u64,
        resource_references,
    })
}

pub(crate) fn analyze_csv(path: &Path) -> io::Result<CsvFileAnalysis> {
    let decoded = decode_text_file(path)?;
    let delimiter = detect_delimiter(&decoded.text);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(decoded.text.as_bytes());

    let mut rows = 0;
    let mut min_columns = u64::MAX;
    let mut max_columns = 0;
    let mut blank_rows = 0;
    let mut duplicate_first_column_values = 0;
    let mut parse_errors = 0;
    let mut first_column_values = HashSet::new();

    for result in reader.records() {
        match result {
            Ok(record) => {
                rows += 1;
                let columns = record.len() as u64;
                min_columns = min_columns.min(columns);
                max_columns = max_columns.max(columns);
                if record.iter().all(|field| field.trim().is_empty()) {
                    blank_rows += 1;
                }
                if let Some(first) = record.get(0) {
                    let first = first.trim();
                    if !first.is_empty() && !first_column_values.insert(first.to_string()) {
                        duplicate_first_column_values += 1;
                    }
                }
            }
            Err(_) => parse_errors += 1,
        }
    }

    if rows == 0 {
        min_columns = 0;
    }

    Ok(CsvFileAnalysis {
        encoding: decoded.encoding,
        decode_errors: decoded.had_errors,
        delimiter: delimiter_name(delimiter).to_string(),
        rows,
        min_columns,
        max_columns,
        blank_rows,
        duplicate_first_column_values,
        parse_errors,
    })
}

fn detect_delimiter(text: &str) -> u8 {
    let mut counts = BTreeMap::from([(b',', 0usize), (b'\t', 0), (b';', 0)]);
    for line in text.lines().filter(|line| !line.trim().is_empty()).take(64) {
        for (delimiter, count) in &mut counts {
            *count += line
                .as_bytes()
                .iter()
                .filter(|byte| *byte == delimiter)
                .count();
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .filter(|(_, count)| *count > 0)
        .map(|(delimiter, _)| delimiter)
        .unwrap_or(b',')
}

fn delimiter_name(delimiter: u8) -> &'static str {
    match delimiter {
        b'\t' => "tab",
        b';' => "semicolon",
        _ => "comma",
    }
}

fn function_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)^\s*@[^\s(;]+").expect("valid function regex"))
}

fn call_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?im)^\s*(?:TRY)?CALL(?:FORM)?\b").expect("valid call regex"))
}

fn conditional_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?im)^\s*(?:SIF|IF|ELSEIF|ELSE|ENDIF)\b").expect("valid conditional regex")
    })
}

fn select_case_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?im)^\s*(?:SELECTCASE|CASEELSE|CASE|ENDSELECT)\b")
            .expect("valid select-case regex")
    })
}

fn print_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?im)^\s*PRINT[A-Z]*\b").expect("valid print regex"))
}

fn variable_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:ARG|ARGS|LOCAL|LOCALS|FLAG|CFLAG|TFLAG|BASE|ABL|TALENT|EXP|PALAM|JUEL|ITEM|STR|CSTR|TCVAR)(?::|\[|\b)",
        )
        .expect("valid variable regex")
    })
}

fn resource_reference_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?i)[^\s,'"<>|]+?\.(?:webp|png|jpe?g|mp3|mid|wav|ogg|flac|ttf|ttc|otf)"#)
            .expect("valid resource reference regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_erb_categories_without_returning_body() {
        let path = test_file(
            "sample.erb",
            b"@TEST\nIF FLAG:1\nCALL OTHER\nPRINTFORM hello.png\nENDIF\n",
        );
        let analysis = analyze_erb(&path).unwrap();
        assert_eq!(analysis.function_definitions, 1);
        assert_eq!(analysis.calls, 1);
        assert_eq!(analysis.conditionals, 2);
        assert_eq!(analysis.print_commands, 1);
        assert_eq!(analysis.variable_references, 1);
        assert_eq!(analysis.resource_references, vec!["hello.png"]);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn analyzes_csv_shape_and_duplicate_first_column() {
        let path = test_file("sample.csv", b"1,a\n1,b\n2,c,d\n");
        let analysis = analyze_csv(&path).unwrap();
        assert_eq!(analysis.rows, 3);
        assert_eq!(analysis.min_columns, 2);
        assert_eq!(analysis.max_columns, 3);
        assert_eq!(analysis.duplicate_first_column_values, 1);
        fs::remove_file(path).unwrap();
    }

    fn test_file(name: &str, body: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "eratw_next_text_analysis_{}_{}_{}",
            std::process::id(),
            name,
            body.len()
        ));
        fs::write(&path, body).unwrap();
        path
    }
}
