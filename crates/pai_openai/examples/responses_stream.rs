use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = pai_openai::OpenAIClient::from_env()?;
    let request = pai_openai::ResponseCreateRequest::text(
        pai_openai::GPT_5_4_MINI,
        "Tell me a three sentence bedtime story about a unicorn.",
    );

    let mut stream = client.stream_response(&request).await?;

    while let Some(delta) = stream.next_text_delta().await? {
        print!("{delta}");
        io::stdout().flush()?;
    }

    println!();

    Ok(())
}
