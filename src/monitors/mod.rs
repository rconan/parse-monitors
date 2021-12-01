mod mirror;
mod reports;
pub use mirror::Mirror;
pub use reports::{Exertion, Monitors, MonitorsLoader};

#[derive(thiserror::Error, Debug)]
pub enum MonitorsError {
    #[error("Failed to decompress the monitor file")]
    Decompress(#[from] bzip2::Error),
    #[error("Failed to open the monitor file")]
    Io(#[from] std::io::Error),
    #[error("Failed to deserialize the CSV file")]
    Csv(#[from] csv::Error),
    #[error("Failed to parse String")]
    Parse(#[from] std::num::ParseFloatError),
    #[error("Failed to parse String")]
    Regex(#[from] regex::Error),
    #[error("Entry {0} not found in Map")]
    MissingEntry(String),
}
