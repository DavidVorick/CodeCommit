# API Signatures

pub async fn run(
    logger: &logger::Logger,
    cli_args: cli::CliArgs,
) -> Result<(), AppError>

pub async fn run_with_codebase(
    logger: &logger::Logger,
    config: &config::Config,
    codebase: String,
) -> Result<String, AppError>