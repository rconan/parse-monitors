//! Pressure maps
//!
//! Plot the pressure maps on M1 and M2 segments

use glob::glob;
use npyz::npz::NpzArchive;
use parse_monitors::cfd;
use rayon::prelude::*;
use std::{error::Error, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let pattern = "optvol_optvol*.npz";

    cfd::Baseline::<2021>::default()
        //.extras()
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>()
        .into_par_iter()
        .for_each(|cfd_case| {
            let now = Instant::now();
            let case_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
            let files: Vec<_> = glob(case_path.join("optvol").join(pattern).to_str().unwrap())
                .unwrap()
                .collect();

            let file = files.last().unwrap();
            let opd: Vec<f64> = NpzArchive::open(file.as_ref().unwrap())
                .map(|mut npz| {
                    npz.by_name("opd")
                        .unwrap()
                        .map(|npy| npy.into_vec::<f64>().unwrap())
                })
                .unwrap()
                .unwrap()
                .into_iter()
                .map(|x| x * 1e6)
                .collect();

            let path = case_path.join("report").join("opd_map.png");
            let filename = format!("{}", path.as_path().display());
            let _: complot::Heatmap = (
                (opd.as_slice(), (512, 512)),
                complot::complot!(filename, xlabel = "WFE [micron]"),
            )
                .into();

            println!("{:?}: {:>8}s", file, now.elapsed().as_secs());
        });
    Ok(())
}
