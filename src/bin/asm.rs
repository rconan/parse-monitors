//use asm::{pressure, refraction_index};
//use indicatif::ParallelProgressIterator;
use parse_monitors::{cfd, pressure::Pressure, FORCE_SAMPLING_FREQUENCY};
use rayon::prelude::*;
use std::{path::Path, time::Instant};

const R: f64 = 1.2;

fn main() -> anyhow::Result<()> {
    let duration = 400;
    type M12 = geotrans::M2;
    //let pattern = "M2p_M2p_*.csv.bz2";
    let geometry = "M2p.csv.bz2";
    let pressure_stats = "m2_pressure-stats_within.csv";
    cfd::Baseline::<2021>::at_zenith(cfd::ZenithAngle::Thirty)
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>()
        .into_par_iter()
        .for_each(|cfd_case| {
            let now = Instant::now();
            let case_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
            let files: Vec<_> = cfd::CfdDataFile::<2021>::M2Pressure
                .glob(cfd_case)
                .unwrap()
                .map(|p| p.unwrap().to_str().unwrap().to_string())
                .collect();
            let n_sample = duration * FORCE_SAMPLING_FREQUENCY as usize;
            let n_skip = if files.len() < n_sample {
                panic!("Not enough data sample")
            } else {
                files.len() - n_sample
            };

            let records: Vec<_> = files
                .into_par_iter()
                .skip(n_skip)
                //.progress_count(n_files as u64)
                .map(|file| {
                    let path = Path::new(&file);
                    let stem = Path::new(path.file_stem().unwrap())
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap();
                    let time = &stem[8..].parse::<f64>().unwrap();
                    let csv_pressure = Pressure::<M12>::decompress(path.to_path_buf()).unwrap();
                    let csv_geometry =
                        Pressure::<M12>::decompress(path.with_file_name(geometry)).unwrap();
                    let pressures = Pressure::<M12>::load(csv_pressure, csv_geometry).unwrap();

                    let filter = pressures
                        .local_radial_filter(1, Some(0.5), None)
                        .zip(pressures.local_radial_filter(2, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(3, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(4, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(5, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(6, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(7, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(7, None, Some(R)))
                        .map(|(l, r)| l && r);
                    let pressure_mean = pressures.mirror_average_pressure_by(filter);
                    let filter = pressures
                        .local_radial_filter(1, Some(0.5), None)
                        .zip(pressures.local_radial_filter(2, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(3, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(4, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(5, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(6, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(7, Some(0.5), None))
                        .map(|(l, r)| l || r)
                        .zip(pressures.local_radial_filter(7, None, Some(R)))
                        .map(|(l, r)| l && r);
                    let pressure_var =
                        pressures.mirror_average_pressure_var_by(pressure_mean, filter);
                    (*time, pressure_mean, pressure_var)
                })
                .collect();

            let filename = case_path.join(pressure_stats);
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_path(filename)
                .unwrap();
            let headers = [
                "Time [s]".to_string(),
                "Mean [Pa]".to_string(),
                "Var [Pa]".to_string(),
            ];
            wtr.write_record(&headers).unwrap();
            for data in records {
                wtr.serialize(data).unwrap();
            }
            wtr.flush().unwrap();
            println!("{:<32}: {:>8}s", cfd_case, now.elapsed().as_secs());
        });
    Ok(())
}
