mod mirror;
mod reports;

use std::path::PathBuf;

pub use mirror::Mirror;
pub use reports::{Exertion, Monitors, MonitorsLoader};

#[derive(thiserror::Error, Debug)]
pub enum MonitorsError {
    #[error("Failed to decompress the monitor file")]
    #[cfg(feature = "bzip2")]
    Decompress(#[from] bzip2::Error),
    #[error("Failed to read the monitor file: {1}")]
    Io(#[source] std::io::Error, PathBuf),
    #[error("Failed to deserialize the CSV file")]
    Csv(#[from] csv::Error),
    #[error("Failed to parse String")]
    Parse(#[from] std::num::ParseFloatError),
    #[error("Failed to parse String")]
    Regex(#[from] regex::Error),
    #[error("Entry {0} not found in Map")]
    MissingEntry(String),
    #[error("expected year {0}, found {1}")]
    YearMismatch(u32, u32),
}
