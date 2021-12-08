use futures::stream;
use futures::StreamExt;
use reqwest::Client;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Duration;
use std::time::Instant;

use crate::dns;
use crate::modules::HttpModule;
use crate::modules::Subdomain;
use crate::ports;
use crate::{modules, Error};

pub async fn modules() -> Result<(), Error> {
    let http = modules::all_http_modules();
    let subdomain = modules::all_subdomain_modules();
    println!("Http Modules:");
    for m in http.into_iter() {
        println!("  {}", m.name());
    }
    for m in subdomain.into_iter() {
        println!("  {}", m.name());
    }
    Ok(())
}

pub async fn scan(target: &str) -> Result<(), Error> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Building tokio's runtime");

    let http_timeout = Duration::from_secs(10);
    let http_client = Client::builder()
        .timeout(http_timeout)
        .build()
        .map_err(Error::HttpError)?;
    let dns_resolver = dns::new_resolver();

    let subdomains_concurrency = 20;
    let dns_concurrency = 100;
    let ports_concurrency = 200;
    let vuln_concurrency = 20;
    let scan_start = Instant::now();

    let subdomain_modules = modules::all_subdomain_modules();

    runtime.block_on(async move {
        let mut subdomains: Vec<String> = stream::iter(subdomain_modules.into_iter())
            .map(|module| async move { module.enumerate(target).await.ok() })
            .buffer_unordered(subdomains_concurrency)
            .filter_map(|domain| async { domain })
            .collect::<Vec<Vec<String>>>()
            .await
            .into_iter()
            .flatten()
            .collect();

        subdomains.push(target.to_string());

        let subdomains: Vec<Subdomain> = HashSet::<String>::from_iter(subdomains.into_iter())
            .into_iter()
            .filter(|s| s.contains(target))
            .map(|d| Subdomain {
                domain: d,
                ports: vec![],
            })
            .collect();

        let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
            .map(|d| dns::resolves(&dns_resolver, d))
            .buffer_unordered(dns_concurrency)
            .filter_map(|d| async move { d })
            .collect()
            .await;

        let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
            .map(|d| ports::scan_ports(ports_concurrency, d))
            .buffer_unordered(1)
            .collect()
            .await;

        for s in &subdomains {
            println!("{}", s.domain);
            for p in &s.ports {
                println!("  {}", p.port);
            }
        }

        println!("---------------------- Vulnerabilities ----------------------");

        let mut targets: Vec<(Box<dyn HttpModule>, String)> = Vec::new();
        for subdomain in subdomains {
            for port in subdomain.ports {
                let http_modules = modules::all_http_modules();
                for http_module in http_modules {
                    let target = format!("http://{}:{}", &subdomain.domain, port.port);
                    targets.push((http_module, target));
                }
            }
        }
        stream::iter(targets.into_iter())
            .for_each_concurrent(vuln_concurrency, move |(m, url)| {
                let http_client = http_client.clone();
                async move {
                    match m.scan(&http_client, &url).await {
                        Ok(Some(finding)) => println!("{:?}", finding),
                        Ok(None) => {}
                        Err(err) => println!(
                            "error in scan {} for url {} with module {}",
                            err,
                            &url,
                            &m.name()
                        ),
                    }
                }
            })
            .await
    });

    let scan_elapsed = scan_start.elapsed();
    println!("Scan completed in {:?}", scan_elapsed);

    Ok(())
}
