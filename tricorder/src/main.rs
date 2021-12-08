mod cli;
mod dns;
mod error;
mod ports;
use error::Error;
mod model;
mod modules;
use clap::{App, Arg, SubCommand};
mod common_ports;
use env_logger;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommand(SubCommand::with_name("modules").about("list all modules"))
        .subcommand(
            SubCommand::with_name("scan").about("scan a target").arg(
                Arg::with_name("target")
                    .help("the domain to scan")
                    .required(true)
                    .index(1),
            ),
        )
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .setting(clap::AppSettings::VersionlessSubcommands)
        .get_matches();

    env::set_var("RUST_LOG", "info,trust_dns_proto=error");
    env_logger::init();

    if let Some(_) = cli.subcommand_matches("modules") {
        cli::modules().await
    } else if let Some(matches) = cli.subcommand_matches("modules") {
        let target = matches.value_of("target").unwrap();
        cli::scan(target).await
    } else {
        Err(Error::CliUsage)
    }
}
