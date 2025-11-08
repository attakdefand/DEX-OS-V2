use std::collections::HashSet;

use dex_core::reference_common::{reference_root, relaxed_slug, slugify};

struct ReferenceSpec {
    file_name: &'static str,
    prefix: Option<&'static str>,
    slug_columns: &'static [&'static str],
    required_columns: Option<&'static [&'static str]>,
    expected_rows: usize,
}

#[test]
fn reference_csvs_are_consistent() {
    let specs = [
        ReferenceSpec {
            file_name: "detection_response_tests_full.csv",
            prefix: Some("test_detection_response__"),
            slug_columns: &["main_type", "component", "behavior", "condition"],
            required_columns: None,
            expected_rows: 4_800,
        },
        ReferenceSpec {
            file_name: "governance_compliance_full.csv",
            prefix: Some("test_governance_compliance__"),
            slug_columns: &["main_type", "component", "behavior", "condition"],
            required_columns: None,
            expected_rows: 5_400,
        },
        ReferenceSpec {
            file_name: "protection_tests_full.csv",
            prefix: Some("test_protection__"),
            slug_columns: &["layer", "component", "behavior", "condition"],
            required_columns: None,
            expected_rows: 5_670,
        },
        ReferenceSpec {
            file_name: "resilience_recovery.csv",
            prefix: Some("test_resilience_recovery__"),
            slug_columns: &["component", "behavior", "condition"],
            required_columns: Some(&["layer", "component", "behavior", "condition"]),
            expected_rows: 432,
        },
        ReferenceSpec {
            file_name: "security_tests_full.csv",
            prefix: Some("test_security__"),
            slug_columns: &["layer", "component", "behavior", "condition"],
            required_columns: None,
            expected_rows: 3_168,
        },
        ReferenceSpec {
            file_name: "testing_web3_full.csv",
            prefix: Some("test_testing__"),
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
    ];

    for spec in specs {
        validate_reference(spec);
    }
}

fn validate_reference(spec: ReferenceSpec) {
    let csv_path = reference_root().join(spec.file_name);
    assert!(
        csv_path.exists(),
        "missing reference file at {}",
        csv_path.display()
    );

    let mut reader = csv::Reader::from_path(&csv_path)
        .unwrap_or_else(|err| panic!("failed to open {}: {err}", spec.file_name));

    let headers = reader
        .headers()
        .unwrap_or_else(|err| panic!("failed to read headers for {}: {err}", spec.file_name))
        .clone();

    let test_name_idx = headers
        .iter()
        .position(|h| h == "test_name")
        .unwrap_or_else(|| panic!("{} missing test_name column", spec.file_name));

    let slug_indexes: Vec<usize> = spec
        .slug_columns
        .iter()
        .map(|column| {
            headers
                .iter()
                .position(|h| h == *column)
                .unwrap_or_else(|| panic!("column {column} missing from {}", spec.file_name))
        })
        .collect();

    let required_columns = spec.required_columns.unwrap_or(spec.slug_columns);
    let required_indexes: Vec<usize> = required_columns
        .iter()
        .map(|column| {
            headers
                .iter()
                .position(|h| h == *column)
                .unwrap_or_else(|| panic!("column {column} missing from {}", spec.file_name))
        })
        .collect();

    let mut seen_names = HashSet::new();
    let mut row_count = 0usize;

    for record in reader.records() {
        let record = record.unwrap_or_else(|err| {
            panic!(
                "failed to deserialize row {row_count} of {}: {err}",
                spec.file_name
            )
        });
        row_count += 1;

        let test_name = record.get(test_name_idx).unwrap_or("").trim().to_string();
        if let Some(prefix) = spec.prefix {
            assert!(
                test_name.starts_with(prefix),
                "test_name {test_name} in {} must start with {}",
                spec.file_name,
                prefix
            );
        } else {
            assert!(
                test_name.starts_with("test_"),
                "test_name {test_name} in {} must start with test_",
                spec.file_name
            );
        }

        assert!(
            seen_names.insert(test_name.clone()),
            "duplicate test_name {test_name} found in {}",
            spec.file_name
        );

        for (idx, column_name) in required_indexes.iter().zip(required_columns.iter()) {
            let cell = record.get(*idx).unwrap_or("").trim().to_string();
            assert!(
                !cell.is_empty(),
                "column {} cannot be empty in {} (row {row_count})",
                column_name,
                spec.file_name
            );
        }

        if let Some(prefix) = spec.prefix {
            if !slug_indexes.is_empty() {
                let mut slug_parts = Vec::with_capacity(slug_indexes.len());
                for idx in slug_indexes.iter() {
                    let cell = record.get(*idx).unwrap_or("").trim().to_string();
                    slug_parts.push(slugify(&cell));
                }

                let expected_tail = slug_parts.join("__");
                let expected_full = format!("{}{}", prefix, expected_tail);
                assert_eq!(
                    relaxed_slug(&test_name),
                    relaxed_slug(&expected_full),
                    "test_name {test_name} does not encode the row from {}",
                    spec.file_name
                );
            }
        }
    }

    assert_eq!(
        row_count, spec.expected_rows,
        "{} expected {} rows but found {}",
        spec.file_name, spec.expected_rows, row_count
    );
}
