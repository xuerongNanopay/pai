use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponsesRequest {
    /// Model ID used to generate the response, like gpt-4o or o3.
    pub model: String,
    // /// A list of one or many input items to the model, containing different content types.
    // pub input: Vec<InputItem>
}