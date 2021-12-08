use crate::{
    modules::{Module, SubdomainModule},
    Error,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub struct Crtsh {}

impl Crtsh {
    pub fn new() -> Self {
        Crtsh {}
    }
}

impl Module for Crtsh {
    fn name(&self) -> String {
        String::from("subdomains/crtsh")
    }

    fn description(&self) -> String {
        String::from("Use crt.sh/ to find subdomains")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CrtshEntry {
    name_value: String,
}

#[async_trait]
impl SubdomainModule for Crtsh {
    async fn enumerate(&self, domain: &str) -> Result<Vec<String>, Error> {
        let url = format!("https://crt.sh/?q=%25.{}&output=json", domain);
        let res = reqwest::get(&url).await.map_err(Error::HttpError)?;

        let success = res.error_for_status().map_err(Error::HttpError)?;
        let crtsh_entries: Vec<CrtshEntry> = success.json().await.map_err(Error::HttpError)?;
        let subdomains: HashSet<String> = crtsh_entries
            .into_iter()
            .map(|entry| {
                entry
                    .name_value
                    .split("\n")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>()
            })
            .flatten()
            .filter(|s: &String| !s.contains("*"))
            .collect();

        Ok(subdomains.into_iter().collect())
    }
}
