use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use tokio::io::AsyncRead;
use tokio::io::{AsyncBufReadExt, BufReader};

// TODO: optimization, do not redirect stdout/stderr if there are no processors to run

#[async_trait]
trait Processor: Send + Sync {
    fn process(&mut self, line: String) -> anyhow::Result<String>;

    async fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Pipeline {
    out_processors: Vec<Box<dyn Processor + Send + Sync>>,
    err_processors: Vec<Box<dyn Processor + Send + Sync>>,
}

impl Pipeline {
    pub fn new(prefix: String, openai_api_key: Option<String>) -> Self {
        let mut p = Pipeline::default();

        // Prepare out processors
        p.out_processors.push(Box::new(Prefixer {
            prefix: prefix.clone(),
        }));

        // Prepare err processors
        if let Some(openai_api_key) = openai_api_key {
            p.err_processors.push(Box::new(OpenaiProcessor {
                lines: vec![],
                api_key: openai_api_key,
            }));
        }
        p.err_processors.push(Box::new(Prefixer { prefix }));

        p
    }

    pub async fn process(
        &mut self,
        out: impl AsyncRead + Unpin,
        err: impl AsyncRead + Unpin,
    ) -> anyhow::Result<()> {
        let mut out = BufReader::new(out).lines();
        let mut err = BufReader::new(err).lines();

        // TODO: read both streams in parallel

        while let Some(mut line) = out.next_line().await? {
            for processor in &mut self.out_processors {
                line = processor.process(line)?;
            }
            println!("{}", line);
        }

        for processor in self.out_processors.iter_mut().rev() {
            processor.flush().await?;
        }

        while let Some(mut line) = err.next_line().await? {
            for processor in &mut self.err_processors {
                line = processor.process(line)?;
            }
            eprintln!("{}", line);
        }

        for processor in &mut self.err_processors {
            processor.flush().await?;
        }

        Ok(())
    }
}

struct Prefixer {
    prefix: String,
}

impl Processor for Prefixer {
    fn process(&mut self, input: String) -> anyhow::Result<String> {
        Ok(format!("{} {}", self.prefix, input))
    }
}

struct OpenaiProcessor {
    lines: Vec<String>,
    api_key: String,
}

#[async_trait]
impl Processor for OpenaiProcessor {
    fn process(&mut self, input: String) -> anyhow::Result<String> {
        self.lines.push(input.clone());
        Ok(input)
    }

    async fn flush(&mut self) -> anyhow::Result<()> {
        eprintln!(
            "\n+=================================[ ChatGPT ]==================================+"
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

        let resp = reqwest::Client::new()
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        eprintln!(
            "|                                                                              |"
        );
        let resp: OpenaiResponse = resp.json().await?;
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
pub struct OpenaiResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub usage: Usage,
    pub choices: Vec<Choice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: i64,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub message: Message,
    #[serde(rename = "finish_reason")]
    pub finish_reason: String,
    pub index: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: String,
    pub content: String,
}
