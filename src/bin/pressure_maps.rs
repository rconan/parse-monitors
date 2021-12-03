//! Pressure maps
//!
//! Plot the pressure maps on M1 and M2 segments

use glob::glob;
use parse_monitors::{cfd, pressure::Pressure};
use rayon::prelude::*;
use std::{error::Error, path::Path, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    type M12 = geotrans::M1;
    let pattern = "M1p_M1p_*.csv.bz2";
    let geometry = "M1p.csv.bz2";

    cfd::Baseline::<2021>::default()
        .extras()
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>()
        .into_par_iter()
        .for_each(|cfd_case| {
            let now = Instant::now();
            let case_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
            let files: Vec<_> = glob(case_path.join("pressures").join(pattern).to_str().unwrap())
                .unwrap()
                .map(|p| p.unwrap().to_str().unwrap().to_string())
                .collect();

            let _ = files.last().map(|file| {
                let path = Path::new(file);
                let csv_pressure = Pressure::<M12>::decompress(path.to_path_buf()).unwrap();
                let csv_geometry =
                    Pressure::<M12>::decompress(path.with_file_name(geometry)).unwrap();
                let mut pressures = Pressure::<M12>::load(csv_pressure, csv_geometry).unwrap();
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
