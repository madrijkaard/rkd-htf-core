use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::credential::get_credentials;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize)]
struct AssistantCreationRequest {
    model: String,
    instructions: String,
}

#[derive(Debug, Deserialize)]
struct Assistant {
    id: String,
}

#[derive(Debug, Deserialize)]
struct Thread {
    id: String,
}

#[derive(Debug, Serialize)]
struct MessageCreateRequest {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct Run {
    id: String,
    status: String,
}

#[derive(Debug, Serialize)]
pub struct AssistantResponse {
    pub content: String,
}

pub async fn send_to_assistant() -> Result<AssistantResponse, String> {
    const PERGUNTA: &str = "Como funciona a Ethereum?";

    let credentials = get_credentials();
    let api_key = &credentials.open_ai_key;
    let client = Client::new();

    // 1. Cria o assistente
    let response = client
        .post("https://api.openai.com/v1/assistants")
        .bearer_auth(api_key)
        .header("OpenAI-Beta", "assistants=v2")
        .json(&AssistantCreationRequest {
            model: "gpt-4-1106-preview".to_string(),
            instructions: "Você é um especialista em blockchain.".to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Erro criando assistente: {:?}", e))?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    println!("\n🔍 Resposta da OpenAI (status: {}):\n{}", status, text);

    if !status.is_success() {
        return Err(format!("Erro ao criar assistente: {}", text));
    }

    let assistant: Assistant = serde_json::from_str(&text)
        .map_err(|e| format!("Erro no JSON do assistente: {:?}", e))?;

    // 2. Cria uma thread
    let thread: Thread = client
        .post("https://api.openai.com/v1/threads")
        .bearer_auth(api_key)
        .header("OpenAI-Beta", "assistants=v2")
        .send()
        .await
        .map_err(|e| format!("Erro criando thread: {:?}", e))?
        .json()
        .await
        .map_err(|e| format!("Erro no JSON da thread: {:?}", e))?;

    // 3. Adiciona a pergunta
    client
        .post(format!("https://api.openai.com/v1/threads/{}/messages", thread.id))
        .bearer_auth(api_key)
        .header("OpenAI-Beta", "assistants=v2")
        .json(&MessageCreateRequest {
            role: "user".to_string(),
            content: PERGUNTA.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Erro adicionando mensagem: {:?}", e))?;

    // 4. Executa o assistente
    let run: Run = client
        .post(format!("https://api.openai.com/v1/threads/{}/runs", thread.id))
        .bearer_auth(api_key)
        .header("OpenAI-Beta", "assistants=v2")
        .json(&serde_json::json!({ "assistant_id": assistant.id }))
        .send()
        .await
        .map_err(|e| format!("Erro iniciando run: {:?}", e))?
        .json()
        .await
        .map_err(|e| format!("Erro no JSON da run: {:?}", e))?;

    // 5. Aguarda a execução (até 30s)
    let mut status = run.status;
    let mut attempts = 0;
    while status != "completed" && attempts < 30 {
        println!("⌛ Esperando execução... tentativa {}", attempts + 1);
        sleep(Duration::from_secs(1)).await;
        let r: Run = client
            .get(format!("https://api.openai.com/v1/threads/{}/runs/{}", thread.id, run.id))
            .bearer_auth(api_key)
            .header("OpenAI-Beta", "assistants=v2")
            .send()
            .await
            .map_err(|e| format!("Erro ao verificar status da run: {:?}", e))?
            .json()
            .await
            .map_err(|e| format!("Erro no JSON da run polling: {:?}", e))?;
        status = r.status.clone();
        attempts += 1;
    }

    if status != "completed" {
        return Err("Timeout esperando resposta do assistente.".into());
    }

    // 6. Obtém a resposta final
    let messages: serde_json::Value = client
        .get(format!("https://api.openai.com/v1/threads/{}/messages", thread.id))
        .bearer_auth(api_key)
        .header("OpenAI-Beta", "assistants=v2")
        .send()
        .await
        .map_err(|e| format!("Erro ao obter mensagens: {:?}", e))?
        .json()
        .await
        .map_err(|e| format!("Erro no JSON das mensagens: {:?}", e))?;

    if let Some(answer) = messages["data"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|msg| msg["content"].as_array())
        .and_then(|content| content.first())
        .and_then(|item| item["text"]["value"].as_str())
    {
        Ok(AssistantResponse {
            content: answer.to_string(),
        })
    } else {
        Err("Não foi possível extrair resposta.".into())
    }
}
