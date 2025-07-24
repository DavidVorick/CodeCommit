//! implement: An agentic coding workflow implementation loop.

pub mod config;
pub mod error;
pub mod execution;
pub mod llm;
pub mod logging;
pub mod parsing;
pub mod state;

use anyhow::Result;
use config::Config;
use logging::Logger;
use state::{History, Interaction};
use std::process::ExitCode;

const MAX_ITERATIONS: usize = 8;

/// The main entry point for the implementation logic.
pub async fn run() -> Result<ExitCode> {
    // 1. (Setup) Initialize configuration, logger, and state.
    let config = Config::load().await?;
    let logger = Logger::new()?;
    let mut history = History::new(config.clone());

    println!(
        "Implement loop started. Log directory: {}",
        logger.log_dir().display()
    );

    // 2. (Setup) Perform the initial LLM call.
    let initial_prompt = history.build_prompt();
    logger.log_prompt(1, &initial_prompt).await?;

    println!("Round 1: Calling LLM...");
    let mut llm_response_text = llm::call_gemini(&config.api_key, &initial_prompt).await?;
    logger.log_llm_response(1, &llm_response_text).await?;

    // 3. Start the implementation loop.
    for i in 1..=MAX_ITERATIONS {
        println!("\n--- Iteration {}/{} ---", i, MAX_ITERATIONS);

        // 4. Parse the LLM response.
        println!("Parsing LLM response...");
        let parsed_result = parsing::parse(&llm_response_text);

        let interaction;
        match parsed_result {
            Err(parse_error) => {
                println!("Error parsing LLM response: {}", parse_error);
                interaction = Interaction {
                    debug_thoughts: "No debug thoughts due to parsing error.".to_string(),
                    file_changes: "No file changes suggested due to parsing error.".to_string(),
                    build_output: parse_error.to_string(),
                };
            }
            Ok(parsed) => {
                for thought in &parsed.user_thoughts {
                    println!("\nLLM Thought:\n---\n{}\n---\n", thought);
                    logger.log_user_thought(thought).await?;
                }

                if let Some(msg) = parsed.success_message {
                    println!(
                        "LLM signaled completion, but build is not passing. Treating as failed fix."
                    );
                    interaction = Interaction {
                        debug_thoughts: format!("LLM indicated completion with message: {}", msg),
                        file_changes: "No file changes were suggested.".to_string(),
                        build_output:
                            "Build failed after LLM signaled completion. No changes were applied."
                                .to_string(),
                    };
                } else {
                    println!("Applying {} file changes...", parsed.file_changes.len());
                    for (path, content) in &parsed.file_changes {
                        if let Some(parent) = path.parent() {
                            tokio::fs::create_dir_all(parent).await?;
                        }
                        tokio::fs::write(path, content).await?;
                        println!("  - Wrote {}", path.display());
                    }

                    println!("Running build script...");
                    let build_result = execution::run_build_script().await?;
                    logger.log_build_output(i, &build_result.output).await?;

                    if build_result.success {
                        println!("\n✅ Build succeeded on iteration {}!", i);
                        return Ok(ExitCode::SUCCESS);
                    }
                    println!("Build failed. Capturing output for next LLM query.");

                    interaction = Interaction {
                        debug_thoughts: parsed.debug_thoughts.join("\n\n"),
                        file_changes: parsing::format_file_changes_for_prompt(
                            &parsed.file_changes,
                        ),
                        build_output: build_result.output,
                    };
                }
            }
        }

        history.add_interaction(interaction);

        if i == MAX_ITERATIONS {
            break;
        }

        // 5. Ask LLM to fix the code.
        let next_prompt = history.build_prompt();
        logger.log_prompt(i + 1, &next_prompt).await?;

        println!("Round {}: Calling LLM with build feedback...", i + 1);
        llm_response_text = llm::call_gemini(&config.api_key, &next_prompt).await?;
        logger.log_llm_response(i + 1, &llm_response_text).await?;
    }

    eprintln!(
        "\n❌ Failed to produce a passing build after {} iterations.",
        MAX_ITERATIONS
    );
    Ok(ExitCode::from(1))
}