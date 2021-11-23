use crate::error::Error;
use crate::model::{CrtShEntry, Subdomain};
use reqwest::blocking::Client;
use std::{collections::HashSet, time::Duration};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};

pub fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    let entries: Vec<CrtShEntry> = http_client
        .get(&format!("https://crt.sh?q=%25.{}&output=json", target))
        .send()
        .and_then(|r| r.json())
        .map_err(Error::HttpError)?;

    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .map(|entry| {
            entry
                .name
                .split("\n")
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .filter(|s| s != target)
        .filter(|s| !s.contains("*"))
        .collect();
    subdomains.insert(target.to_string());

    let subdomains: Vec<Subdomain> = subdomains
        .into_iter()
        .map(|d| Subdomain {
            ports: Vec::new(),
            domain: d,
        })
        .filter(resolves)
        .collect();
    Ok(subdomains)
}

fn resolves(domain: &Subdomain) -> bool {
    let dns_resolver = Resolver::new(
        ResolverConfig::default(),
        ResolverOpts {
            timeout: Duration::from_secs(4),
            ..Default::default()
        },
    )
    .expect("subdomain resolver: building DNS client");
    dns_resolver.lookup_ip(domain.domain.as_str()).is_ok()
}
