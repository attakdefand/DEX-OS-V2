use crate::config::AppState;
use crate::results::{CommandResult, TestRunSummary};
use axum::{extract::State, Json};
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;

pub async fn run_all_tests(State(state): State<Arc<AppState>>) -> Json<TestRunSummary> {
    let mut results = Vec::new();

    results.push(
        run_if_exists(
            "cargo test (rust)",
            &state.tests_out_root.join("rust"),
            "cargo",
            &["test"],
        )
        .await,
    );

    results.push(
        run_if_exists(
            "forge test (solidity)",
            &state.tests_out_root.join("solidity"),
            "forge",
            &["test"],
        )
        .await,
    );

    results.push(
        run_if_exists(
            "python tests",
            &state.tests_out_root.join("python"),
            "python",
            &["-m", "pytest"],
        )
        .await,
    );

    Json(TestRunSummary { results })
}

async fn run_if_exists(label: &str, dir: &Path, bin: &str, args: &[&str]) -> CommandResult {
    if !dir.exists() {
        return CommandResult {
            label: label.to_string(),
            command: format!("{} {}", bin, args.join(" ")),
            status: "skipped (missing directory)".to_string(),
            stdout: String::new(),
            stderr: String::new(),
            skipped: true,
        };
    }

    let output = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .output()
        .await;

    match output {
        Ok(out) => {
            let status = if out.status.success() {
                "ok".to_string()
            } else {
                format!("failed ({})", out.status)
            };

            CommandResult {
                label: label.to_string(),
                command: format!("{} {}", bin, args.join(" ")),
                status,
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                skipped: false,
            }
        }
        Err(err) => CommandResult {
            label: label.to_string(),
            command: format!("{} {}", bin, args.join(" ")),
            status: format!("spawn error: {}", err),
            stdout: String::new(),
            stderr: String::new(),
            skipped: true,
        },
    }
}
