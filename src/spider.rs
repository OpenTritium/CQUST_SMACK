use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use reqwest::{
    Client, Url,
    header::{COOKIE, HeaderMap, HeaderValue},
};
use scraper::{Html, Selector};
use serde_json::json;
use sled::Db;

type Topic = (String, Vec<String>);
type Topics = HashMap<String, Vec<String>>;

pub struct Page {
    headers: HeaderMap,
    client: Client,
}

pub enum Question {
    MultiChoice(u8),
    MultiSelect(u8),
    TrueOrFalse(u8),
}

impl Question {
    fn select(&self, dom: &Html) -> Result<Topic> {
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
                header.insert(COOKIE, HeaderValue::from_str("ASP.NET_SessionId=q23nsyj50oxm5tpxbcoxqwyo; .ASPXAUTH=C9A211E8281471661EAA8412937CF0A2A8ACFDB9640005A556C552E03A9B0B4A30AA3D5B79443C85824F5F6BDB992A70555C209FF19E5D9AD368C1EB74F065604F8246DCFE92CA13D1D21CD94045FFCA801ABD2BD6FB17298A693DFA7CF47070E0609A049018523919E842905D959D059260850BEC261402A4ABAF102A5A0752BD0AABA3BA6C97ABC127442FB6BF7E2D6861AA578D69E67A152A64F1E95507AE37F63333E164A20F639DA0EB5F9C7F37290FFF94A3EE3414E0FFC5409874F4E2E3B0951F47BCE81570CFE465A5775E2275DE14D40C1F37C038502626DC03C04DD16BAEF5").unwrap());
                header
            },
            client: Client::new(),
        }
    }
    pub async fn fetch(&self) -> Result<Topics> {
        let dom = self.client.get(Url::from_str("http://xgbd.cqust.edu.cn:866/txxm/dkkt.aspx?xq=2024-2025-2&nd=2024&km=tm_ks_jy&tmfl=“行为养成·智慧指南”知识竞答").unwrap()).headers(self.headers.clone()).send().await?.text().await?;
        let dom = Html::parse_document(&dom);
        let mut paper: HashMap<String, Vec<String>> = Default::default();
        for i in 0..4 {
            let q = Question::MultiChoice(i);
            let (q, c) = q.select(&dom)?;
            paper.insert(q, c);
        }
        for i in 0..20 {
            let q = Question::MultiSelect(i);
            let (q, c) = q.select(&dom)?;
            paper.insert(q, c);
        }
        for i in 0..15 {
            let q = Question::TrueOrFalse(i);
            let (q, c) = q.select(&dom)?;
            paper.insert(q, c);
        }
        println!("{:?}", paper);
        Ok(paper)
    }
    pub fn cache(db: &Db, data: &Topics) {
        data.iter().for_each(|(q, cs)| {
            db.insert(q.as_bytes(), json!(cs).to_string().as_bytes()).unwrap();
        });
    }
}
