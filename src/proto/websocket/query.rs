use std::collections::HashMap;
use anyhow::*;

#[allow(dead_code)]
pub struct Query {
    pub name: String,
    pub key: String,
}

impl Query {
    pub fn from_str(src: &str) -> Result<Self> {
        let querys = parse(src);
        Ok(Self {
            name: querys
                .get("name")
                .ok_or_else(|| anyhow!("name is not found!"))?
                .to_string(),
            key: querys
                .get("key")
                .ok_or_else(|| anyhow!("key is not found!"))?
                .to_string(),
        })
    }
}

pub fn parse(src: &str) -> HashMap<String, String> {
    let mut querys = HashMap::with_capacity(5);

    src.split('&').for_each(|item| {
        if let Some((k, v)) = item.split_once('=') {
            querys.insert(k.to_string(), v.to_string());
        }
    });

    querys
}
