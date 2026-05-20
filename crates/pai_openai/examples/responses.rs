#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = pai_openai::OpenAIClient::from_env()?;
    let request = pai_openai::ResponseCreateRequest::with_items(
        pai_openai::GPT_5_4_MINI,
        [pai_openai::InputItem::Message(
            pai_openai::InputMessage::user(
                "Tell me a three sentence bedtime story about a unicorn.",
            ),
        )],
    );

    let response = client.create_response(&request).await?;
    println!("{}", response.output_text());

    Ok(())
}
