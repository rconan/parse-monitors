use asm::{pressure, refraction_index};
use indicatif::ParallelProgressIterator;
use parse_monitors::cfd;
use polars::prelude::*;
use rayon::prelude::*;
use std::{fs::File, time::Instant};

const R: f64 = 1.2;

fn main() -> anyhow::Result<()> {
    let duration = 400_usize;
    //let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    //println!("CFD case: {}", cfd_case);
    let now = Instant::now();
    let cfd_cases: Vec<cfd::CfdCase<2021>> = cfd::Baseline::<2021>::default().into_iter().collect();
    let pressure = cfd_cases
        .clone()
        .into_par_iter()
        .progress_count(cfd_cases.len() as u64)
        .map(|cfd_case| {
            Ok(pressure::stats(duration, cfd_case, R)?
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
    let df = DataFrame::new(vec![cases, Series::new("pressure var [Pa]", &pressure)])?;
    print!("{}", df);

    let mut file = File::create("pressure_std.csv")?;
    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&df)?;

    Ok(())
}
