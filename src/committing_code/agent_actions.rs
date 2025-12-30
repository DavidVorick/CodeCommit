use super::build_runner;
use crate::app_error::{AppError, BuildFailure};
use crate::cli::Model;
use crate::llm;
use crate::logger::Logger;
use std::future::Future;
use std::pin::Pin;

pub(crate) trait AgentActions {
    fn query_llm<'a>(
        &'a self,
        model: Model,
        api_key: String,
        prompt: String,
        logger: &'a Logger,
        log_prefix: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>>;

    fn run_build(&self) -> Result<String, BuildFailure>;
}

pub(crate) struct RealAgentActions;

impl AgentActions for RealAgentActions {
    fn query_llm<'a>(
        &'a self,
        model: Model,
        api_key: String,
        prompt: String,
        logger: &'a Logger,
        log_prefix: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
        Box::pin(async move { llm::query(model, api_key, &prompt, logger, &log_prefix).await })
    }

    fn run_build(&self) -> Result<String, BuildFailure> {
        build_runner::run()
    }
}
