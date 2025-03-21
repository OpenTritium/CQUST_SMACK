use anyhow::Result;
use reqwest::{
    Client, Url,
    header::{COOKIE, HeaderMap, HeaderValue},
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, str::FromStr, time::Duration};
use tokio::time::{self, Instant};
use tracing::{info, warn};

use crate::config;

pub type Options = Vec<String>;
pub type Topics = HashMap<TopicType, Vec<String>>;

pub struct Page {
    headers: HeaderMap,
    client: Client,
}

pub enum Question {
    MultiChoice(u8),
    MultiSelect(u8),
    TrueOrFalse(u8),
}

#[derive(Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Clone)]
pub enum TopicType {
    MultiChoice(String),
    MultiSelect(String),
    TrueOrFalse(String),
}

impl From<TopicType> for String {
    fn from(value: TopicType) -> Self {
        match value {
            TopicType::MultiChoice(s) | TopicType::MultiSelect(s) | TopicType::TrueOrFalse(s) => s,
        }
    }
}
impl Question {
    /// 选出题目文本和选项文本
    fn select(&self, dom: &Html) -> Result<(String, Vec<String>)> {
        let gen_id = |t, n, count| {
            const PREFIX: &str = "Mydatalist__ctl0_Mydatalist";
            let question = format!("#{PREFIX}{t}__ctl{n}_tm");
            let choices = (0..count)
                .into_iter()
                .map(|i| {
                    let id = format!("{PREFIX}{t}__ctl{n}_xz_{i}");
                    format!("label[for={id}]")
                })
                .collect::<Vec<String>>();
            (question, choices)
        };
        let select_with = |selector_str: &str| {
            let selector = Selector::parse(&selector_str).unwrap();
            dom.select(&selector)
                .next()
                .map(|element| element.text().collect::<String>())
                .ok_or_else(|| anyhow::anyhow!("Element not found"))
        };
        let (q_id, c_ids) = match self {
            Question::MultiChoice(n) => gen_id(1, n, 4),
            Question::MultiSelect(n) => gen_id(2, n, 4),
            Question::TrueOrFalse(n) => gen_id(3, n, 2),
        };
        let q_text = select_with(&q_id)?;
        let cs_text = c_ids
            .into_iter()
            .filter_map(|c_id| select_with(&c_id).ok())
            .collect::<Vec<String>>();
        Ok((q_text, cs_text))
    }
}

impl Page {
    pub fn new() -> Self {
        Self {
            headers: {
                let mut header = HeaderMap::new();
                header.insert(COOKIE, HeaderValue::from_str(&config().cookie).unwrap());
                header
            },
            client: Client::new(),
        }
    }

    pub async fn fetch(&self) -> Result<Topics> {
        let dom = self.client.get(Url::from_str("http://xgbd.cqust.edu.cn:866/txxm/dkkt.aspx?xq=2024-2025-2&nd=2024&km=tm_ks_jy&tmfl=“行为养成·智慧指南”知识竞答").unwrap()).headers(self.headers.clone()).send().await?.text().await?;
        let dom = Html::parse_document(&dom);
        let mut paper: Topics = Default::default();
        for i in 0..4 {
            let q = Question::MultiChoice(i);
            let (q, c) = q.select(&dom)?;
            paper.insert(TopicType::MultiChoice(q), c);
        }
        for i in 0..20 {
            let q = Question::MultiSelect(i);
            let (q, c) = q.select(&dom)?;
            paper.insert(TopicType::MultiSelect(q), c);
        }
        for i in 0..15 {
            let q = Question::TrueOrFalse(i);
            let (q, c) = q.select(&dom)?;
            paper.insert(TopicType::TrueOrFalse(q), c);
        }
        Ok(paper)
    }

    pub async fn caching() {
        let timeout = Duration::from_hours(4);
        time::timeout(timeout, async {
            let sb = sled::open("./cache").unwrap();
            let mut interval = time::interval(Duration::from_secs(3));
            let mut prev_len = sb.len();
            let mut last_change = Instant::now();
            loop {
                interval.tick().await;
                let data = Page::new().fetch().await.unwrap();
                data.iter().for_each(|(q, cs)| {
                    sb.insert(
                        json!(q).to_string().as_bytes(),
                        json!(cs).to_string().as_bytes(),
                    )
                    .unwrap();
                });
                let current_len = sb.len();
                info!("已经爬下来 {current_len} 条");
                if current_len != prev_len {
                    prev_len = current_len;
                    last_change = Instant::now();
                } else if Instant::now().duration_since(last_change) > Duration::from_mins(3) {
                    warn!("No changes for 3 minutes, exiting...");
                    break;
                }
            }
        })
        .await
        .unwrap();
    }
}
