# API Signatures

```rust
pub async fn build_codebase_context(
    next_agent_full_prompt: &str,
    config: &crate::config::Config,
    logger: &crate::logger::Logger,
    log_prefix: &str,
) -> Result<String, crate::app_error::AppError>;
```
