use anyhow::Result;
use code_commit::run;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> Result<ExitCode> {
    // Run the main application logic from the library crate
    run().await
}
