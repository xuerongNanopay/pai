use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResponseCreateRequest {
    /// Model ID used to generate the response, like gpt-5 or o3.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Text, image, or file inputs to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResponseInput>,
    /// A system or developer message inserted into the model's context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

impl ResponseCreateRequest {
    pub fn with_text(model: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            model: Some(model.into()),
            input: Some(ResponseInput::Text(input.into())),
            ..Self::default()
        }
    }

    pub fn with_items<I>(model: impl Into<String>, items: I) -> Self
    where
        I: IntoIterator<Item = InputItem>,
    {
        Self {
            model: Some(model.into()),
            input: Some(ResponseInput::Items(items.into_iter().collect())),
            ..Self::default()
        }
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ResponseInput {
    Text(String),
    Items(Vec<InputItem>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum InputItem {
    Message(InputMessage),
    ItemReference(ItemReference),
    FunctionCallOutput(FunctionCallOutput),
    CustomToolCallOutput(CustomToolCallOutput),
    Raw(Value),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputMessage {
    pub role: MessageRole,
    pub content: InputMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl InputMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: InputMessageContent::Text(text.into()),
            status: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Developer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum InputMessageContent {
    Text(String),
    Parts(Vec<InputContent>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputContent {
    InputText {
        text: String,
    },
    InputImage {
        detail: ImageDetail,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
    },
    InputFile {
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemReference {
    pub id: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCallOutput {
    #[serde(rename = "type")]
    pub item_type: FunctionCallOutputType,
    pub call_id: String,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FunctionCallOutputType {
    FunctionCallOutput,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomToolCallOutput {
    #[serde(rename = "type")]
    pub item_type: CustomToolCallOutputType,
    pub call_id: String,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CustomToolCallOutputType {
    CustomToolCallOutput,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSummary {
    Auto,
    Concise,
    Detailed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<TextFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextFormat {
    Text,
    JsonObject,
    JsonSchema {
        name: String,
        schema: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Truncation {
    Auto,
    Disabled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ToolChoice {
    Mode(ToolChoiceMode),
    Function { name: String, r#type: String },
    Raw(Value),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceMode {
    Auto,
    None,
    Required,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tool {
    Function {
        name: String,
        parameters: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    WebSearchPreview,
    FileSearch {
        vector_store_ids: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_num_results: Option<u32>,
    },
    CodeInterpreter {
        container: Value,
    },
    ImageGeneration,
    #[serde(untagged)]
    Raw(Value),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub status: ResponseStatus,
    #[serde(default)]
    pub output: Vec<ResponseOutputItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(default)]
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponseUsage>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Response {
    pub fn output_text(&self) -> String {
        self.output
            .iter()
            .filter_map(ResponseOutputItem::as_message)
            .flat_map(|message| message.content.iter())
            .filter_map(|content| match content {
                OutputContent::OutputText { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Completed,
    Failed,
    InProgress,
    Cancelled,
    Queued,
    Incomplete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseOutputItem {
    Message {
        id: String,
        status: String,
        role: MessageRole,
        content: Vec<OutputContent>,
    },
    FunctionCall {
        id: String,
        call_id: String,
        name: String,
        arguments: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    WebSearchCall {
        id: String,
        status: String,
        #[serde(flatten)]
        extra: HashMap<String, Value>,
    },
    FileSearchCall {
        id: String,
        status: String,
        #[serde(flatten)]
        extra: HashMap<String, Value>,
    },
    Reasoning {
        id: String,
        #[serde(default)]
        summary: Vec<Value>,
        #[serde(flatten)]
        extra: HashMap<String, Value>,
    },
    #[serde(untagged)]
    Raw(Value),
}

impl ResponseOutputItem {
    pub fn as_message(&self) -> Option<ResponseMessage<'_>> {
        match self {
            Self::Message { content, .. } => Some(ResponseMessage { content }),
            _ => None,
        }
    }
}

pub struct ResponseMessage<'a> {
    pub content: &'a [OutputContent],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputContent {
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<Value>,
        #[serde(default)]
        logprobs: Vec<Value>,
    },
    Refusal {
        refusal: String,
    },
    #[serde(untagged)]
    Raw(Value),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens_details: Option<TokenDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<TokenDetails>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenDetails {
    #[serde(flatten)]
    pub values: HashMap<String, u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiErrorObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteResponse {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputItemList {
    pub object: String,
    pub data: Vec<InputItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListInputItemsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ListOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ListOrder {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputTokenCount {
    pub object: String,
    pub input_tokens: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseStreamEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

impl ResponseStreamEvent {
    pub fn text_delta(&self) -> Option<&str> {
        if self.event_type == "response.output_text.delta" {
            self.data.get("delta").and_then(Value::as_str)
        } else {
            None
        }
    }

    pub fn response(&self) -> Option<Response> {
        self.data
            .get("response")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
    }
}

pub type InputTokenCountRequest = ResponseCreateRequest;
pub type CompactResponseRequest = ResponseCreateRequest;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_simple_text_request() {
        let request = ResponseCreateRequest::with_text("gpt-5", "Tell me a joke.");
        let json = serde_json::to_value(request).expect("request should serialize");

        assert_eq!(json["model"], "gpt-5");
        assert_eq!(json["input"], "Tell me a joke.");
        assert!(json.get("tools").is_none());
    }

    #[test]
    fn serializes_items_request() {
        let request = ResponseCreateRequest::with_items(
            "gpt-5",
            [InputItem::Message(InputMessage::user("Tell me a joke."))],
        );
        let json = serde_json::to_value(request).expect("request should serialize");

        assert_eq!(json["model"], "gpt-5");
        assert_eq!(json["input"][0]["role"], "user");
        assert_eq!(json["input"][0]["content"], "Tell me a joke.");
    }

    #[test]
    fn deserializes_response_and_extracts_output_text() {
        let json = serde_json::json!({
            "id": "resp_123",
            "object": "response",
            "created_at": 1741476542,
            "status": "completed",
            "model": "gpt-5",
            "output": [
                {
                    "type": "message",
                    "id": "msg_123",
                    "status": "completed",
                    "role": "assistant",
                    "content": [
                        {
                            "type": "output_text",
                            "text": "Hello",
                            "annotations": []
                        },
                        {
                            "type": "output_text",
                            "text": " world",
                            "annotations": []
                        }
                    ]
                }
            ],
            "metadata": {}
        });

        let response: Response = serde_json::from_value(json).expect("response should parse");

        assert_eq!(response.output_text(), "Hello world");
    }

    #[test]
    fn extracts_text_delta_from_stream_event() {
        let event: ResponseStreamEvent = serde_json::from_value(serde_json::json!({
            "type": "response.output_text.delta",
            "delta": "Hello",
            "output_index": 0,
            "content_index": 0,
            "sequence_number": 1
        }))
        .expect("stream event should parse");

        assert_eq!(event.text_delta(), Some("Hello"));
    }
}
