use super::TopicType;
use super::config;
use anyhow::Result;
use anyhow::anyhow;
use reqwest::Client;
use reqwest::multipart;
use reqwest::multipart::Part;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::read_to_string;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::error;
use tracing::info;
use tracing::warn;

type IndexResult = Vec<u8>;

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub system_fingerprint: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: RecvMessage,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecvMessage {
    pub role: String,
    pub content: String,
    pub reasoning_content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<SendMessage>,
    temperature: f32,
    max_tokens: u32,
}

impl ChatRequest {
    fn new(messages: Vec<SendMessage>) -> Self {
        Self {
            model: "deepseek-ai/DeepSeek-R1".to_string(),
            messages,
            temperature: 0.2,
            max_tokens: 512,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SendMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BatchConfig {
    custom_id: String,
    method: String,
    url: String,
    body: ChatRequest,
}

pub struct Llm {
    client: Client,
    prompt_msg: SendMessage,
}

impl Llm {
    pub fn new() -> Self {
        Self {
            prompt_msg: SendMessage {
                role: "system".to_string(),
                content: read_to_string("./prompt.json").unwrap(),
            },
            client: Client::new(),
        }
    }

    pub async fn init(&self) -> Result<()> {
        let path = Path::new(&config().document_path);
        let mut buf: Vec<u8> = vec![];
        File::open(path).await?.read(&mut buf).await?;
        let doc = String::from_utf8_lossy(&buf).to_string();
        let doc = json!(doc).to_string();
        let batch = {
            let mut rst = json!(BatchConfig {
                custom_id: "cqust_smack".to_string(),
                method: "POST".to_string(),
                url: "/v1/chat/completions".to_string(),
                body: ChatRequest::new(vec![
                    SendMessage {
                        role: "system".to_string(),
                        content: format!("你会仔细阅读以下 Markdown 文档 ，以应对用户随即抛出的问题，文档内容是:{doc}")
                    }
                ])
            })
            .to_string();
            rst.push('\n'); // 坑
            rst
        };
        let mp = multipart::Form::new().text("purpose", "batch").part(
            "file",
            Part::bytes(batch.clone().into_bytes()).file_name("batch.jsonl"),
        );
        let resp = self
            .client
            .post("https://api.siliconflow.cn/v1/files")
            .bearer_auth(config().llm_api_key.clone())
            .multipart(mp)
            .send()
            .await?;
        if resp.status().is_success() {
            info!("料子喂上了");
        } else {
            warn!("料子上传失败");
            return Err(anyhow!("post failed"));
        }
        Ok(())
    }

    pub async fn ask(&self, q: TopicType, o: Vec<String>) -> Result<IndexResult> {
        let msg = SendMessage {
            role: "user".to_string(),
            content: format!("{},{}", json!(q).to_string(), json!(o).to_string()),
        };
        let resp = self
            .client
            .post("https://api.siliconflow.cn/v1/chat/completions")
            .bearer_auth(config().llm_api_key.clone())
            .json(&ChatRequest::new(vec![msg, self.prompt_msg.clone()]))
            .send()
            .await?;
        let content = resp
            .json::<ChatResponse>()
            .await
            .inspect_err(|e| error!("{:?}", e))
            .and_then(|resp| Ok(resp.choices[0].message.content.clone()))?;
        let rst = serde_json::from_str::<IndexResult>(&content)?;
        Ok(rst)
    }
}
