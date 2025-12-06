use crate::config::AppState;
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct DatasetSummary {
    pub name: String,
    pub path: String,
    pub rows: usize,
    pub columns: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadSummary {
    pub top_level: Vec<DatasetSummary>,
    pub gold: Vec<DatasetSummary>,
    pub diamond: Vec<DatasetSummary>,
    pub total_rows: usize,
}

pub async fn load_all(state: Arc<AppState>) -> Result<LoadSummary> {
    let top_level = load_named(
        &state.reference_root,
        &[
            "detection_response_tests_full.csv",
            "protection_tests_full.csv",
            "security_tests_full.csv",
            "resilience_recovery.csv",
            "testing_web3_full.csv",
            "governance_compliance_full.csv",
            "governance_compliance_full_enriched.csv",
        ],
    )
    .await?;

    let gold_dir = state.reference_root.join("layers").join("gold");
    let diamond_dir = gold_dir.join("diamond");

    let gold = load_csvs_in_dir(&gold_dir, 1).await?;
    let diamond = load_csvs_in_dir(&diamond_dir, 1).await?;

    let total_rows = top_level
        .iter()
        .chain(gold.iter())
        .chain(diamond.iter())
        .map(|dataset| dataset.rows)
        .sum();

    Ok(LoadSummary {
        top_level,
        gold,
        diamond,
        total_rows,
    })
}

async fn load_named(root: &Path, names: &[&str]) -> Result<Vec<DatasetSummary>> {
    let mut summaries = Vec::new();

    for name in names {
        let path = root.join(name);
        if !path.exists() {
            tracing::warn!("Skipping missing dataset: {}", path.display());
            continue;
        }

        summaries.push(summarize_csv(name.to_string(), path).await?);
    }

    Ok(summaries)
}

async fn load_csvs_in_dir(dir: &Path, max_depth: usize) -> Result<Vec<DatasetSummary>> {
    if !dir.exists() {
        tracing::warn!("Directory not found: {}", dir.display());
        return Ok(Vec::new());
    }

    let mut paths: Vec<PathBuf> = WalkDir::new(dir)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("csv"))
                .unwrap_or(false)
        })
        .collect();

    paths.sort();

    let mut summaries = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        summaries.push(summarize_csv(name, path).await?);
    }

    Ok(summaries)
}

async fn summarize_csv(name: String, path: PathBuf) -> Result<DatasetSummary> {
    let summary = task::spawn_blocking(move || -> Result<DatasetSummary> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(&path)
            .with_context(|| format!("opening {}", path.display()))?;

        let mut rows = 0usize;
        let mut columns = reader.headers().map(|h| h.len()).unwrap_or(0);

        for record in reader.records() {
            let rec = record?;
            columns = columns.max(rec.len());
            rows += 1;
        }

        Ok(DatasetSummary {
            name,
            path: path.display().to_string(),
            rows,
            columns,
        })
    })
    .await
    .context("csv parsing task failed")??;

    Ok(summary)
}
