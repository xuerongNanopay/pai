use std::{env, error::Error, fmt};

pub mod responses;

pub use responses::*;

pub const GPT_5_5_PRO: &str = "gpt-5.5-pro";
pub const GPT_5_5: &str = "gpt-5.5";
pub const GPT_5_4_PRO: &str = "gpt-5.4-pro";
pub const GPT_5_4: &str = "gpt-5.4";
pub const GPT_5_4_MINI: &str = "gpt-5.4-mini";
pub const GPT_5_4_NANO: &str = "gpt-5.4-nano";

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Clone)]
pub struct OpenAIClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl OpenAIClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    pub fn from_env() -> Result<Self, OpenAIError> {
        let api_key = env::var("OPENAI_API_KEY").map_err(|_| OpenAIError::MissingApiKey)?;
        Ok(Self::new(api_key))
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    pub async fn create_response(
        &self,
        request: &ResponseCreateRequest,
    ) -> Result<Response, OpenAIError> {
        self.post_json("responses", request).await
    }

    pub async fn retrieve_response(&self, response_id: &str) -> Result<Response, OpenAIError> {
        self.get_json(&format!("responses/{response_id}")).await
    }

    pub async fn delete_response(&self, response_id: &str) -> Result<DeleteResponse, OpenAIError> {
        self.delete_json(&format!("responses/{response_id}")).await
    }

    pub async fn cancel_response(&self, response_id: &str) -> Result<Response, OpenAIError> {
        self.post_json(&format!("responses/{response_id}/cancel"), &())
            .await
    }

    pub async fn compact_response(
        &self,
        request: &CompactResponseRequest,
    ) -> Result<Response, OpenAIError> {
        self.post_json("responses/compact", request).await
    }

    pub async fn list_input_items(
        &self,
        response_id: &str,
        query: &ListInputItemsQuery,
    ) -> Result<InputItemList, OpenAIError> {
        let response = self
            .http
            .get(self.url(&format!("responses/{response_id}/input_items")))
            .bearer_auth(&self.api_key)
            .query(query)
            .send()
            .await?;

        Self::parse_response(response).await
    }

    pub async fn count_input_tokens(
        &self,
        request: &InputTokenCountRequest,
    ) -> Result<InputTokenCount, OpenAIError> {
        self.post_json("responses/input_tokens", request).await
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, OpenAIError>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .http
            .get(self.url(path))
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        Self::parse_response(response).await
    }

    async fn post_json<T, B>(&self, path: &str, body: &B) -> Result<T, OpenAIError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let response = self
            .http
            .post(self.url(path))
            .bearer_auth(&self.api_key)
            .json(body)
            .send()
            .await?;

        Self::parse_response(response).await
    }

    async fn delete_json<T>(&self, path: &str) -> Result<T, OpenAIError>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .http
            .delete(self.url(path))
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        Self::parse_response(response).await
    }

    async fn parse_response<T>(response: reqwest::Response) -> Result<T, OpenAIError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).map_err(OpenAIError::Json)
        } else {
            Err(OpenAIError::Api { status, body })
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }
}

pub type Client = OpenAIClient;

#[derive(Debug)]
pub enum OpenAIError {
    MissingApiKey,
    Request(reqwest::Error),
    Json(serde_json::Error),
    Api {
        status: reqwest::StatusCode,
        body: String,
    },
}

impl fmt::Display for OpenAIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => write!(f, "OPENAI_API_KEY is not set"),
            Self::Request(error) => write!(f, "request failed: {error}"),
            Self::Json(error) => write!(f, "failed to parse response JSON: {error}"),
            Self::Api { status, body } => write!(f, "OpenAI API returned {status}: {body}"),
        }
    }
}

impl Error for OpenAIError {}

impl From<reqwest::Error> for OpenAIError {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

impl From<serde_json::Error> for OpenAIError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
