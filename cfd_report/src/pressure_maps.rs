//! Pressure maps
//!
//! Plot the pressure maps on M1 and M2 segments

use geotrans::{Segment, SegmentTrait};
use parse_monitors::{
    cfd::{self, BaselineTrait},
    pressure::{MirrorProperties, Pressure},
};
use rayon::prelude::*;
use std::path::Path;

use crate::{ReportError, error::PressureMapsError};

pub trait Config {
    const Y: u32;
    type CfdCase;
    fn configure(cfd_case: Self::CfdCase) -> Result<(String, Vec<String>), ReportError>;
}
impl Config for geotrans::M1 {
    const Y: u32 = parse_monitors::CFD_YEAR;
    type CfdCase = cfd::CfdCase<{ Self::Y }>;
    fn configure(cfd_case: Self::CfdCase) -> Result<(String, Vec<String>), ReportError> {
        Ok((
            "M1p.csv.z".to_string(),
            cfd::CfdDataFile::<{ Self::Y }>::M1Pressure
                .glob(cfd_case)
                .map_err(|e| PressureMapsError::new(cfd_case, e))?
                .into_iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect(),
        ))
    }
}
impl Config for geotrans::M2 {
    const Y: u32 = parse_monitors::CFD_YEAR;
    type CfdCase = cfd::CfdCase<{ Self::Y }>;
    fn configure(cfd_case: Self::CfdCase) -> Result<(String, Vec<String>), ReportError> {
        Ok((
            "M2p.csv.z".to_string(),
            cfd::CfdDataFile::<{ Self::Y }>::M2Pressure
                .glob(cfd_case)
                .map_err(|e| PressureMapsError::new(cfd_case, e))?
                .into_iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect(),
        ))
    }
}

pub fn task<M12, const Y: u32>(cfd_cases: &[cfd::CfdCase<Y>]) -> Result<(), ReportError>
where
    M12: Config<CfdCase = cfd::CfdCase<Y>> + Default,
    Segment<M12>: SegmentTrait,
    Pressure<M12>: MirrorProperties,
{
    cfd_cases.into_par_iter().for_each(|cfd_case| {
        // let now = Instant::now();
        let case_path = cfd::Baseline::<{ Y }>::path()
            .expect("undefined path to CFD repository")
            .join(cfd_case.to_string());
        let (_geometry, files) = M12::configure(cfd_case.clone()).unwrap();

        let _ = files.last().map(|file| {
            let path = Path::new(file);
            let csv_pressure = Pressure::<M12>::decompress(path.to_path_buf()).unwrap();
            //let csv_geometry =
            //    Pressure::<M12>::decompress(path.with_file_name(geometry)).unwrap();
            let mut pressures = Pressure::<M12>::load(csv_pressure).unwrap();
            pressures.pressure_map(case_path);
        });

        // println!(
        //     "{:<32}{}: {:>8}s",
        //     cfd_case,
        //     files.last().unwrap(),
        //     now.elapsed().as_secs()
        // );
    });
    Ok(())
}
