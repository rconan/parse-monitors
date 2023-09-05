use std::{
    env,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use parse_monitors::{
    cfd::{CfdCase, CfdDataFile},
    pressure::Pressure,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use rayon::prelude::*;

/// Segment average pressure time series
#[derive(Default, Debug, Clone)]
pub struct SegmentAveragePressure {
    // segment-average pressure
    pressure: Vec<f64>,
    // segment-average dynamic pressure
    dynamic_pressure: Vec<f64>,
}
impl SegmentAveragePressure {
    pub fn mean_dynamic_pressure(&self) -> f64 {
        let n = self.dynamic_pressure.len() as f64;
        self.dynamic_pressure.iter().sum::<f64>() / n
    }
    pub fn mean_pressure(&self) -> f64 {
        let n = self.pressure.len() as f64;
        self.pressure.iter().sum::<f64>() / n
    }
}
impl Deref for SegmentAveragePressure {
    type Target = Vec<f64>;

    fn deref(&self) -> &Self::Target {
        &self.dynamic_pressure
    }
}
impl DerefMut for SegmentAveragePressure {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dynamic_pressure
    }
}

/// Collection of 7 M1 segments average pressure
#[derive(Default, Debug)]
pub struct M1Segments {
    // M1 7 segments-average pressure
    pressure: Vec<f64>,
    // Segments average pressure time series
    segments: Vec<SegmentAveragePressure>,
}
/*
impl Deref for M1Segments {
    type Target = Vec<SegmentAveragePressure>;

    fn deref(&self) -> &Self::Target {
        &self.segments
    }
}
impl DerefMut for M1Segments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.segments
    }
}
 */
impl M1Segments {
    pub fn new() -> Self {
        Self {
            pressure: vec![],
            segments: vec![Default::default(); 7],
        }
    }
    pub fn mean_pressure(&self) -> f64 {
        let n = self.pressure.len() as f64;
        self.pressure.iter().sum::<f64>() / n
    }
    pub fn segments_mean_pressure(&self) -> Vec<f64> {
        self.segments
            .iter()
            .map(|segment| segment.mean_pressure())
            .collect()
    }
    pub fn segments_mean_dynamic_pressure(&self) -> Vec<f64> {
        self.segments
            .iter()
            .map(|segment| segment.mean_dynamic_pressure())
            .collect()
    }
    pub fn push(&mut self, pressure: &mut Pressure<geotrans::M1>) {
        self.pressure.push(pressure.mirror_average_pressure());
        pressure
            .segments_average_pressure()
            .into_iter()
            .zip(pressure.segments_average_dynamic_pressure().into_iter())
            .zip(self.segments.iter_mut())
            .for_each(|((p, dyn_p), segment)| {
                segment.pressure.push(p);
                segment.dynamic_pressure.push(dyn_p);
            });
    }
}

impl Display for M1Segments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "M1 average pressure: {:.3}", self.mean_pressure())?;
        writeln!(
            f,
            "M1 segments average pressure: {:.3?}",
            self.segments_mean_pressure()
        )?;
        writeln!(
            f,
            "M1 segments average dynamic pressure: {:.3?}",
            self.segments_mean_dynamic_pressure()
        )
    }
}

fn task(cfd_case: &CfdCase<2021>, bar: ProgressBar) -> anyhow::Result<M1Segments> {
    let files: Vec<std::path::PathBuf> = CfdDataFile::<2021>::M1Pressure.glob(cfd_case.clone())?;

    /*     let mut pressure = Pressure::<geotrans::M1>::load(Pressure::<geotrans::M1>::decompress(
        files.last().cloned().unwrap(),
    )?)?;

    println!("{:.3}", pressure.mirror_average_pressure());
    println!("{:.3?}", pressure.segments_average_pressure());
    println!("{:.3?}", pressure.segments_average_dynamic_pressure()); */

    let mut m1_average_pressure = M1Segments::new();
    let n = files.len() - N_SAMPLE;
    for file in files.into_iter().skip(n) {
        bar.inc(1);
        let mut pressure =
            Pressure::<geotrans::M1>::load(Pressure::<geotrans::M1>::decompress(file)?)?;
        m1_average_pressure.push(&mut pressure);
    }
    bar.finish();
    Ok(m1_average_pressure)
}

const N_SAMPLE: usize = 1000;

fn main() -> anyhow::Result<()> {
    println!(
        "CFD REPO: {}",
        env::var("CFD_REPO").expect("CFD_REPO not set")
    );

    let cfd_cases: Vec<CfdCase<2021>> = vec![
        CfdCase::colloquial(0, 0, "os", 2)?,
        CfdCase::colloquial(0, 0, "os", 7)?,
        CfdCase::colloquial(0, 0, "cd", 12)?,
        CfdCase::colloquial(0, 0, "cd", 17)?,
    ];

    let m = MultiProgress::new();
    let sty =
        ProgressStyle::with_template("[{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-");

    let pbs: Vec<_> = cfd_cases
        .iter()
        .map(|_| {
            let pb = m.add(ProgressBar::new(N_SAMPLE as u64));
            pb.set_style(sty.clone());
            pb
        })
        .collect();

    let results: anyhow::Result<Vec<_>> = cfd_cases
        .par_iter()
        .zip(pbs.into_par_iter())
        .map(|(cfd_case, pb)| task(cfd_case, pb))
        .collect();

    cfd_cases
        .into_iter()
        .zip(results.unwrap().into_iter())
        .for_each(|(cfd_case, m1_mean_average_pressure)| {
            println!("{}: {:}", cfd_case, m1_mean_average_pressure)
        });

    Ok(())
}
