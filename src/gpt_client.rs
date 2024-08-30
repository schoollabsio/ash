use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GptMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct GptRequest<'a> {
    model: &'a str,
    messages: Vec<GptMessage<'a>>,
    max_tokens: u16,
}

#[derive(Deserialize)]
struct GptResponse {
    choices: Vec<GptChoice>,
}

#[derive(Deserialize)]
struct GptChoice {
    message: GptMessageResponse
}

#[derive(Deserialize)]
struct GptMessageResponse {
    role: String,
    content: String,
}

pub struct GptClient {
    client: Client,
    api_key: String,
    model: String,
    system_prompt: String,
}

impl GptClient {
    pub fn new(api_key: String, model: String, system_prompt: String) -> Self {
        GptClient {
            client: Client::new(),
            api_key,
            model,
            system_prompt,
        }
    }

    pub async fn send(&self, prompt: &str) -> Result<String, String> {
        let gpt_request = GptRequest {
            model: &self.model,
            messages: vec![
                GptMessage { role: "system", content: &self.system_prompt },
                GptMessage { role: "user", content: prompt },
            ],
            max_tokens: 100,
        };

        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&gpt_request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Request failed with status code: {}. Response text: {}", response.status(), response.text().await.unwrap_or_else(|_| String::from("Failed to get response text."))));
        }

        let body = response.text().await.map_err((|e| format!("Failed to read response body: {}", e)))?;

        let gpt_response: GptResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(choice) = gpt_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No choices returned by GPT API".to_string())
        }
    }
}


impl Clone for GptClient {
    fn clone(&self) -> GptClient {
        GptClient {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            system_prompt: self.system_prompt.clone(),
        }
    }
}
