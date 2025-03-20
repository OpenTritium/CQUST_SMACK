use anyhow::Result;
use cqust_smack_server::{Llm, TopicType};

#[tokio::main]
async fn main() -> Result<()> {
    let llm = Llm::new();
    llm.init().await?;
    let rst = llm.ask(
        TopicType::TrueOrFalse("根据《重庆科技大学学生公寓管理办法》，重庆科技大学学生公寓寒暑假期间绝对不允许学生留校住宿。（ ）".to_string()),
        vec!["A:对".to_string(), "B:错".to_string()]
    ).await?;
    println!("{:?}", rst);
    Ok(())
}
