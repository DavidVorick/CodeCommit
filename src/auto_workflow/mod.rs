mod discovery;
mod executor;
mod file_updater;
mod graph;
mod prompt_builder;
mod prompts;
mod types;

#[cfg(test)]
mod discovery_test;
#[cfg(test)]
mod enforcement_test;
#[cfg(test)]
mod executor_test;
#[cfg(test)]
mod file_updater_test;
#[cfg(test)]
mod graph_test;
#[cfg(test)]
mod phase1_test;
#[cfg(test)]
mod prompt_builder_test;

use crate::app_error::AppError;
use crate::cli::CliArgs;
use crate::config::Config;
use crate::logger::Logger;
use std::path::Path;

pub async fn run(logger: &Logger, cli_args: CliArgs) -> Result<(), AppError> {
    let config = Config::load(&cli_args)?;

    loop {
        let root = Path::new(".");
        let task_opt = discovery::find_next_task(root)?;

        let task = match task_opt {
            Some(t) => t,
            None => {
                println!("No more tasks to process in the specification review stages.");
                break;
            }
        };

        let result = executor::execute_task(root, &task, &config, logger).await?;

        match result {
            executor::ExecutionResult::Success => {
                // Continue to next task
                continue;
            }
            executor::ExecutionResult::ChangesAttempted => {
                println!("Changes attempted. Retrying task...");
                let retry_result = executor::execute_task(root, &task, &config, logger).await?;
                match retry_result {
                    executor::ExecutionResult::Success => {
                        println!("Retry successful. Continuing workflow.");
                        continue;
                    }
                    _ => {
                        println!("Retry did not result in success. Stopping.");
                        break;
                    }
                }
            }
            executor::ExecutionResult::ChangesRequested => {
                println!("Changes requested by Auto Workflow. Stopping.");
                break;
            }
            executor::ExecutionResult::Failure => {
                println!("Auto Workflow failed. Stopping.");
                return Err(AppError::FileUpdate(
                    "Auto Workflow task failed.".to_string(),
                ));
            }
        }
    }

    Ok(())
}
