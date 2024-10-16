use crate::errors::{OpenAIError, SyntaxError};
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
    pub fn new(api_key: Option<String>) -> Result<Self, SyntaxError> {
        let key = match api_key {
            Some(key) => key,
            None => match env::var("OPENAI_API_KEY") {
                Ok(key) => key,
                Err(_) => {
                    return Err(SyntaxError::InvalidOpenAIKey(
                        "OPENAI_API_KEY not set".to_string(),
                    ));
                }
            },
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
    ) -> Result<String, OpenAIError> {
        let request = OpenAIRequest {
            model: "gpt-4o-mini".to_string(), // TODO: make this configurable
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens,
        };

        let response = match self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                error!("Error during API call: {:?}", e);
                return Err(OpenAIError::NetworkError(e.to_string()));
            }
        };

        let status = response.status();
        debug!("Response status: {}", status);
        if status.is_success() {
            let openai_response = match response.json::<OpenAIResponse>().await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Error deserializing response: {:?}", e);
                    return Err(OpenAIError::DeserializationError(e.to_string()));
                }
            };
            let content = openai_response
                .choices
                .into_iter()
                .map(|c| c.message.content)
                .collect();
            Ok(content)
        } else {
            let error_text = match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    error!("Error reading error response: {:?}", e);
                    return Err(OpenAIError::DeserializationError(e.to_string()));
                }
            };
            error!("API error: {}", error_text);
            Err(OpenAIError::APIError(error_text))
        }
    }
}
