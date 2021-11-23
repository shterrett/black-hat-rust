use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Usage: tricorder <kerkour.com>")]
    CliUsage,
    #[error("Http Error")]
    HttpError(reqwest::Error),
}
