use std::{env, fs::File, path::Path, time::Instant};

use clap::Parser;
use crseo::{Builder, FromBuilder, Gmt, Source};
use gmt_dos_clients_domeseeing::DomeSeeing;
use indicatif::ProgressBar;
use parse_monitors::{
    CFD_YEAR,
    cfd::{self, CfdCase},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(
    name = "DOME SEEING",
    about = "Computes PSSn from dome seeing OPD maps"
)]
struct Cli {
    /// skip that many CFD cases
    #[arg(short, long)]
    skip: Option<usize>,
    /// process only that many CFD cases
    #[arg(short, long)]
    take: Option<usize>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match (cli.skip, cli.take) {
        (None, None) => Box::new(cfd::Baseline::<CFD_YEAR>::default().into_iter())
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (None, Some(t)) => Box::new(cfd::Baseline::<CFD_YEAR>::default().into_iter().take(t))
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (Some(s), None) => Box::new(cfd::Baseline::<CFD_YEAR>::default().into_iter().skip(s))
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (Some(s), Some(t)) => Box::new(
            cfd::Baseline::<CFD_YEAR>::default()
                .into_iter()
                .skip(s)
                .take(t),
        ) as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
    }
    .map(|cfd_case| {
        Path::new(&env::var("CFD_REPO").expect("CFD_REPO is not set")).join(&cfd_case.to_string())
    })
    .for_each(|path| task(&path).expect(&format!("{path:?} failed")));

    Ok(())
}
fn task(cfd_path: &Path) -> anyhow::Result<()> {
    println!("{cfd_path:?}");

    let mut ds = DomeSeeing::builder(&cfd_path).build()?;

    let mut gmt = Gmt::builder().build()?;
    let mut src = Source::builder().build()?;
    src.through(&mut gmt).xpupil();

    let now = Instant::now();
    let mut time = 0f64;
    let n_radial_order = 10;
    let n_xy = 512;
    let mask: Vec<f64> = src
        .amplitude()
        .into_iter()
        .map(|x| if x == 0f32 { f64::NAN } else { 1f64 })
        .collect();
    let mut zern_opd = ZernikeOpd::new(n_radial_order, &mask);
    let pb = ProgressBar::new(ds.len() as _);
    while let Some(opd) = ds.next() {
        let c = zernike::projection_on_mask(&opd, n_radial_order, n_xy, &mask);
        zern_opd.push(ZernikeCoefficients { time, c });
        time += 0.2;
        pb.inc(1);
    }
    pb.finish();
    println!(
        "Record[{}] completed in {}s",
        zern_opd.len(),
        now.elapsed().as_secs()
    );

    serde_pickle::to_writer(
        &mut File::create(cfd_path.join("domeseeing_zernikes.pkl"))?,
        &zern_opd,
        Default::default(),
    )?;
    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ZernikeCoefficients {
    time: f64,
    c: Vec<f64>,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ZernikeOpd {
    coefficients: Vec<ZernikeCoefficients>,
    n_radial_order: u32,
    mask: Vec<f64>,
}
impl ZernikeOpd {
    pub fn new(n_radial_order: u32, mask: &[f64]) -> Self {
        Self {
            n_radial_order,
            mask: mask.to_vec(),
            ..Default::default()
        }
    }
    pub fn push(&mut self, c: ZernikeCoefficients) {
        self.coefficients.push(c)
    }
    pub fn len(&self) -> usize {
        self.coefficients.len()
    }
}
