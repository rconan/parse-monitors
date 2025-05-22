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
pub use baseline::{Baseline, BaselineError, BaselineTrait};
pub use cfd_case::{Azimuth, CfdCase, CfdCaseError, Enclosure, WindSpeed, ZenithAngle};

use std::path::PathBuf;

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

type Result<T> = std::result::Result<T, CfdError>;

/// Data file collections available in the CFD database
#[derive(Debug)]
pub enum CfdDataFile<const YEAR: u32> {
    M1Pressure,
    M2Pressure,
    TemperatureField,
    OpticalPathDifference,
    TelescopePressure,
}
impl CfdDataFile<2021> {
    pub fn pattern(self) -> String {
        use CfdDataFile::*;
        String::from(match self {
            M1Pressure => "M1p_M1p_",
            M2Pressure => "M2p_M2p_",
            TemperatureField => "optvol_optvol_",
            OpticalPathDifference => "optvol_optvol_",
            TelescopePressure => "Telescope_p_table_",
        })
    }
    pub fn glob(self, cfd_case: CfdCase<2021>) -> Result<Vec<PathBuf>> {
        use CfdDataFile::*;
        let cfd_path = Baseline::<2021>::path().join(cfd_case.to_string());
        let paths = match self {
            M1Pressure => glob::glob(
                cfd_path
                    .join("pressures")
                    .join("M1p_M1p_*.csv.z")
                    .to_str()
                    .unwrap(),
            ),
            M2Pressure => glob::glob(
                cfd_path
                    .join("pressures")
                    .join("M2p_M2p_*.csv.z")
                    .to_str()
                    .unwrap(),
            ),
            TemperatureField => glob::glob(
                cfd_path
                    .join("optvol")
                    .join("optvol_optvol_*.csv.gz")
                    .to_str()
                    .unwrap(),
            ),
            OpticalPathDifference => glob::glob(
                cfd_path
                    .join("optvol")
                    .join("optvol_optvol_*.npz")
                    .to_str()
                    .unwrap(),
            ),
            TelescopePressure => glob::glob(
                cfd_path
                    .join("pressures")
                    .join("Telescope_p_table_*.csv.z")
                    .to_str()
                    .unwrap(),
            ),
        }?;
        Ok(paths.collect::<std::result::Result<Vec<PathBuf>, glob::GlobError>>()?)
    }
}
impl CfdDataFile<2020> {
    pub fn glob(
        self,
        cfd_case: CfdCase<2021>,
    ) -> std::result::Result<impl Iterator<Item = glob::GlobResult>, CfdError> {
        use CfdDataFile::*;
        let cfd_path = Baseline::<2021>::path().join(cfd_case.to_string());
        match self {
            M1Pressure => Ok(glob::glob(
                cfd_path.join("M1_data_Mod_M1_Data_*.csv").to_str().unwrap(),
            )?),
            M2Pressure => Ok(glob::glob(
                cfd_path.join("M2_data_Mod_M2_Data_*.csv").to_str().unwrap(),
            )?),
            TemperatureField => Ok(glob::glob(
                cfd_path.join("OPDData_OPD_Data_*.csv.gz").to_str().unwrap(),
            )?),
            OpticalPathDifference => Ok(glob::glob(
                cfd_path.join("OPDData_OPD_Data_*.npz").to_str().unwrap(),
            )?),
            _ => Err(CfdError::DataFile(format!("{:?}", self))),
        }
    }
}
