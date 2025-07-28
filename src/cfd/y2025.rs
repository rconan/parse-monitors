use std::path::PathBuf;

use super::{Baseline, BaselineTrait, CfdCase, CfdDataFile, CfdError};

type Result<T> = std::result::Result<T, CfdError>;

impl CfdDataFile<2025> {
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
    pub fn glob(self, cfd_case: CfdCase<2025>) -> Result<Vec<PathBuf>> {
        use CfdDataFile::*;
        let cfd_path = Baseline::<2025>::path()?.join(cfd_case.to_string());
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
