use crate::error::Error;
use crate::model::{CrtShEntry, Subdomain};
use futures::stream;
use futures::StreamExt;
use reqwest::Client;
use std::{collections::HashSet, time::Duration};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    name_server::{GenericConnection, GenericConnectionProvider, TokioRuntime},
    AsyncResolver,
};

type DnsResolver = AsyncResolver<GenericConnection, GenericConnectionProvider<TokioRuntime>>;

pub async fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    let entries: Vec<CrtShEntry> = http_client
        .get(&format!("https://crt.sh?q=%25.{}&output=json", target))
        .send()
        .await
        .map_err(Error::HttpError)?
        .json()
        .await
        .map_err(Error::HttpError)?;

    let dns_resolver = AsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts {
            timeout: Duration::from_secs(4),
            ..Default::default()
        },
    )
    .expect("Subdomain resolver: building dns client");

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

    let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
        .map(|d| Subdomain {
            ports: Vec::new(),
            domain: d,
        })
        .filter_map(|d| {
            let dns_resolver = dns_resolver.clone();
            async move {
                if resolves(&dns_resolver, &d).await {
                    Some(d)
                } else {
                    None
                }
            }
        })
        .collect()
        .await;
    Ok(subdomains)
}

async fn resolves(dns_resolver: &DnsResolver, domain: &Subdomain) -> bool {
    dns_resolver.lookup_ip(domain.domain.as_str()).await.is_ok()
}
