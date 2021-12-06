use parse_monitors::TEMPERATURE_SAMPLING_FREQUENCY;
use rayon::prelude::*;
use std::{path::PathBuf, time::Instant};

const R: f64 = 1.2;

use parse_monitors::cfd;
use polars::prelude::*;

fn refraction_index(temperature: f64) -> f64 {
    let pref = 75000.0; //  Reference pressure [Pa]
    let wlm = 0.5; // wavelength [micron]
    7.76e-7 * pref * (1. + 0.00752 / (wlm * wlm)) / temperature
}

fn main() -> anyhow::Result<()> {
    let duration = 400_usize;
    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    println!("CFD case: {}", cfd_case);
    let now = Instant::now();
    let files: Vec<PathBuf> = cfd::CfdDataFile::TemperatureField
        .glob(cfd_case)?
        .collect::<std::result::Result<Vec<PathBuf>, glob::GlobError>>()?;
    let n_sample = duration * TEMPERATURE_SAMPLING_FREQUENCY as usize;
    let n_skip = if files.len() < n_sample {
        panic!("Not enough data sample ({})", files.len())
    } else {
        files.len() - n_sample
    };
    let data = files
        .into_iter()
        .skip(n_skip)
        .take(100)
        .map(|path| {
            let df = CsvReader::from_path(path)?
                .infer_schema(None)
                .has_header(true)
                .finish()?;
            let df_asm = df.filter(
                &{
                    let x = df.column("X (m)")?;
                    let y = df.column("Y (m)")?;
                    &(x * x) + &(y * y)
                }
                .lt(R * R),
            )?;
            let df_asm = df_asm.filter(&df_asm.column("Z (m)")?.gt(23.99))?;
            let df_asm = df_asm.filter(&df_asm.column("Z (m)")?.lt(24.29))?;
            let ri = df_asm
                .column("Temperature (K)")?
                .f64()?
                .apply(refraction_index);
            Ok((
                ri.len(),
                ri.mean(),
                ri.median(),
                ri.std(),
                ri.max(),
                ri.min(),
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    println!(
        "{} pressure files processed in: {}s",
        data.len(),
        now.elapsed().as_secs()
    );

    Ok(())
}
