use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

type ToolResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(
    name = "reference-tools",
    version,
    about = "Reference data maintenance utilities"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Normalize reference datasets to their canonical formats
    Normalize {
        #[arg(value_enum)]
        target: NormalizeTarget,
    },
    /// Validate reference datasets for structural integrity
    Validate {
        #[arg(value_enum, default_value_t = ValidateTarget::All)]
        target: ValidateTarget,
    },
}

#[derive(Clone, ValueEnum, Debug)]
enum NormalizeTarget {
    /// Normalize .reference/testing_web3_full.csv
    Web3,
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq, Debug)]
enum ValidateTarget {
    All,
    Detection,
    Governance,
    Protection,
    Resilience,
    Security,
    Web3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SpecKind {
    Detection,
    Governance,
    Protection,
    Resilience,
    Security,
    Web3,
}

struct ReferenceSpec {
    kind: SpecKind,
    file_name: &'static str,
    prefix: &'static str,
    slug_columns: &'static [&'static str],
    required_columns: Option<&'static [&'static str]>,
    expected_rows: usize,
}

#[derive(Debug, Deserialize, Serialize)]
struct Web3Row {
    category: String,
    test_type: String,
    component: String,
    behavior: String,
    condition: String,
    test_name: String,
}

fn main() -> ToolResult<()> {
    let cli = Cli::parse();
    let workspace_root = workspace_root();

    match cli.command {
        Command::Normalize { target } => match target {
            NormalizeTarget::Web3 => {
                let csv_path = workspace_root.join(".reference/testing_web3_full.csv");
                let updated = normalize_testing_web3(&csv_path)?;
                if updated {
                    println!("Normalized test_name values in {}", csv_path.display());
                } else {
                    println!("testing_web3_full.csv already normalized");
                }
            }
        },
        Command::Validate { target } => {
            run_validations(&workspace_root, target)?;
            println!("Reference validation complete for {:?}", target);
        }
    }

    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("reference-tools must live inside the workspace root")
        .to_path_buf()
}

fn normalize_testing_web3(csv_path: &Path) -> ToolResult<bool> {
    let mut reader = csv::Reader::from_path(csv_path)
        .map_err(|err| format!("failed to open {}: {err}", csv_path.display()))?;

    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    let mut updated = 0usize;

    for row in reader.deserialize::<Web3Row>() {
        let mut entry = row
            .map_err(|err| format!("failed to deserialize row in {}: {err}", csv_path.display()))?;

        let normalized = format!(
            "test_testing__{}__{}__{}__{}__{}",
            slugify(&entry.category),
            slugify(&entry.test_type),
            slugify(&entry.component),
            slugify(&entry.behavior),
            slugify(&entry.condition),
        );

        if entry.test_name != normalized {
            updated += 1;
            entry.test_name = normalized;
        }

        if !seen.insert(entry.test_name.clone()) {
            return Err(format!("duplicate test_name detected: {}", entry.test_name).into());
        }

        rows.push(entry);
    }

    if updated == 0 {
        return Ok(false);
    }

    let tmp_path = tmp_path(csv_path);
    {
        let mut writer = csv::Writer::from_path(&tmp_path)
            .map_err(|err| format!("failed to open {} for writing: {err}", tmp_path.display()))?;
        for row in rows {
            writer.serialize(row)?;
        }
        writer.flush()?;
    }

    fs::rename(&tmp_path, csv_path)
        .map_err(|err| format!("failed to replace {}: {err}", csv_path.display()))?;

    Ok(true)
}

fn run_validations(root: &Path, target: ValidateTarget) -> ToolResult<()> {
    let specs = reference_specs();
    let filtered: Vec<&ReferenceSpec> = specs
        .iter()
        .filter(|spec| target.matches(spec.kind))
        .collect();

    for spec in filtered {
        validate_reference(root, spec)?;
    }
    Ok(())
}

fn reference_specs() -> Vec<ReferenceSpec> {
    vec![
        ReferenceSpec {
            kind: SpecKind::Detection,
            file_name: "detection_response_tests_full.csv",
            prefix: "test_detection_response__",
            slug_columns: &["main_type", "component", "behavior", "condition"],
            required_columns: None,
            expected_rows: 4_800,
        },
        ReferenceSpec {
            kind: SpecKind::Governance,
            file_name: "governance_compliance_full.csv",
            prefix: "test_governance_compliance__",
            slug_columns: &["main_type", "component", "behavior", "condition"],
            required_columns: None,
            expected_rows: 5_400,
        },
        ReferenceSpec {
            kind: SpecKind::Protection,
            file_name: "protection_tests_full.csv",
            prefix: "test_protection__",
            slug_columns: &["layer", "component", "behavior", "condition"],
            required_columns: None,
            expected_rows: 5_670,
        },
        ReferenceSpec {
            kind: SpecKind::Resilience,
            file_name: "resilience_recovery.csv",
            prefix: "test_resilience_recovery__",
            slug_columns: &["component", "behavior", "condition"],
            required_columns: Some(&["layer", "component", "behavior", "condition"]),
            expected_rows: 432,
        },
        ReferenceSpec {
            kind: SpecKind::Security,
            file_name: "security_tests_full.csv",
            prefix: "test_security__",
            slug_columns: &["layer", "component", "behavior", "condition"],
            required_columns: None,
            expected_rows: 3_168,
        },
        ReferenceSpec {
            kind: SpecKind::Web3,
            file_name: "testing_web3_full.csv",
            prefix: "test_testing__",
            slug_columns: &[
                "category",
                "test_type",
                "component",
                "behavior",
                "condition",
            ],
            required_columns: None,
            expected_rows: 2_980,
        },
    ]
}

fn validate_reference(root: &Path, spec: &ReferenceSpec) -> ToolResult<()> {
    let csv_path = root.join(".reference").join(spec.file_name);
    let mut reader = csv::Reader::from_path(&csv_path)
        .map_err(|err| format!("failed to open {}: {err}", csv_path.display()))?;

    let headers = reader
        .headers()
        .map_err(|err| format!("failed to read headers for {}: {err}", csv_path.display()))?
        .clone();

    let test_name_idx = headers
        .iter()
        .position(|h| h == "test_name")
        .ok_or_else(|| format!("{} missing test_name column", spec.file_name))?;

    let slug_indexes: Vec<usize> = spec
        .slug_columns
        .iter()
        .map(|column| {
            headers
                .iter()
                .position(|h| h == *column)
                .ok_or_else(|| format!("column {column} missing from {}", spec.file_name))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let required_columns = spec.required_columns.unwrap_or(spec.slug_columns);
    let required_indexes: Vec<usize> = required_columns
        .iter()
        .map(|column| {
            headers
                .iter()
                .position(|h| h == *column)
                .ok_or_else(|| format!("column {column} missing from {}", spec.file_name))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut seen_names = HashSet::new();
    let mut row_count = 0usize;

    for record in reader.records() {
        let record = record.map_err(|err| {
            format!(
                "failed to deserialize row {row_count} of {}: {err}",
                csv_path.display()
            )
        })?;
        row_count += 1;

        let test_name = record.get(test_name_idx).unwrap_or("").trim().to_string();
        if !test_name.starts_with(spec.prefix) {
            return Err(format!(
                "test_name {} in {} must start with {}",
                test_name, spec.file_name, spec.prefix
            )
            .into());
        }

        if !seen_names.insert(test_name.clone()) {
            return Err(format!(
                "duplicate test_name {} found in {}",
                test_name, spec.file_name
            )
            .into());
        }

        for (idx, column_name) in required_indexes.iter().zip(required_columns.iter()) {
            let cell = record.get(*idx).unwrap_or("").trim();
            if cell.is_empty() {
                return Err(format!(
                    "column {} cannot be empty in {} (row {row_count})",
                    column_name, spec.file_name
                )
                .into());
            }
        }

        if !slug_indexes.is_empty() {
            let mut slug_parts = Vec::with_capacity(slug_indexes.len());
            for idx in slug_indexes.iter() {
                let cell = record.get(*idx).unwrap_or("").trim().to_string();
                slug_parts.push(slugify(&cell));
            }

            let expected_tail = slug_parts.join("__");
            let expected_full = format!("{}{}", spec.prefix, expected_tail);
            if relaxed_slug(&test_name) != relaxed_slug(&expected_full) {
                return Err(format!(
                    "test_name {} does not encode its row in {}",
                    test_name, spec.file_name
                )
                .into());
            }
        }
    }

    if row_count != spec.expected_rows {
        return Err(format!(
            "{} expected {} rows but found {}",
            spec.file_name, spec.expected_rows, row_count
        )
        .into());
    }

    Ok(())
}

fn tmp_path(csv_path: &Path) -> PathBuf {
    let mut tmp = csv_path.as_os_str().to_os_string();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

fn slugify(input: &str) -> String {
    let normalized = input
        .replace(" & ", " and ")
        .replace('&', " and ")
        .to_lowercase();

    let mut slug = String::with_capacity(normalized.len());
    for ch in normalized.chars() {
        match ch {
            'a'..='z' | '0'..='9' => slug.push(ch),
            '_' => slug.push('_'),
            _ => slug.push('_'),
        }
    }

    slug.trim_matches('_').to_string()
}

fn relaxed_slug(input: &str) -> String {
    let base = slugify(input);
    let mut compact = String::with_capacity(base.len());
    let mut prev_was_underscore = false;

    for ch in base.chars() {
        if ch == '_' {
            if !prev_was_underscore {
                compact.push(ch);
                prev_was_underscore = true;
            }
        } else {
            prev_was_underscore = false;
            compact.push(ch);
        }
    }

    compact
}

impl ValidateTarget {
    fn matches(self, kind: SpecKind) -> bool {
        match self {
            ValidateTarget::All => true,
            ValidateTarget::Detection => kind == SpecKind::Detection,
            ValidateTarget::Governance => kind == SpecKind::Governance,
            ValidateTarget::Protection => kind == SpecKind::Protection,
            ValidateTarget::Resilience => kind == SpecKind::Resilience,
            ValidateTarget::Security => kind == SpecKind::Security,
            ValidateTarget::Web3 => kind == SpecKind::Web3,
        }
    }
}
