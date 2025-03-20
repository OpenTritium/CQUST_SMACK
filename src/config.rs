use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, sync::OnceLock};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub llm_api_key: String,
    pub solve_parallel: u8,
    pub document_path: String,
    pub cookie: String,
}

static CONFNIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFNIG.get_or_init(|| {
        let config = read_to_string("./config.toml").unwrap();
        let config = toml::from_str::<Config>(&config).unwrap();
        config
    })
}
