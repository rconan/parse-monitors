//! Center of pressure and associated force and moment

use glob::glob;
use indicatif::ParallelProgressIterator;
use parse_monitors::{cfd, pressure::Pressure};
use rayon::prelude::*;
use std::{error::Error, path::Path, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    cfd::Baseline::<2021>::default()
        .extras()
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
        .collect::<Vec<cfd::CfdCase<2021>>>()
        .into_iter()
        .for_each(|cfd_case| {
            let now = Instant::now();
            let case_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
            let files: Vec<_> = glob(
                case_path
                    .join("pressures")
                    .join("M2p_M2p_*.csv.bz2")
                    .to_str()
                    .unwrap(),
            )
            .unwrap()
            .map(|p| p.unwrap().to_str().unwrap().to_string())
            .collect();
            let n_files = files.len();

            let time_cop_fm: Vec<_> = files
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
                    let csv_pressure = Pressure::decompress(path.to_path_buf()).unwrap();
                    let csv_geometry =
                        Pressure::decompress(path.with_file_name("M2p.csv.bz2")).unwrap();
                    let mut pressures = Pressure::load(csv_pressure, csv_geometry).unwrap();
                    let cop_fm: Vec<_> = (1..=7)
                        .map(|sid| pressures.segment_pressure_integral(sid))
                        .collect();
                    (*time, cop_fm)
                })
                .collect();

            let filename = case_path.join("m2_center_of_pressure.csv");
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_path(filename)
                .unwrap();
            let headers: Vec<_> = std::iter::once("Time [s]".to_string())
                .chain((1..=7).flat_map(|sid| {
                    ["X", "Y", "Z"]
                        .iter()
                        .map(|xyz| format!("S{} COP {} [M]", sid, xyz))
                        .chain(
                            ["X", "Y", "Z"]
                                .iter()
                                .map(|xyz| format!("S{} FORCE {} [N]", sid, xyz))
                                .chain(
                                    ["X", "Y", "Z"]
                                        .iter()
                                        .map(|xyz| format!("S{} MOMENT {} [N.M]", sid, xyz))
                                        .collect::<Vec<String>>(),
                                )
                                .collect::<Vec<String>>(),
                        )
                        .collect::<Vec<String>>()
                }))
                .collect();
            wtr.write_record(&headers).unwrap();
            for data in time_cop_fm {
                wtr.serialize(data).unwrap();
            }
            wtr.flush().unwrap();
            println!("{:<32}: {:>8}s", cfd_case, now.elapsed().as_secs());
        });
    Ok(())
}
