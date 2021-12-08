use crate::{
    modules::{Module, SubdomainModule},
    Error,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

pub struct WebArchive {}

impl WebArchive {
    pub fn new() -> Self {
        WebArchive {}
    }
}

impl Module for WebArchive {
    fn name(&self) -> String {
        String::from("subdomains/webarchive")
    }

    fn description(&self) -> String {
        String::from("Use web.archive.org to find subdomains")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct WebArchiveResponse(Vec<Vec<String>>);

#[async_trait]
impl SubdomainModule for WebArchive {
    async fn enumerate(&self, domain: &str) -> Result<Vec<String>, Error> {
        let url = format!("https://web.archive.org/cdx/search/cdx?matchType=domain&fl=original&output=json&collapse=urlkey&url={}", domain);
        let res = reqwest::get(&url).await.map_err(Error::HttpError)?;

        let success = res.error_for_status().map_err(Error::HttpError)?;
        let archive_entries: WebArchiveResponse = success.json().await.map_err(Error::HttpError)?;
        let subdomains: HashSet<String> = archive_entries
            .0
            .into_iter()
            .flatten()
            .filter_map(|e| Url::parse(&e).ok())
            .filter_map(|url| url.host_str().map(|h| h.to_string()))
            .collect();

        Ok(subdomains.into_iter().collect())
    }
}
