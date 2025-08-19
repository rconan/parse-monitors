//use asm::{pressure, refraction_index};
//use indicatif::ParallelProgressIterator;
use parse_monitors::{
    cfd::{self, BaselineTrait},
    pressure::Pressure,
    CFD_YEAR, FORCE_SAMPLING_FREQUENCY,
};
use rayon::prelude::*;
use std::{env, iter::once, path::Path, time::Instant};

const R: f64 = 1.2;

fn main() -> anyhow::Result<()> {
    let duration = 400;

    let job_idx = env::var("AWS_BATCH_JOB_ARRAY_INDEX")
        .expect("AWS_BATCH_JOB_ARRAY_INDEX env var missing")
        .parse::<usize>()
        .expect("AWS_BATCH_JOB_ARRAY_INDEX parsing failed");

    let cfd_case = cfd::Baseline::<{ CFD_YEAR }>::default()
        .into_iter()
        .nth(job_idx)
        .unwrap();

    type M12 = geotrans::M2;
    //let pattern = "M2p_M2p_*.csv.bz2";
    let geometry = "M2p.csv.bz2";
    let pressure_stats = "m2-es_pressure-stats.csv";

    let now = Instant::now();
    let case_path = cfd::Baseline::<{ CFD_YEAR }>::path()?.join(cfd_case.to_string());
    let files: Vec<_> = cfd::CfdDataFile::<{ CFD_YEAR }>::M2Pressure
        .glob(cfd_case)
        .unwrap()
        .into_iter()
        .map(|p| p.to_str().unwrap().to_string())
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
            let csv_geometry = Pressure::<M12>::decompress(path.with_file_name(geometry)).unwrap();
            let pressures = Pressure::<M12>::load(csv_pressure).unwrap();

            let mut pressure_mean = Vec::<f64>::new();
            for i in 0..12 {
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
                    .map(|(l, r)| l && r)
                    .zip(pressures.xy_iter().map(|(x, y)| {
                        if i < 6 {
                            let (s, c) = (90f64 - 60f64 * i as f64).to_radians().sin_cos();
                            (x - 0.55 * c).hypot(y - 0.55 * s) < 0.05
                        } else {
                            let (s, c) = (-60f64 * i as f64).to_radians().sin_cos();
                            (x - 0.942 * c).hypot(y - 0.942 * s) < 0.05
                        }
                    }))
                    .map(|(l, r)| l && r);
                pressure_mean.push(pressures.mirror_average_pressure_by(filter));
            }
            (*time, pressure_mean)
        })
        .collect();

    let filename = case_path.join(pressure_stats);
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_path(filename)
        .unwrap();
    let headers: Vec<String> = once("Time [s]".to_string())
        .chain((1..13).map(|i| format!("ES {}", i)))
        .collect();
    wtr.write_record(&headers).unwrap();
    for data in records {
        wtr.serialize(data).unwrap();
    }
    wtr.flush().unwrap();
    println!("{:<32}: {:>8}s", cfd_case, now.elapsed().as_secs());

    Ok(())
}
