use crate::app_error::AppError;
use reqwest::Client;
use serde_json::{json, Value};

const GEMINI_API_URL_BASE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent";

pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn query(&self, prompt: &str) -> Result<Value, AppError> {
        let url = GEMINI_API_URL_BASE;

        let request_body = json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
            "generationConfig": {
                "temperature": 0.7
            }
        });

        let response = self
            .client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Network(censor_api_key_in_error_string(e, &self.api_key)))?
            .error_for_status()
            .map_err(|e| AppError::Network(censor_api_key_in_error_string(e, &self.api_key)))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| AppError::Network(censor_api_key_in_error_string(e, &self.api_key)))?;
        Ok(response_json)
    }
}

fn censor_api_key_in_error_string(e: reqwest::Error, api_key: &str) -> String {
    let error_string = e.to_string();
    if api_key.is_empty() {
        return error_string;
    }

    let censored_key = if api_key.len() > 2 {
        format!("...{}", &api_key[api_key.len() - 2..])
    } else {
        "...".to_string()
    };

    error_string.replace(api_key, &censored_key)
}

pub fn extract_text_from_response(response: &Value) -> Result<String, AppError> {
    let parts_array = response
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .ok_or_else(|| {
            AppError::ResponseParsing(
                "Could not find 'parts' array in Gemini response JSON.".to_string(),
            )
        })?;

    let text_segments: Vec<String> = parts_array
        .iter()
        .filter_map(|part| part.get("text"))
        .filter_map(|text_val| text_val.as_str())
        .map(|s| s.to_string())
        .collect();

    if text_segments.is_empty() {
        return Err(AppError::ResponseParsing(
            "Found 'parts' array, but it contained no valid text segments.".to_string(),
        ));
    }

    Ok(text_segments.join(""))
}
