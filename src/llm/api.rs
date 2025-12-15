use crate::app_error::AppError;
use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const GPT_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const GPT_MODEL_NAME: &str = "gpt-5.2-thinking";

// Internal error classification used for robust retry handling.
#[derive(Debug)]
pub(crate) enum QueryError {
    Http {
        status: StatusCode,
        body: String,
        retry_after: Option<Duration>,
    },
    Transport {
        is_connect: bool,
        is_timeout: bool,
        message: String,
    },
    InvalidJson {
        body: String,
        parse_error: String,
    },
}

pub(crate) struct GeminiClient {
    client: Client,
    api_key: String,
    model_name: &'static str,
    api_url: String,
}

impl GeminiClient {
    pub(crate) fn new(api_key: String, model_name: &'static str) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            .build()
            .unwrap_or_else(|_| Client::new());
        let api_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model_name}:generateContent"
        );
        Self {
            client,
            api_key,
            model_name,
            api_url,
        }
    }

    // Single attempt. No retries here; higher-level logic decides retries.
    async fn query_once(&self, request_body: &Value) -> Result<Value, QueryError> {
        let url = &self.api_url;

        let resp_res = self
            .client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(request_body)
            .send()
            .await;

        let resp = match resp_res {
            Ok(r) => r,
            Err(e) => {
                return Err(QueryError::Transport {
                    is_connect: e.is_connect(),
                    is_timeout: e.is_timeout(),
                    message: censor_api_key_in_error_string(e, &self.api_key),
                });
            }
        };

        handle_response_to_json(resp, &self.api_key).await
    }
}

pub(crate) struct GptClient {
    client: Client,
    api_key: String,
}

impl GptClient {
    pub(crate) fn new(api_key: String) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client, api_key }
    }

    // Single attempt with optional idempotency key for safe retries.
    async fn query_once(
        &self,
        request_body: &Value,
        idempotency_key: &str,
    ) -> Result<Value, QueryError> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        if !idempotency_key.is_empty() {
            if let Ok(hv) = HeaderValue::from_str(idempotency_key) {
                headers.insert("Idempotency-Key", hv);
            }
        }

        let resp_res = self
            .client
            .post(GPT_API_URL)
            .bearer_auth(&self.api_key)
            .headers(headers)
            .json(request_body)
            .send()
            .await;

        let resp = match resp_res {
            Ok(r) => r,
            Err(e) => {
                return Err(QueryError::Transport {
                    is_connect: e.is_connect(),
                    is_timeout: e.is_timeout(),
                    message: censor_api_key_in_error_string(e, &self.api_key),
                });
            }
        };

        handle_response_to_json(resp, &self.api_key).await
    }
}

pub(crate) enum LlmApiClient {
    Gemini(GeminiClient),
    Gpt(GptClient),
}

impl LlmApiClient {
    pub(crate) fn get_model_name(&self) -> &'static str {
        match self {
            LlmApiClient::Gemini(c) => c.model_name,
            LlmApiClient::Gpt(_) => GPT_MODEL_NAME,
        }
    }

    pub(crate) fn get_url(&self) -> &str {
        match self {
            LlmApiClient::Gemini(c) => &c.api_url,
            LlmApiClient::Gpt(_) => GPT_API_URL,
        }
    }

    pub(crate) fn build_request_body(&self, prompt: &str) -> Value {
        match self {
            LlmApiClient::Gemini(_) => json!({
                "contents": [{
                    "parts": [{ "text": prompt }]
                }],
                "generationConfig": {
                    "temperature": 0.7
                }
            }),
            LlmApiClient::Gpt(_) => json!({
                "model": GPT_MODEL_NAME,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
            }),
        }
    }

    // Robust retries with idempotency when supported
    pub(crate) async fn query_with_retries(
        &self,
        request_body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<Value, AppError> {
        let policy = RetryPolicy::for_model(self);
        let mut attempt: u32 = 1;

        loop {
            let result = match self {
                LlmApiClient::Gemini(c) => c.query_once(request_body).await,
                LlmApiClient::Gpt(c) => {
                    c.query_once(request_body, idempotency_key.unwrap_or_default())
                        .await
                }
            };

            match result {
                Ok(v) => return Ok(v),
                Err(e) => {
                    if attempt >= policy.max_attempts || !policy.is_retryable(self, &e) {
                        // Map QueryError -> AppError with full body/message preserved.
                        return Err(map_query_error_to_app_error(e));
                    }

                    // Respect Retry-After if provided (HTTP errors only).
                    let mut delay = policy.backoff_delay(attempt);
                    if let QueryError::Http {
                        retry_after: Some(ra),
                        ..
                    } = e
                    {
                        if ra > delay {
                            delay = ra;
                        }
                    }
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    pub(crate) fn extract_text_from_response(&self, response: &Value) -> Result<String, AppError> {
        match self {
            LlmApiClient::Gemini(_) => extract_text_from_gemini_response(response),
            LlmApiClient::Gpt(_) => extract_text_from_gpt_response(response),
        }
    }

    pub(crate) fn supports_idempotency(&self) -> bool {
        matches!(self, LlmApiClient::Gpt(_))
    }
}

pub(crate) trait LlmApi: Send + Sync {
    fn get_model_name(&self) -> &'static str;
    fn get_url(&self) -> &str;
    fn build_request_body(&self, prompt: &str) -> Value;
    fn query_with_retries<'a>(
        &'a self,
        request_body: &'a Value,
        idempotency_key: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Value, AppError>> + Send + 'a>>;
    fn extract_text_from_response(&self, response: &Value) -> Result<String, AppError>;
    fn supports_idempotency(&self) -> bool;
}

impl LlmApi for LlmApiClient {
    fn get_model_name(&self) -> &'static str {
        LlmApiClient::get_model_name(self)
    }

    fn get_url(&self) -> &str {
        LlmApiClient::get_url(self)
    }

    fn build_request_body(&self, prompt: &str) -> Value {
        LlmApiClient::build_request_body(self, prompt)
    }

    fn query_with_retries<'a>(
        &'a self,
        request_body: &'a Value,
        idempotency_key: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Value, AppError>> + Send + 'a>> {
        Box::pin(LlmApiClient::query_with_retries(
            self,
            request_body,
            idempotency_key,
        ))
    }

    fn extract_text_from_response(&self, response: &Value) -> Result<String, AppError> {
        LlmApiClient::extract_text_from_response(self, response)
    }

    fn supports_idempotency(&self) -> bool {
        LlmApiClient::supports_idempotency(self)
    }
}

// Retry policy tuned to avoid double-inference while maximizing delivery.
pub(crate) struct RetryPolicy {
    pub(crate) max_attempts: u32,
    pub(crate) base_delay: Duration,
    pub(crate) max_delay: Duration,
}

impl RetryPolicy {
    pub(crate) fn for_model(model: &LlmApiClient) -> Self {
        match model {
            // Conservative on Gemini: avoid retries on ambiguous timeouts to prevent double inference.
            LlmApiClient::Gemini(_) => RetryPolicy {
                max_attempts: 4,
                base_delay: Duration::from_millis(400),
                max_delay: Duration::from_secs(8),
            },
            // Aggressive retries for GPT with idempotency key.
            LlmApiClient::Gpt(_) => RetryPolicy {
                max_attempts: 6,
                base_delay: Duration::from_millis(300),
                max_delay: Duration::from_secs(10),
            },
        }
    }

    pub(crate) fn is_retryable(&self, model: &LlmApiClient, err: &QueryError) -> bool {
        match err {
            QueryError::Transport {
                is_connect,
                is_timeout,
                ..
            } => match model {
                LlmApiClient::Gemini(_) => {
                    // Retry only if the connection was not established (safe to retry).
                    *is_connect
                }
                LlmApiClient::Gpt(_) => {
                    // With idempotency key, both connect and timeout are safe to retry.
                    *is_connect || *is_timeout
                }
            },
            QueryError::Http { status, .. } => {
                // Retry on common transient codes.
                matches!(status.as_u16(), 408 | 429 | 500 | 502 | 503 | 504)
            }
            // If we already received a 2xx with invalid JSON, don't retry to avoid double inference.
            QueryError::InvalidJson { .. } => false,
        }
    }

    pub(crate) fn backoff_delay(&self, attempt: u32) -> Duration {
        // Exponential backoff with jitter derived from system time nanos (no RNG dependency).
        let shift = attempt.saturating_sub(1).min(10);
        let exp = 1u32 << shift;
        let base = self.base_delay.saturating_mul(exp);
        let capped = if base > self.max_delay {
            self.max_delay
        } else {
            base
        };
        let jitter = jitter_duration(self.base_delay);
        capped + jitter
    }
}

fn jitter_duration(base: Duration) -> Duration {
    // 0..(base/2)
    let nanos_now: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u128)
        .unwrap_or(0);

    let half = base.as_nanos() / 2;
    if half == 0 {
        return Duration::from_millis(0);
    }
    let bound = half.min(u128::from(u64::MAX));
    let jitter_nanos = nanos_now % bound;
    Duration::from_nanos(jitter_nanos as u64)
}

fn map_query_error_to_app_error(e: QueryError) -> AppError {
    match e {
        QueryError::Http { status, body, .. } => {
            AppError::Network(format!("HTTP {status} with body:\n{body}"))
        }
        QueryError::Transport { message, .. } => AppError::Network(message),
        QueryError::InvalidJson { body, parse_error } => AppError::Network(format!(
            "Invalid JSON in success response: {parse_error}; raw body:\n{body}"
        )),
    }
}

pub(crate) fn censor_api_key(text: &str, api_key: &str) -> String {
    if api_key.is_empty() {
        return text.to_string();
    }
    // Only censor things that look like keys. Very short strings are unlikely to be keys.
    let censored_key = if api_key.len() > 8 {
        format!("...{}", &api_key[api_key.len() - 4..])
    } else {
        "...".to_string()
    };
    text.replace(api_key, &censored_key)
}

fn censor_api_key_in_error_string(e: reqwest::Error, api_key: &str) -> String {
    censor_api_key(&e.to_string(), api_key)
}

async fn handle_response_to_json(
    resp: reqwest::Response,
    api_key: &str,
) -> Result<Value, QueryError> {
    let status = resp.status();
    let retry_after = parse_retry_after(resp.headers());

    let text = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            return Err(QueryError::Transport {
                is_connect: e.is_connect(),
                is_timeout: e.is_timeout(),
                message: censor_api_key_in_error_string(e, api_key),
            })
        }
    };

    if !status.is_success() {
        return Err(QueryError::Http {
            status,
            body: censor_api_key(&text, api_key),
            retry_after,
        });
    }

    match serde_json::from_str::<Value>(&text) {
        Ok(v) => Ok(v),
        Err(e) => Err(QueryError::InvalidJson {
            body: text,
            parse_error: e.to_string(),
        }),
    }
}

fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    if let Some(val) = headers.get(RETRY_AFTER) {
        if let Ok(s) = val.to_str() {
            if let Ok(secs) = s.trim().parse::<u64>() {
                return Some(Duration::from_secs(secs));
            }
        }
    }
    None
}

pub(crate) fn extract_text_from_gemini_response(response: &Value) -> Result<String, AppError> {
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

pub(crate) fn extract_text_from_gpt_response(response: &Value) -> Result<String, AppError> {
    let content = response
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            AppError::ResponseParsing("Could not find 'content' in GPT response JSON.".to_string())
        })?;
    Ok(content.to_string())
}
