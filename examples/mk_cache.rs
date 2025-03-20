use cqust_smack_server::Page;

#[tokio::main]
async fn main() {
    Page::caching().await;
}
