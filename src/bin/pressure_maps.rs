//! Pressure maps
//!
//! Plot the pressure maps on M1 and M2 segments

use parse_monitors::{cfd, cfd::BaselineTrait, pressure::Pressure};
use rayon::prelude::*;
use std::{path::Path, time::Instant};

trait Config {
    fn configure(cfd_case: cfd::CfdCase<2021>) -> anyhow::Result<(String, Vec<String>)>;
}
impl Config for geotrans::M1 {
    fn configure(cfd_case: cfd::CfdCase<2021>) -> anyhow::Result<(String, Vec<String>)> {
        Ok((
            "M1p.csv.z".to_string(),
            cfd::CfdDataFile::<2021>::M1Pressure
                .glob(cfd_case)?
                .into_iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect(),
        ))
    }
}
impl Config for geotrans::M2 {
    fn configure(cfd_case: cfd::CfdCase<2021>) -> anyhow::Result<(String, Vec<String>)> {
        Ok((
            "M2p.csv.z".to_string(),
            cfd::CfdDataFile::<2021>::M2Pressure
                .glob(cfd_case)?
                .into_iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect(),
        ))
    }
}

fn main() -> anyhow::Result<()> {
    type M12 = geotrans::M2;

    cfd::Baseline::<2021>::default()
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>()
        .into_par_iter()
        .for_each(|cfd_case| {
            let now = Instant::now();
            let case_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
            let (_geometry, files) = M12::configure(cfd_case).unwrap();

            let _ = files.last().map(|file| {
                let path = Path::new(file);
                let csv_pressure = Pressure::<M12>::decompress(path.to_path_buf()).unwrap();
                //let csv_geometry =
                //    Pressure::<M12>::decompress(path.with_file_name(geometry)).unwrap();
                let mut pressures = Pressure::<M12>::load(csv_pressure).unwrap();
                pressures.pressure_map(case_path);
            });

            println!(
                "{:<32}{}: {:>8}s",
                cfd_case,
                files.last().unwrap(),
                now.elapsed().as_secs()
            );
        });
    Ok(())
}
