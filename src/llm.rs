use crate::error::AppError;
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct ApiResponse {
    candidates: Vec<ApiCandidate>,
}

#[derive(Deserialize)]
struct ApiCandidate {
    content: ApiContent,
}

#[derive(Deserialize)]
struct ApiContent {
    parts: Vec<ApiPart>,
}

#[derive(Deserialize)]
struct ApiPart {
    text: String,
}

/// Holds the full JSON response and the extracted text from the LLM.
pub struct GeminiResponse {
    pub full_json: String,
    pub extracted_text: String,
}

pub async fn call_gemini(api_key: &str, prompt: &str) -> Result<GeminiResponse> {
    let client = Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-pro-latest:generateContent?key={}",
        api_key
    );

    let body = json!({
      "contents": [{
        "parts":[{
          "text": prompt
        }]
      }],
      "generationConfig": {
        "temperature": 0.2,
        "topK": 1,
        "topP": 1.0,
      }
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to send request to Gemini API")?;

    if !response.status().is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(anyhow!(AppError::Api(format!("API Error: {}", error_body))));
    }

    // Read the full response body as text to preserve it for logging.
    let response_text = response
        .text()
        .await
        .context("Failed to read Gemini API response body")?;

    // Deserialize from the text, not the original response object.
    let api_response: ApiResponse = serde_json::from_str(&response_text)
        .context("Failed to deserialize Gemini API response from text")?;

    let extracted_text = api_response
        .candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| {
            anyhow!(AppError::Api(
                "API response was empty or had an invalid structure".to_string()
            ))
        })?;

    Ok(GeminiResponse {
        full_json: response_text,
        extracted_text,
    })
}
