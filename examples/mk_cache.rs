use cqust_smack_server::Page;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    Page::caching().await;
}
