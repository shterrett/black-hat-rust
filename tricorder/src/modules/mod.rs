use crate::error::Error;
use async_trait::async_trait;
use reqwest::Client;
mod http;
mod subdomains;
use http::{GitHeadDisclosure, GitlabOpenRegistrations};
use subdomains::{Crtsh, WebArchive};

pub trait Module {
    fn name(&self) -> String;
    fn description(&self) -> String;
}

#[async_trait]
pub trait SubdomainModule: Module {
    async fn enumerate(&self, domain: &str) -> Result<Vec<String>, Error>;
}

#[derive(Debug, Clone)]
pub struct Subdomain {
    pub domain: String,
    pub ports: Vec<Port>,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub port: u16,
    pub is_open: bool,
}

#[async_trait]
pub trait HttpModule: Module {
    async fn scan(
        &self,
        http_client: &Client,
        endpoint: &str,
    ) -> Result<Option<HttpFinding>, Error>;
}

pub fn all_http_modules() -> Vec<Box<dyn HttpModule>> {
    return vec![
        Box::new(GitlabOpenRegistrations::new()),
        Box::new(GitHeadDisclosure::new()),
    ];
}

pub fn all_subdomain_modules() -> Vec<Box<dyn SubdomainModule>> {
    return vec![Box::new(Crtsh::new()), Box::new(WebArchive::new())];
}

#[derive(Debug, Clone)]
pub enum HttpFinding {
    GitlabOpenRegistrations(String),
    GitHeadDisclosure(String),
}
