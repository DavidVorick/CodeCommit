pub mod api;

#[cfg(test)]
mod api_test;
#[cfg(test)]
mod mod_test;

use crate::app_error::AppError;
use crate::cli::Model;
use crate::logger::Logger;
use api::{LlmApi, LlmApiClient};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static IDEMPOTENCY_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn generate_request_id(prefix: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let n = IDEMPOTENCY_COUNTER.fetch_add(1, Ordering::Relaxed);
    if prefix.is_empty() {
        format!("req-{now}-{n}")
    } else {
        format!("{prefix}-{now}-{n}")
    }
}

pub async fn query(
    model: Model,
    api_key: String,
    prompt: &str,
    logger: &Logger,
    log_prefix: &str,
) -> Result<String, AppError> {
    let api_client = match model {
        Model::Gemini3Pro => {
            LlmApiClient::Gemini(api::GeminiClient::new(api_key, "gemini-3-pro-preview"))
        }
        Model::Gemini2_5Pro => {
            LlmApiClient::Gemini(api::GeminiClient::new(api_key, "gemini-2.5-pro"))
        }
        Model::Gpt5 => LlmApiClient::Gpt(api::GptClient::new(api_key)),
    };
    query_internal(&api_client, prompt, logger, log_prefix).await
}

async fn query_internal(
    api_client: &dyn LlmApi,
    prompt: &str,
    logger: &Logger,
    log_prefix: &str,
) -> Result<String, AppError> {
    logger.log_text(&format!("{log_prefix}-query.txt"), prompt)?;

    let request_body = api_client.build_request_body(prompt);
    let url = api_client.get_url();
    let request_id = generate_request_id("llm");

    let log_body = json!({
        "url": url,
        "body": &request_body,
        "requestId": request_id
    });
    logger.log_json(&format!("{log_prefix}-query.json"), &log_body)?;

    let start_time = Instant::now();
    let idempotency_key = if api_client.supports_idempotency() {
        Some(request_id.as_str())
    } else {
        None
    };

    let response_result = api_client
        .query_with_retries(&request_body, idempotency_key)
        .await;
    let duration = start_time.elapsed();

    println!(
        "LLM call to {} took {:.3}s",
        api_client.get_model_name(),
        duration.as_secs_f64()
    );

    let response_json = match response_result {
        Ok(json) => json,
        Err(e) => {
            let error_json =
                json!({ "error": e.to_string(), "totalResponseTime": duration.as_millis() });
            logger.log_json(&format!("{log_prefix}-response.json"), &error_json)?;
            let error_msg = format!("ERROR\n{e}");
            logger.log_text(&format!("{log_prefix}-response.txt"), &error_msg)?;
            return Err(e);
        }
    };

    let mut logged_response = response_json.clone();
    if let Some(obj) = logged_response.as_object_mut() {
        obj.insert("totalResponseTime".to_string(), json!(duration.as_millis()));
    } else {
        logged_response = json!({
            "response_payload": logged_response,
            "totalResponseTime": duration.as_millis(),
        });
    }
    logger.log_json(&format!("{log_prefix}-response.json"), &logged_response)?;

    let response_text = match api_client.extract_text_from_response(&response_json) {
        Ok(text) => text,
        Err(e) => {
            let error_msg = format!("ERROR\n{e}");
            logger.log_text(&format!("{log_prefix}-response.txt"), &error_msg)?;
            return Err(e);
        }
    };
    logger.log_text(&format!("{log_prefix}-response.txt"), &response_text)?;

    Ok(response_text)
}
