#![feature(slice_pattern)]
use anyhow::Result;
use core::slice::SlicePattern;
use cqust_smack_server::{Llm, Options, TopicType, config};
use dashmap::DashMap;
use futures::{StreamExt, stream};
use murmur3::murmur3_x64_128;
use serde_json::json;
use std::{
    fs::File,
    io::{Cursor, Write},
    sync::Arc,
};
use tracing::error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let llm = Arc::new(Llm::new());
    llm.init().await?;
    let solutions: Arc<DashMap<u128, Vec<u8>>> = Default::default();
    let topics = sled::open("./cache")?;
    let stream = stream::iter(topics.iter().map(|entry| entry.map(|(k, v)| (k, v))));
    stream
        // RPM:1000; TPM:10000 when using DeepSeek R1
        .for_each_concurrent(config().solve_parallel as usize, |entry| {
            let llm_c = llm.clone();
            let sln_c = solutions.clone();
            async move {
                let (k, v) = entry.expect("failed to unwrapper cache db entry");
                let topic = serde_json::from_slice::<TopicType>(k.as_slice())
                    .expect("failed to deser topic");
                let options = serde_json::from_slice::<Options>(v.as_slice())
                    .expect("failed to deser options");
                let Ok(rst) = llm_c.ask(topic.clone(), options.clone()).await else {
                    error!("failed to obtain answer");
                    return;
                };
                let hash = murmur3_x64_128(
                    &mut Cursor::new({
                        let rst: String = topic.clone().into();
                        rst
                    }),
                    0,
                )
                .unwrap();
                info!("题目：{:?} 选项：{:?} 解答：{:?}", topic, options, rst);
                sln_c.insert(hash, rst);
            }
        })
        .await;
    let mut f = File::create("./solution_mapping.json")?;
    let serializable_solutions: std::collections::HashMap<_, _> = solutions
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect();
    f.write(json!(serializable_solutions).to_string().as_bytes())?;
    Ok(())
}
