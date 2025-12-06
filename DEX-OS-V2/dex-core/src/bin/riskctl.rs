use clap::{Parser, Subcommand};
use dex_core::governance::{GovernanceReferenceIndex, RiskRegistry};

#[derive(Debug, Parser)]
#[command(author, version, about = "Risk registry helper for DEX-OS governance", long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Write (or initialize) the exception register CSV to the configured path
    SaveRegister,
    /// Print the count of currently open risks
    OpenCount,
}

fn main() -> Result<(), dex_core::governance::RiskError> {
    let args = Args::parse();
    let index = GovernanceReferenceIndex::shared()
        .map_err(|_| dex_core::governance::RiskError::ScenarioMissing)?;
    let mut registry = RiskRegistry::new(&index)?;

    match args.cmd {
        Command::SaveRegister => {
            registry.save_exception_register()?;
            println!("exception register saved");
        }
        Command::OpenCount => {
            println!("{}", registry.open_risks_count());
        }
    }

    Ok(())
}
