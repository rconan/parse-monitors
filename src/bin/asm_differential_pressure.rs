//! Pressure maps
//!
//! Plot the pressure maps on M1 and M2 segments

use linya::{Bar, Progress};
use matio_rs::{Field, MatFile, MatStruct, MatStructBuilder, Save};
use parse_monitors::{cfd, pressure::Pressure};
use rayon::prelude::*;
use std::{path::Path, sync::Mutex};

trait Config {
    fn configure(cfd_case: cfd::CfdCase<2021>) -> anyhow::Result<(String, Vec<String>)>;
}
impl Config for geotrans::M1 {
    fn configure(cfd_case: cfd::CfdCase<2021>) -> anyhow::Result<(String, Vec<String>)> {
        Ok((
            "M1p.csv.z".to_string(),
            cfd::CfdDataFile::<2021>::M1Pressure
                .glob(cfd_case)?
                .map(|p| p.unwrap().to_str().unwrap().to_string())
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
                .map(|p| p.unwrap().to_str().unwrap().to_string())
                .collect(),
        ))
    }
}

fn main() -> anyhow::Result<()> {
    type M12 = geotrans::M2;

    let progress = Mutex::new(Progress::new());

    cfd::Baseline::<2021>::mount()
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>()
        .into_par_iter()
        .for_each(|cfd_case| {
            let case_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
            let (_geometry, files) = M12::configure(cfd_case).unwrap();

            let bar: Bar = progress.lock().unwrap().bar(8001, format!("{}", cfd_case));

            let (time_stamps, (mean_press, diff_press)): (
                Vec<f64>,
                (Vec<Vec<f64>>, Vec<Vec<Vec<f64>>>),
            ) = files
                .iter()
                .map(|file| {
                    progress.lock().unwrap().inc_and_draw(&bar, 1);
                    let path = Path::new(file);
                    let time_stamp = path
                        .file_name()
                        .and_then(|f| Path::new(f).file_stem())
                        .and_then(|f| Path::new(f).file_stem())
                        .and_then(|f| f.to_str())
                        .and_then(|s| s.split('_').last())
                        .and_then(|s| s.parse::<f64>().ok())
                        .expect("Failed to parse file time stamp");
                    let csv_pressure = Pressure::<M12>::decompress(path.to_path_buf()).unwrap();
                    //let csv_geometry =
                    //    Pressure::<M12>::decompress(path.with_file_name(geometry)).unwrap();
                    let pressures = Pressure::<M12>::load(csv_pressure).unwrap();
                    let (mean_press, diff_press): (Vec<f64>, Vec<Vec<f64>>) = (1..=7)
                        .map(|sid| pressures.asm_differential_pressure(sid))
                        .unzip();
                    (time_stamp, (mean_press, diff_press))
                })
                .unzip();

            let mut mat = MatStruct::new("ams_differential_pressure");
            mat = <MatStructBuilder as matio_rs::FieldIterator<f64>>::field(
                mat,
                "time",
                time_stamps.iter(),
            )
            .expect("failed to convert timestamps to MatVar");
            for k in 0..7 {
                let segment: Vec<MatStruct> = mean_press
                    .iter()
                    .zip(&diff_press)
                    .map(|(m, dp)| {
                        MatStruct::new(format!("S{}", k + 1))
                            .field("mean_pressure", &m[k])
                            .expect("failed to convert mean pressure to MatVar")
                            .field("diff_pressure", &dp[k])
                            .expect("failed to convert differential pressure to MatVar")
                            .build()
                            .expect(&format!("failed to build Matlab segment # {k} structure"))
                    })
                    .collect();
                mat = <MatStructBuilder as matio_rs::FieldMatObjectIterator<MatStruct>>::field(
                    mat,
                    format!("S{}", k + 1),
                    segment.into_iter(),
                )
                .expect("failed to convert segment to MatStruct");
            }
            let mat_file = MatFile::save(case_path.join("asm_differential_pressure.mat"))
                .expect("failed to create mat file");
            mat_file.write(mat.build().expect("failed to build Matlab structure"));
        });
    Ok(())
}
