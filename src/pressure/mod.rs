mod mirrors;
pub use mirrors::*;
mod telescope;
pub use telescope::*;

#[derive(thiserror::Error, Debug)]
pub enum PressureError {
    #[cfg(feature = "bzip2")]
    #[error("Failed to decompress the file")]
    Decompress(#[from] bzip2::Error),
    #[error("Failed to open the pressure file")]
    Io(#[from] std::io::Error),
    #[error("Failed to deserialize the CSV file")]
    Csv(#[from] csv::Error),
    #[error("Failed to apply geometric transformation")]
    Geotrans(#[from] geotrans::Error),
    #[error("Missing decompression protocol")]
    Decompression,
}
