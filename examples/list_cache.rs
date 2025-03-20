#![feature(slice_pattern)]
use anyhow::Result;
use core::slice::SlicePattern;
use cqust_smack_server::{Options, TopicType};

fn main() -> Result<()> {
    let sb = sled::open("./cache").unwrap();
    for i in sb.iter() {
        let (k, v) = i?;
        let k = serde_json::from_slice::<TopicType>(k.as_slice())?;
        let v = serde_json::from_slice::<Options>(v.as_slice())?;
        println!("{:?},{:?}", k, v);
    }
    Ok(())
}
