//! Pressure statistics
//!
//! Compute the average pressure per segment and for the whole mirror as well as
//! the pressure standart deviation per segment

use glob::glob;
//use indicatif::ParallelProgressIterator;
use parse_monitors::{
    cfd::{self, BaselineTrait},
    pressure::Pressure,
    CFD_YEAR,
};
use rayon::prelude::*;
use std::{error::Error, path::Path, time::Instant};

trait Config {
    fn configure(cfd_case: cfd::CfdCase<{ CFD_YEAR }>) -> anyhow::Result<(String, Vec<String>)>;
}
impl Config for geotrans::M1 {
    fn configure(cfd_case: cfd::CfdCase<{ CFD_YEAR }>) -> anyhow::Result<(String, Vec<String>)> {
        Ok((
            "M1p.csv.z".to_string(),
            cfd::CfdDataFile::<{ CFD_YEAR }>::M1Pressure
                .glob(cfd_case)?
                .into_iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect(),
        ))
    }
}
impl Config for geotrans::M2 {
    fn configure(cfd_case: cfd::CfdCase<{ CFD_YEAR }>) -> anyhow::Result<(String, Vec<String>)> {
        Ok((
            "M2p.csv.z".to_string(),
            cfd::CfdDataFile::<{ CFD_YEAR }>::M2Pressure
                .glob(cfd_case)?
                .into_iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect(),
        ))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    type M12 = geotrans::M1;
    cfd::Baseline::<{ CFD_YEAR }>::default()
        //.extras()
        .into_iter()
        /*
                   .filter(|c| {
                       *c == cfd::CfdCase::new(
                           cfd::ZenithAngle::Thirty,
                           cfd::Azimuth::FortyFive,
                           cfd::Enclosure::OpenStowed,
                           cfd::WindSpeed::Seven,
                       )
                   })
        */
        //        .skip(1)
        //.collect::<Vec<cfd::CfdCase<{CFD_YEAR}>>>()
        //.into_par_iter()
        .nth(7)
        .into_iter()
        .for_each(|cfd_case| {
            println!("{cfd_case}");
            let now = Instant::now();
            let case_path = cfd::Baseline::<{ CFD_YEAR }>::path()
                .unwrap()
                .join(cfd_case.to_string());
            let (_geometry, files) = M12::configure(cfd_case).unwrap();
            //let n_files = files.len();

            let records: Vec<_> = files
                .into_iter()
                .collect::<Vec<String>>()
                .par_iter()
                //.progress_count(n_files as u64)
                .map(|file| {
                    let path = Path::new(file);
                    let stem = Path::new(path.file_stem().unwrap())
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap();
                    let time = &stem[8..].parse::<f64>().unwrap();
                    let csv_pressure = Pressure::<M12>::decompress(path.to_path_buf()).unwrap();
                    //let csv_geometry =
                    //    Pressure::<M12>::decompress(path.with_file_name("M2p.csv.bz2")).unwrap();
                    let mut pressures = Pressure::<M12>::load(csv_pressure).unwrap();
                    let segments_pressure = pressures.segments_average_pressure();
                    let segments_pressure_std = pressures.segments_pressure_std();
                    let average_pressure = pressures.mirror_average_pressure();
                    (
                        *time,
                        average_pressure,
                        segments_pressure,
                        segments_pressure_std,
                    )
                })
                .collect();

            let filename = case_path.join("m1_pressure-stats.csv");
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_path(filename)
                .unwrap();
            let headers: Vec<_> = std::iter::once("Time [s]".to_string())
                .chain(std::iter::once("Mean [Pa]".to_string()))
                .chain((1..=7).map(|sid| format!("S{} Mean [Pa]", sid)))
                .chain((1..=7).map(|sid| format!("S{} Std [Pa]", sid)))
                .collect();
            wtr.write_record(&headers).unwrap();
            for data in records {
                wtr.serialize(data).unwrap();
            }
            wtr.flush().unwrap();
            println!("{:<32}: {:>8}s", cfd_case, now.elapsed().as_secs());
        });
    Ok(())
}
