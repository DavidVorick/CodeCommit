use super::Logger;
use crate::app_error::AppError;
use crate::llm_api::LlmApiClient;
use serde_json::{json, Value};
use std::time::Instant;

pub async fn call_llm_and_log(
    llm_client: &LlmApiClient,
    request_body: &Value,
    logger: &Logger,
    log_prefix: &str,
) -> Result<String, AppError> {
    let start_time = Instant::now();
    let response_result = llm_client.query(request_body).await;
    let duration = start_time.elapsed();

    let response_json = match response_result {
        Ok(json) => json,
        Err(e) => {
            let error_json =
                json!({ "error": e.to_string(), "totalResponseTime": duration.as_millis() });
            logger.log_response_json(log_prefix, &error_json)?;
            let error_msg = format!("ERROR\n{e}");
            logger.log_response_text(log_prefix, &error_msg)?;
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
    logger.log_response_json(log_prefix, &logged_response)?;

    let response_text = match llm_client.extract_text_from_response(&response_json) {
        Ok(text) => text,
        Err(e) => {
            let error_msg = format!("ERROR\n{e}");
            logger.log_response_text(log_prefix, &error_msg)?;
            return Err(e);
        }
    };
    logger.log_response_text(log_prefix, &response_text)?;

    Ok(response_text)
}
