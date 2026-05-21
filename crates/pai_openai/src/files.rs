use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileObject {
    pub id: String,
    pub object: String,
    pub bytes: u64,
    pub created_at: i64,
    pub filename: String,
    pub purpose: FilePurpose,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<FileStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_details: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileList {
    pub object: String,
    pub data: Vec<FileObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FileListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<FileListOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<FilePurpose>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FileListOrder {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUploadRequest {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub purpose: FilePurpose,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<FileExpiresAfter>,
}

impl FileUploadRequest {
    pub fn with_bytes(
        filename: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
        purpose: FilePurpose,
    ) -> Self {
        Self {
            filename: filename.into(),
            bytes: bytes.into(),
            purpose,
            expires_after: None,
        }
    }

    pub async fn with_path(path: impl AsRef<Path>, purpose: FilePurpose) -> std::io::Result<Self> {
        let path = path.as_ref();
        let bytes = tokio::fs::read(path).await?;
        let filename = path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or("file")
            .to_string();

        Ok(Self::with_bytes(filename, bytes, purpose))
    }

    pub fn with_expires_after(mut self, expires_after: FileExpiresAfter) -> Self {
        self.expires_after = Some(expires_after);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileExpiresAfter {
    pub anchor: FileExpiresAfterAnchor,
    pub seconds: u32,
}

impl FileExpiresAfter {
    pub fn created_at(seconds: u32) -> Self {
        Self {
            anchor: FileExpiresAfterAnchor::CreatedAt,
            seconds,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FileExpiresAfterAnchor {
    CreatedAt,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileDeleted {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Uploaded,
    Processed,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FilePurpose {
    #[serde(rename = "assistants")]
    Assistants,
    #[serde(rename = "assistants_output")]
    AssistantsOutput,
    #[serde(rename = "batch")]
    Batch,
    #[serde(rename = "batch_output")]
    BatchOutput,
    #[serde(rename = "fine-tune")]
    FineTune,
    #[serde(rename = "fine-tune-results")]
    FineTuneResults,
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "user_data")]
    UserData,
    #[serde(rename = "evals")]
    Evals,
}

impl FilePurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assistants => "assistants",
            Self::AssistantsOutput => "assistants_output",
            Self::Batch => "batch",
            Self::BatchOutput => "batch_output",
            Self::FineTune => "fine-tune",
            Self::FineTuneResults => "fine-tune-results",
            Self::Vision => "vision",
            Self::UserData => "user_data",
            Self::Evals => "evals",
        }
    }
}

impl AsRef<str> for FilePurpose {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_upload_purpose_values() {
        assert_eq!(
            serde_json::to_value(FilePurpose::FineTune).expect("purpose should serialize"),
            serde_json::json!("fine-tune")
        );
        assert_eq!(FilePurpose::UserData.as_str(), "user_data");
    }

    #[test]
    fn deserializes_file_object() {
        let file: FileObject = serde_json::from_value(serde_json::json!({
            "id": "file-abc123",
            "object": "file",
            "bytes": 120000,
            "created_at": 1677610602,
            "expires_at": 1677614202,
            "filename": "mydata.jsonl",
            "purpose": "fine-tune"
        }))
        .expect("file should deserialize");

        assert_eq!(file.id, "file-abc123");
        assert!(matches!(file.purpose, FilePurpose::FineTune));
    }

    #[test]
    fn serializes_list_query() {
        let query = FileListQuery {
            after: Some("file-abc123".to_string()),
            limit: Some(20),
            order: Some(FileListOrder::Desc),
            purpose: Some(FilePurpose::UserData),
        };

        assert_eq!(
            serde_json::to_value(query).expect("query should serialize"),
            serde_json::json!({
                "after": "file-abc123",
                "limit": 20,
                "order": "desc",
                "purpose": "user_data"
            })
        );
    }
}
