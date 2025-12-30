use crate::app_error::AppError;
use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const GPT_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const GPT_MODEL_NAME: &str = "gpt-5.2";
const GEMINI_INTERACTIONS_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/interactions";

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
    polling_interval: Duration,
}

impl GeminiClient {
    pub(crate) fn new(api_key: String, model_name: &'static str) -> Self {
        Self::create(
            api_key,
            model_name,
            GEMINI_INTERACTIONS_URL.to_string(),
            Duration::from_secs(2),
        )
    }

    #[cfg(test)]
    pub(crate) fn new_test(api_key: String, model_name: &'static str, api_url: String) -> Self {
        Self::create(api_key, model_name, api_url, Duration::from_millis(10))
    }

    fn create(
        api_key: String,
        model_name: &'static str,
        api_url: String,
        polling_interval: Duration,
    ) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            api_key,
            model_name,
            api_url,
            polling_interval,
        }
    }

    async fn query_once(&self, request_body: &Value) -> Result<Value, QueryError> {
        let mut resp = self.post_interaction(request_body).await?;
        loop {
            let status = resp.get("status").and_then(|s| s.as_str()).unwrap_or("");
            if status == "completed" || status == "failed" || status == "cancelled" {
                return Ok(resp);
            }
            let id =
                resp.get("id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| QueryError::InvalidJson {
                        body: resp.to_string(),
                        parse_error: "Missing 'id' in non-terminal Interaction response"
                            .to_string(),
                    })?;
            tokio::time::sleep(self.polling_interval).await;
            resp = self.get_interaction(id).await?;
        }
    }

    async fn post_interaction(&self, body: &Value) -> Result<Value, QueryError> {
        let resp_res = self
            .client
            .post(&self.api_url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await;
        self.handle_response(resp_res).await
    }

    async fn get_interaction(&self, id: &str) -> Result<Value, QueryError> {
        let url = format!("{}/{}", self.api_url, id);
        let resp_res = self
            .client
            .get(&url)
            .header("x-goog-api-key", &self.api_key)
            .send()
            .await;
        self.handle_response(resp_res).await
    }

    async fn handle_response(
        &self,
        resp_res: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<Value, QueryError> {
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
    api_url: String,
}

impl GptClient {
    pub(crate) fn new(api_key: String) -> Self {
        Self::create(api_key, GPT_API_URL.to_string())
    }

    #[cfg(test)]
    pub(crate) fn new_test(api_key: String, api_url: String) -> Self {
        Self::create(api_key, api_url)
    }

    fn create(api_key: String, api_url: String) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            api_key,
            api_url,
        }
    }

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
            .post(&self.api_url)
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
            LlmApiClient::Gpt(c) => &c.api_url,
        }
    }

    pub(crate) fn build_request_body(&self, prompt: &str) -> Value {
        match self {
            LlmApiClient::Gemini(c) => json!({
                "model": c.model_name,
                "input": prompt,
            }),
            LlmApiClient::Gpt(_) => json!({
                "model": GPT_MODEL_NAME,
                "messages": [
                    { "role": "user", "content": prompt }
                ],
            }),
        }
    }

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
                        return Err(map_query_error_to_app_error(e));
                    }
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

pub(crate) struct RetryPolicy {
    pub(crate) max_attempts: u32,
    pub(crate) base_delay: Duration,
    pub(crate) max_delay: Duration,
}

impl RetryPolicy {
    pub(crate) fn for_model(model: &LlmApiClient) -> Self {
        match model {
            LlmApiClient::Gemini(_) => RetryPolicy {
                max_attempts: 4,
                base_delay: Duration::from_millis(400),
                max_delay: Duration::from_secs(8),
            },
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
                LlmApiClient::Gemini(_) => *is_connect || *is_timeout,
                LlmApiClient::Gpt(_) => *is_connect || *is_timeout,
            },
            QueryError::Http { status, .. } => {
                matches!(status.as_u16(), 408 | 429 | 500 | 502 | 503 | 504)
            }
            QueryError::InvalidJson { .. } => false,
        }
    }
    pub(crate) fn backoff_delay(&self, attempt: u32) -> Duration {
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
    let outputs = response
        .get("outputs")
        .and_then(|o| o.as_array())
        .ok_or_else(|| {
            AppError::ResponseParsing(
                "Could not find 'outputs' array in Gemini Interaction response.".to_string(),
            )
        })?;
    let text_segments: Vec<String> = outputs
        .iter()
        .filter(|part| part.get("type").and_then(|t| t.as_str()) == Some("text"))
        .filter_map(|part| part.get("text"))
        .filter_map(|text_val| text_val.as_str())
        .map(|s| s.to_string())
        .collect();
    if text_segments.is_empty() {
        return Err(AppError::ResponseParsing(
            "Found 'outputs' array, but it contained no valid text segments.".to_string(),
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
