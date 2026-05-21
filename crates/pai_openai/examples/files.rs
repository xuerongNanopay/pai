#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = pai_openai::OpenAIClient::from_env()?;

    let upload = pai_openai::FileUploadRequest::with_bytes(
        "pai-openai-example.jsonl",
        br#"{"messages":[{"role":"user","content":"Hello from pai_openai."}]}"#,
        pai_openai::FilePurpose::UserData,
    );

    let file = client.upload_file(upload).await?;
    println!("created file:");
    println!("{}", serde_json::to_string_pretty(&file)?);

    let retrieved = client.retrieve_file(&file.id).await?;
    println!("retrieved file:");
    println!("{}", serde_json::to_string_pretty(&retrieved)?);

    let content = client.retrieve_file_content(&file.id).await?;
    println!("downloaded {} bytes", content.len());

    let deleted = client.delete_file(&file.id).await?;
    println!("deleted file:");
    println!("{}", serde_json::to_string_pretty(&deleted)?);

    Ok(())
}
