//use asm::{pressure, refraction_index};
use geotrans::{Segment, SegmentTrait, Transform, M2};
use parse_monitors::{
    cfd, cfd::BaselineTrait, temperature::Temperature, TEMPERATURE_SAMPLING_FREQUENCY,
};
use rayon::prelude::*;
use std::{env, iter::once, path::Path, time::Instant};

const R: f64 = 1.2;

fn main() -> anyhow::Result<()> {
    let duration = 400;
    let pressure_stats = "m2-es_temperature-stats.csv";
    let job_idx = env::var("AWS_BATCH_JOB_ARRAY_INDEX")
        .expect("AWS_BATCH_JOB_ARRAY_INDEX env var missing")
        .parse::<usize>()
        .expect("AWS_BATCH_JOB_ARRAY_INDEX parsing failed");

    let cfd_case = cfd::Baseline::<2021>::default()
        .into_iter()
        .nth(job_idx)
        .unwrap();

    let now = Instant::now();
    let case_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
    let files: Vec<_> = cfd::CfdDataFile::<2021>::TemperatureField
        .glob(cfd_case)
        .unwrap()
        .map(|p| p.unwrap().to_str().unwrap().to_string())
        .collect();
    let n_sample = duration * TEMPERATURE_SAMPLING_FREQUENCY as usize;
    let n_skip = if files.len() < n_sample {
        0
    } else {
        files.len() - n_sample
    };
    let pattern = cfd::CfdDataFile::<2021>::TemperatureField.pattern();

    let records: anyhow::Result<Vec<_>> = files
        .into_par_iter()
        .skip(n_skip)
        //.progress_count(n_sample as u64)
        .map(|file| {
            let path = Path::new(&file);
            let time = Path::new(path.file_stem().unwrap())
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(&pattern, "")
                .parse::<f64>()?;
            type Data = Vec<f64>;
            let es_temp: Vec<(i32, f64)> = {
                let (x, y, z, temp): (Data, Data, Data, Data) = {
                    let temperature = Temperature::from_path(path)?;
                    (
                        temperature.x_iter().collect(),
                        temperature.y_iter().collect(),
                        temperature.z_iter().collect(),
                        temperature.temperature_iter().cloned().collect(),
                    )
                };

                x.into_iter()
                    .zip(y.into_iter())
                    .zip(z.into_iter())
                    .zip(temp.into_iter())
                    .filter(|((_, z), _)| *z > 23.99 && *z < 24.29)
                    .filter(|(((x, y), _), _)| x.hypot(*y) < R)
                    .filter(|(((x, y), z), _)| {
                        let u = [*x, *y, *z];
                        (1..=7).fold(false, |s, sid| {
                            let v = u.fro(Segment::<M2>::new(sid)).unwrap();
                            let r = v[0].hypot(v[1]);
                            let m = r >= 0.5 && r < 0.55;
                            s || m
                        })
                    })
                    .flat_map(|(((x, y), _), temp)| {
                        (0..6)
                            .filter_map(|i| {
                                let (s, c) = (90f64 - 60f64 * i as f64).to_radians().sin_cos();
                                if (x - 0.55 * c).hypot(y - 0.55 * s) < 0.05 {
                                    Some((i, temp))
                                } else {
                                    let (s, c) = (-60f64 * i as f64).to_radians().sin_cos();
                                    if (x - 0.942 * c).hypot(y - 0.942 * s) < 0.05 {
                                        Some((i + 6, temp))
                                    } else {
                                        None
                                    }
                                }
                            })
                            .collect::<Vec<(i32, f64)>>()
                    })
                    .collect()
            };
            let temp: Vec<_> = (0..12)
                .map(|i| {
                    es_temp
                        .iter()
                        .filter_map(|(j, t)| if i == *j { Some(*t) } else { None })
                        .collect::<Vec<f64>>()
                })
                .collect();
            let temperature_mean: Vec<_> = temp
                .iter()
                .map(|temp| temp.iter().sum::<f64>() / temp.len() as f64)
                .collect();
            Ok((time, temperature_mean))
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
    for data in records.unwrap() {
        wtr.serialize(data).unwrap();
    }
    wtr.flush().unwrap();
    println!("{:<32}: {:>8}s", cfd_case, now.elapsed().as_secs());

    Ok(())
}
