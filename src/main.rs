#![feature(duration_constructors)]
use spider::Page;
use std::time::Duration;
use tokio::time;

mod spider;

#[tokio::main]
async fn main() {
    let timeout = Duration::from_hours(4);
    time::timeout(timeout, async {
        let sb = sled::open("./cache").unwrap();
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let data = spider::Page::new().fetch().await.unwrap();
            Page::cache(&sb, &data);
        }
    })
    .await
    .unwrap();
}
