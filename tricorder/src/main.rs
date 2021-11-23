use reqwest::{blocking::Client, redirect};
use std::{env, time::Duration};
mod error;
pub use error::Error;
mod common_ports;
mod model;
use model::Subdomain;
mod ports;
mod subdomains;
use rayon::prelude::*;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(Error::CliUsage.into());
    }

    let target = args[1].as_str();
    let http_timeout = Duration::from_secs(5);
    let http_client = Client::builder()
        .redirect(redirect::Policy::limited(4))
        .timeout(http_timeout)
        .build()
        .map_err(Error::HttpError)?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(256)
        .build()
        .unwrap();

    pool.install(|| {
        let scan_result: Vec<Subdomain> = subdomains::enumerate(&http_client, target)
            .unwrap()
            .into_par_iter()
            .map(ports::scan_ports)
            .collect();
        for subdomain in scan_result {
            println!("{}:", subdomain.domain);
            for port in subdomain.ports {
                println!("    {}", port.port);
            }
            println!("");
        }
    });
    Ok(())
}
