use crate::modules::{HttpFinding, HttpModule, Module};
use crate::Error;
use async_trait::async_trait;
use reqwest::Client;
pub struct GitHeadDisclosure {}

impl GitHeadDisclosure {
    pub fn new() -> Self {
        GitHeadDisclosure {}
    }

    fn is_head_file(&self, content: &str) -> bool {
        return Some(0) == content.to_lowercase().trim().find("ref:");
    }
}

impl Module for GitHeadDisclosure {
    fn name(&self) -> String {
        String::from("http/git_head_disclosure")
    }

    fn description(&self) -> String {
        String::from("Check for .git/HEAD file disclosure")
    }
}

#[async_trait]
impl HttpModule for GitHeadDisclosure {
    async fn scan(
        &self,
        http_client: &Client,
        endpoint: &str,
    ) -> Result<Option<HttpFinding>, Error> {
        let url = format!("{}/.git/HEAD", &endpoint);
        let res = http_client
            .get(&url)
            .send()
            .await
            .map_err(Error::HttpError)?;

        let success = res.status().is_success();
        let body = res.text().await.map_err(Error::HttpError)?;
        if success && self.is_head_file(&body) {
            Ok(Some(HttpFinding::GitHeadDisclosure(url)))
        } else {
            Ok(None)
        }
    }
}
