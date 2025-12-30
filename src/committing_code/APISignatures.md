# API Signatures

pub async fn run(
    logger: &logger::Logger,
    cli_args: cli::CliArgs,
) -> Result<(), app_error::AppError>

pub async fn run_with_codebase(
    logger: &logger::Logger,
    config: &config::Config,
    codebase: String,
) -> Result<String, app_error::AppError>
