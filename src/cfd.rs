/*!
# CFD database model based on Rust types

## Examples

Iterator that iterates over all the [CfdCase]s of CFD [Baseline] 2021
```
use parse_monitors::cfd;
let cfd_cases_iter = cfd::Baseline::<2021>::default().into_iter();
```
*/

mod baseline;
mod cfd_case;
#[cfg(feature = "2020")]
mod y2020;
#[cfg(feature = "2021")]
mod y2021;
#[cfg(feature = "2025")]
mod y2025;
pub use baseline::{Baseline, BaselineError, BaselineTrait};
pub use cfd_case::{Azimuth, CfdCase, CfdCaseError, Enclosure, WindSpeed, ZenithAngle};

#[derive(thiserror::Error, Debug)]
pub enum CfdError {
    #[error("Failed to read CFD data file")]
    ReadDataFile(#[from] glob::GlobError),
    #[error("Data file not recognized")]
    DataFileGlob(#[from] glob::PatternError),
    #[error("{0} data not available")]
    DataFile(String),
    #[error("CFD baseline error")]
    Baseline(#[from] BaselineError),
}

/// Data file collections available in the CFD database
#[derive(Debug)]
pub enum CfdDataFile<const YEAR: u32> {
    M1Pressure,
    M2Pressure,
    TemperatureField,
    OpticalPathDifference,
    TelescopePressure,
}
