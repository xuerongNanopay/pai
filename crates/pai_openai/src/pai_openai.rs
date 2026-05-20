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

    pub async fn stream_response(
        &self,
        request: &ResponseCreateRequest,
    ) -> Result<ResponseStream, OpenAIError> {
        let mut request = request.clone();
        request.stream = Some(true);

        let response = self
            .http
            .post(self.url("responses"))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        Self::parse_stream_response(response).await
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

    async fn parse_stream_response(
        response: reqwest::Response,
    ) -> Result<ResponseStream, OpenAIError> {
        let status = response.status();

        if status.is_success() {
            Ok(ResponseStream::new(response))
        } else {
            let body = response.text().await?;
            Err(OpenAIError::Api { status, body })
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }
}

pub type Client = OpenAIClient;

pub struct ResponseStream {
    response: reqwest::Response,
    buffer: String,
    done: bool,
}

impl ResponseStream {
    fn new(response: reqwest::Response) -> Self {
        Self {
            response,
            buffer: String::new(),
            done: false,
        }
    }

    pub async fn next_event(&mut self) -> Result<Option<ResponseStreamEvent>, OpenAIError> {
        loop {
            if let Some(raw_event) = self.next_buffered_event() {
                match parse_sse_event(&raw_event)? {
                    ParsedSseEvent::Event(event) => return Ok(Some(event)),
                    ParsedSseEvent::Done => {
                        self.done = true;
                        return Ok(None);
                    }
                    ParsedSseEvent::Empty => continue,
                }
            }

            if self.done {
                return Ok(None);
            }

            match self.response.chunk().await? {
                Some(chunk) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&chunk));
                }
                None => {
                    self.done = true;
                    let raw_event = self.buffer.trim().to_string();
                    self.buffer.clear();

                    if raw_event.is_empty() {
                        return Ok(None);
                    }

                    match parse_sse_event(&raw_event)? {
                        ParsedSseEvent::Event(event) => return Ok(Some(event)),
                        ParsedSseEvent::Done | ParsedSseEvent::Empty => return Ok(None),
                    }
                }
            }
        }
    }

    pub async fn next_text_delta(&mut self) -> Result<Option<String>, OpenAIError> {
        while let Some(event) = self.next_event().await? {
            if let Some(delta) = event.text_delta() {
                return Ok(Some(delta.to_string()));
            }
        }

        Ok(None)
    }

    fn next_buffered_event(&mut self) -> Option<String> {
        let (index, marker_len) = find_sse_boundary(&self.buffer)?;
        let raw_event = self.buffer[..index].to_string();
        self.buffer.drain(..index + marker_len);
        Some(raw_event)
    }
}

enum ParsedSseEvent {
    Event(ResponseStreamEvent),
    Done,
    Empty,
}

fn find_sse_boundary(buffer: &str) -> Option<(usize, usize)> {
    buffer
        .find("\r\n\r\n")
        .map(|index| (index, 4))
        .or_else(|| buffer.find("\n\n").map(|index| (index, 2)))
}

fn parse_sse_event(raw_event: &str) -> Result<ParsedSseEvent, OpenAIError> {
    let mut event = None;
    let mut id = None;
    let mut data = Vec::new();

    for line in raw_event.lines() {
        let line = line.trim_end_matches('\r');

        if line.is_empty() || line.starts_with(':') {
            continue;
        }

        let (field, value) = line.split_once(':').unwrap_or((line, ""));
        let value = value.strip_prefix(' ').unwrap_or(value);

        match field {
            "event" => event = Some(value.to_string()),
            "id" => id = Some(value.to_string()),
            "data" => data.push(value),
            _ => {}
        }
    }

    if data.is_empty() {
        return Ok(ParsedSseEvent::Empty);
    }

    let data = data.join("\n");

    if data == "[DONE]" {
        return Ok(ParsedSseEvent::Done);
    }

    let mut stream_event: ResponseStreamEvent = serde_json::from_str(&data)?;
    stream_event.id = stream_event.id.or(id);
    stream_event.event = stream_event.event.or(event);

    Ok(ParsedSseEvent::Event(stream_event))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sse_response_event() {
        let parsed = parse_sse_event(
            "event: response.output_text.delta\n\
             data: {\"type\":\"response.output_text.delta\",\"delta\":\"Hi\",\"sequence_number\":1}",
        )
        .expect("event should parse");

        let ParsedSseEvent::Event(event) = parsed else {
            panic!("expected parsed event");
        };

        assert_eq!(event.event.as_deref(), Some("response.output_text.delta"));
        assert_eq!(event.text_delta(), Some("Hi"));
    }

    #[test]
    fn parses_done_sse_event() {
        let parsed = parse_sse_event("data: [DONE]").expect("done should parse");

        assert!(matches!(parsed, ParsedSseEvent::Done));
    }

    #[test]
    fn finds_crlf_and_lf_boundaries() {
        assert_eq!(find_sse_boundary("data: {}\r\n\r\n"), Some((8, 4)));
        assert_eq!(find_sse_boundary("data: {}\n\n"), Some((8, 2)));
    }
}
