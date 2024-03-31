mod config;
use config::Config;

use crossbeam_skiplist::SkipMap;
use std::{
    fs,
    sync::{atomic::AtomicUsize, Arc},
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config: Arc<Config> = Arc::new(toml::from_str(&fs::read_to_string("config.toml")?)?);
    let spam_count_map: SkipMap<String, AtomicUsize> = SkipMap::new();

    Ok(())
}
