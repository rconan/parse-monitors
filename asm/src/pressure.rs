use crate::{file_timestamp, Stats};
use bzip2::bufread::BzDecoder;
use parse_monitors::{cfd, FORCE_SAMPLING_FREQUENCY};
use polars::prelude::*;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::PathBuf,
    time::Instant,
};

pub fn stats(
    duration: usize,
    cfd_case: cfd::CfdCase<2021>,
    radius: f64,
) -> anyhow::Result<Vec<Stats>> {
    let files: Vec<PathBuf> = cfd::CfdDataFile::<2021>::M2Pressure
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
                        .with_path(Some(path.clone()))
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
                let p = es_df.column("Pressure (Pa)")?.f64()?.to_owned();
                (
                    file_timestamp(path, &cfd::CfdDataFile::<2021>::M2Pressure.pattern()),
                    p,
                )
                    .into()
            })
        })
        .collect()
}

pub fn process(duration: usize, radius: f64) -> anyhow::Result<()> {
    let now = Instant::now();
    let cfd_cases: Vec<cfd::CfdCase<2021>> = cfd::Baseline::<2021>::default().into_iter().collect();
    let pressure = cfd_cases
        .clone()
        .into_iter()
        //.progress_count(cfd_cases.len() as u64)
        .map(|cfd_case| {
            Ok(stats(duration, cfd_case, radius)?
                .into_iter()
                .collect::<Result<DataFrame>>()?
                .column("var")?
                .f64()?
                .mean()
                .map(|x| x.sqrt())
                .unwrap())
        })
        .collect::<anyhow::Result<Vec<f64>>>()?;
    println!(
        "{} CFD cases processed in: {}s",
        &cfd_cases.len(),
        now.elapsed().as_secs()
    );
    let mut cases = cfd_cases
        .into_iter()
        .map(|c| c.to_string())
        .collect::<Series>();
    cases.rename("case");
    let df = DataFrame::new(vec![cases, Series::new("pressure std [Pa]", &pressure)])?;
    print!("{}", df);

    let mut file = File::create("pressure_std.csv")?;
    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&df)?;

    Ok(())
}
