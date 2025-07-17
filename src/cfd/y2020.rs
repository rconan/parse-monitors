use super::{Baseline, BaselineTrait, CfdCase, CfdDataFile, CfdError};

impl CfdDataFile<2020> {
    pub fn glob(
        self,
        cfd_case: CfdCase<2021>,
    ) -> std::result::Result<impl Iterator<Item = glob::GlobResult>, CfdError> {
        let cfd_path = Baseline::<2021>::path().join(cfd_case.to_string());
        match self {
            CfdDataFile::M1Pressure => Ok(glob::glob(
                cfd_path.join("M1_data_Mod_M1_Data_*.csv").to_str().unwrap(),
            )?),
            CfdDataFile::M2Pressure => Ok(glob::glob(
                cfd_path.join("M2_data_Mod_M2_Data_*.csv").to_str().unwrap(),
            )?),
            CfdDataFile::TemperatureField => Ok(glob::glob(
                cfd_path.join("OPDData_OPD_Data_*.csv.gz").to_str().unwrap(),
            )?),
            CfdDataFile::OpticalPathDifference => Ok(glob::glob(
                cfd_path.join("OPDData_OPD_Data_*.npz").to_str().unwrap(),
            )?),
            _ => Err(CfdError::DataFile(format!("{:?}", self))),
        }
    }
}
