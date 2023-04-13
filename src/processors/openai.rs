use crate::executor::Processor;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

pub struct Openai {
    lines: Vec<String>,
    api_key: String,
    api_base_url: String,
}

impl Openai {
    pub fn new(api_base_url: String, api_key: String) -> Self {
        Self {
            lines: vec![],
            api_base_url,
            api_key,
        }
    }
}

#[async_trait]
impl Processor for Openai {
    fn process(&mut self, input: String) -> anyhow::Result<String> {
        self.lines.push(input.clone());
        Ok(input)
    }

    async fn flush(&mut self) -> anyhow::Result<()> {
        eprintln!(
            "\n+==================================[ OpenAI ]==================================+"
        );

        let body = json!({
          "model": "gpt-3.5-turbo",
          "messages": [
              {
                  "role": "user",
                  "content": format!("I am a developer working in a terminal. I'm trying to run a command but I get the following error message. In a first paragraph explain what the issue is and why it is happening. In a second paragraph explain how to fix the issue. In a third paragraph suggest a command that might help fixing the issue.\n{}", self.lines.join("\n")),
              },
          ]
        });

        let url = format!("{}/v1/chat/completions", self.api_base_url);
        let resp = reqwest::Client::new()
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        eprintln!(
            "|                                                                              |"
        );
        let resp: Response = resp.json().await?;
        for line in textwrap::wrap(&resp.choices[0].message.content, 76) {
            eprintln!("| {:<76} |", line);
        }
        eprintln!(
            "|                                                                              |"
        );
        eprintln!(
            "+==============================================================================+\n"
        );

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub usage: Usage,
    pub choices: Vec<Choice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Usage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: i64,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Choice {
    pub message: Message,
    #[serde(rename = "finish_reason")]
    pub finish_reason: String,
    pub index: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    pub role: String,
    pub content: String,
}
