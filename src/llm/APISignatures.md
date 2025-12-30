pub async fn query(
    model: crate::cli::Model,
    api_key: String,
    prompt: &str,
    logger: &crate::logger::Logger,
    log_prefix: &str,
) -> Result<String, crate::app_error::AppError>;
