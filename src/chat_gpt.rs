// chat_gpt.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

const CHAT_GPT_API_KEY: &str = "";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequestBody {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize)]
pub struct AssistantResponse {
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

pub async fn send_to_assistant(question: &str) -> Result<AssistantResponse, String> {
    let client = Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let body = ChatRequestBody {
        model: "gpt-4".to_string(),
        messages: vec![ChatMessage {
            role: "user".into(),
            content: question.into(),
        }],
    };

    let res = client
        .post(url)
        .bearer_auth(CHAT_GPT_API_KEY)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Erro na requisição: {:?}", e))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Erro da API OpenAI: {}", err_text));
    }

    let parsed: ChatResponse = res
        .json()
        .await
        .map_err(|e| format!("Erro ao interpretar resposta JSON: {:?}", e))?;

    let answer = parsed
        .choices
        .get(0)
        .map(|c| AssistantResponse {
            content: c.message.content.clone(),
        })
        .unwrap_or_else(|| AssistantResponse {
            content: "Sem resposta.".into(),
        });

    Ok(answer)
}
