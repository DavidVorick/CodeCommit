pub mod api;

#[cfg(test)]
mod api_test;

use crate::app_error::AppError;
use crate::cli::Model;
use crate::logger::Logger;
use api::LlmApiClient;
use serde_json::json;
use std::time::Instant;

pub async fn query(
    model: Model,
    api_key: String,
    prompt: &str,
    logger: &Logger,
    log_prefix: &str,
) -> Result<String, AppError> {
    let api_client = match model {
        Model::Gemini2_5Pro => LlmApiClient::Gemini(api::GeminiClient::new(api_key)),
        Model::Gpt5 => LlmApiClient::Gpt(api::GptClient::new(api_key)),
    };

    logger.log_text(&format!("{log_prefix}-query.txt"), prompt)?;
    let request_body = api_client.build_request_body(prompt);
    let url = api_client.get_url();
    let log_body = json!({
        "url": url,
        "body": &request_body
    });
    logger.log_json(&format!("{log_prefix}-query.json"), &log_body)?;

    let start_time = Instant::now();
    let response_result = api_client.query(&request_body).await;
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
