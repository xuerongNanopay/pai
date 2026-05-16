// struct Context {
//     system_prompt: String,
//     messages: Vec<Message>,
//     tools: Option<Vec<Tool>>,
// }

// struct Tool {
//     name: String,
//     description: String,
// }

// enum Provider {
//     Openai,
//     Deepseek,
// }

// enum StopReason {
//     Stop,
//     Length,
//     ToolUse,
//     Error,
//     Aborted,
// }

// enum Message {
//     User {
//         content: String,
//         timestamp: u64,
//     },
//     Ai {
//         content: String,
//         model: String,
//         provider: String,
//         response_id: Option<String>,
//         error_message: Option<String>,
//         stop_reason: String,
//         timestamp: u64,
//     },
// }

// pub async fn prompt()