use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u16,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

#[derive(Debug, Clone)]
pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let key = match api_key {
            Some(key) => key,
            None => env::var("OPENAI_API_KEY")?,
        };

        Ok(OpenAIClient {
            client: Client::new(),
            api_key: key,
        })
    }

    pub async fn generate_text(
        &self,
        prompt: &str,
        max_tokens: u16,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = OpenAIRequest {
            model: "gpt-4o-mini".to_string(), // TODO: make this configurable
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let openai_response: OpenAIResponse = response.json().await?;
            Ok(openai_response
                .choices
                .into_iter()
                .map(|c| c.message.content)
                .collect())
        } else {
            let error_text = response.text().await?;
            Err(Box::from(error_text))
        }
    }
}
