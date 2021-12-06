use crate::Stats;
use bzip2::bufread::BzDecoder;
use parse_monitors::{cfd, FORCE_SAMPLING_FREQUENCY};
use polars::prelude::*;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::PathBuf,
};

pub fn stats(
    duration: usize,
    cfd_case: cfd::CfdCase<2021>,
    radius: f64,
) -> anyhow::Result<Vec<Stats>> {
    let files: Vec<PathBuf> = cfd::CfdDataFile::M2Pressure
        .glob(cfd_case)?
        .collect::<std::result::Result<Vec<PathBuf>, glob::GlobError>>()?;
    let n_sample = duration * FORCE_SAMPLING_FREQUENCY as usize;
    let n_skip = if files.len() < n_sample {
        panic!("Not enough data sample")
    } else {
        files.len() - n_sample
    };
    files
        .into_iter()
        .skip(n_skip)
        .map(|path| {
            let es_df = {
                let df = {
                    let csv_file = File::open(&path)?;
                    let mut contents = String::new();
                    BzDecoder::new(BufReader::new(csv_file)).read_to_string(&mut contents)?;
                    CsvReader::new(Cursor::new(contents.as_bytes()))
                        .with_path(Some(path))
                        .infer_schema(None)
                        .has_header(true)
                        .finish()?
                };
                df.filter(
                    &{
                        let x = df.column("X (m)")?;
                        let y = df.column("Y (m)")?;
                        &(x * x) + &(y * y)
                    }
                    .lt(radius * radius),
                )?
            };
            Ok({
                let mut pa = es_df.column("Pressure (Pa)")?.f64()?.to_owned();
                pa.rename(&cfd_case.to_string());
                pa.into()
            })
        })
        .collect()
}
