use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandResult {
    pub label: String,
    pub command: String,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
    pub skipped: bool,
}

#[derive(Debug, Serialize)]
pub struct TestRunSummary {
    pub results: Vec<CommandResult>,
}
