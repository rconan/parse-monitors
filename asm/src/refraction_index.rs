use parse_monitors::{cfd, TEMPERATURE_SAMPLING_FREQUENCY};
use polars::prelude::*;
use std::path::PathBuf;

use crate::{file_timestamp, Stats};

fn refraction_index(temperature: f64) -> f64 {
    let pref = 75000.0; //  Reference pressure [Pa]
    let wlm = 0.5; // wavelength [micron]
    7.76e-7 * pref * (1. + 0.00752 / (wlm * wlm)) / temperature
}

pub fn stats(
    duration: usize,
    cfd_case: cfd::CfdCase<2021>,
    radius: f64,
) -> anyhow::Result<Vec<Stats>> {
    let files: Vec<PathBuf> = cfd::CfdDataFile::<2021>::TemperatureField
        .glob(cfd_case)?
        .collect::<std::result::Result<Vec<PathBuf>, glob::GlobError>>()?;
    let n_sample = duration * TEMPERATURE_SAMPLING_FREQUENCY as usize;
    let n_skip = if files.len() < n_sample {
        panic!("Not enough data sample ({})", files.len())
    } else {
        files.len() - n_sample
    };
    files
        .into_iter()
        .skip(n_skip)
        .map(|path| {
            let df = CsvReader::from_path(&path)?
                .infer_schema(None)
                .has_header(true)
                .finish()?;
            let df_asm = df.filter(
                &{
                    let x = df.column("X (m)")?;
                    let y = df.column("Y (m)")?;
                    &(x * x) + &(y * y)
                }
                .lt(radius * radius),
            )?;
            let df_asm = df_asm.filter(&df_asm.column("Z (m)")?.gt(23.99))?;
            let df_asm = df_asm.filter(&df_asm.column("Z (m)")?.lt(24.29))?;
            Ok({
                let mut ri = df_asm
                    .column("Temperature (K)")?
                    .f64()?
                    .apply(refraction_index);
                ri.rename(&cfd_case.to_string());
                (
                    file_timestamp(path, &cfd::CfdDataFile::<2021>::M2Pressure.pattern()),
                    ri,
                )
                    .into()
            })
        })
        .collect()
}
