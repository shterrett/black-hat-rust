use crate::modules::{HttpFinding, HttpModule, Module};
use crate::Error;
use async_trait::async_trait;
use reqwest::Client;

pub struct GitlabOpenRegistrations {}

impl GitlabOpenRegistrations {
    pub fn new() -> Self {
        GitlabOpenRegistrations {}
    }
}

impl Module for GitlabOpenRegistrations {
    fn name(&self) -> String {
        String::from("http/gitlab_open_registration")
    }

    fn description(&self) -> String {
        String::from("Check if the GitLab instance is open to registrations")
    }
}

#[async_trait]
impl HttpModule for GitlabOpenRegistrations {
    async fn scan(
        &self,
        http_client: &Client,
        endpoint: &str,
    ) -> Result<Option<HttpFinding>, Error> {
        let url = format!("{}", &endpoint);
        let res = http_client
            .get(&url)
            .send()
            .await
            .map_err(Error::HttpError)?;

        let success = res.status().is_success();
        let body = res.text().await.map_err(Error::HttpError)?;
        if success
            && body.contains("This is a self-managed instance of GitLab")
            && body.contains("Register")
        {
            Ok(Some(HttpFinding::GitlabOpenRegistrations(url)))
        } else {
            Ok(None)
        }
    }
}
