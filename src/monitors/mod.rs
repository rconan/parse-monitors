mod mirror;
mod reports;

use std::path::PathBuf;

pub use mirror::Mirror;
pub use reports::{Exertion, Monitors, MonitorsLoader};

#[derive(thiserror::Error, Debug)]
pub enum MonitorsError {
    #[error("failed to decompress the monitor file")]
    #[cfg(feature = "bzip2")]
    Decompress(#[from] bzip2::Error),
    #[error("failed to read the monitor file: {1}")]
    Io(#[source] std::io::Error, PathBuf),
    #[error("failed to deserialize the CSV file")]
    Csv(#[from] csv::Error),
    #[error("failed to parse String")]
    Parse(#[from] std::num::ParseFloatError),
    #[error("failed to parse String")]
    Regex(#[from] regex::Error),
    #[error("entry {0} not found in Map")]
    MissingEntry(String),
    #[error("expected year {0}, found {1}")]
    YearMismatch(u32, u32),
    #[cfg(feature = "plot")]
    #[error("failed to plot forces: {0}")]
    PlotForces(String),
}
