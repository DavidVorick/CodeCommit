use crate::app_error::AppError;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

const GEMINI_API_URL_BASE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-pro-latest:generateContent";

#[derive(Serialize)]
struct GeminiRequest<'a> {
    contents: Vec<Content<'a>>,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

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
        let url = format!("{}?key={}", GEMINI_API_URL_BASE, self.api_key);

        let request_body = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
        };

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        let response_json: Value = response.json().await?;
        Ok(response_json)
    }
}

pub fn extract_text_from_response(response: &Value) -> Result<String, AppError> {
    let text = response["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| {
            AppError::ResponseParsing("Could not find text in Gemini response JSON.".to_string())
        })?;
    Ok(text.to_string())
}
