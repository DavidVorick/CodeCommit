# API Signatures

pub async fn build_codebase_context(
    next_agent_full_prompt: &str,
    config: &config::Config,
    logger: &logger::Logger,
) -> Result<String, AppError>