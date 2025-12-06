use dex_core::governance::{build_compliance_report, render_report_json};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = build_compliance_report()?;
    let json = render_report_json(&report);
    let out_path = env::var("COMPLIANCE_REPORT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("compliance_report.json"));
    fs::write(&out_path, json)?;
    println!("wrote compliance report to {}", out_path.display());
    Ok(())
}
