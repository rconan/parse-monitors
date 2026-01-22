use clap::{Parser, ValueEnum};
use crseo::{
    Atmosphere, Builder, FromBuilder, Fwhm, Gmt, PSSn, Source,
    builders::AtmosphereBuilder,
    pssn::{AtmosphereTelescopeError as Terr, PSSnBuilder},
};
use gmt_dos_clients_domeseeing::DomeSeeing;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use object_store::{ObjectStore, path::Path as ObjectPath};
use parse_monitors::{
    CFD_YEAR,
    cfd::{Baseline, BaselineTrait, CfdCase},
};
use serde::{Deserialize, Serialize};
use skyangle::Conversion;

use std::{env, fmt::Display, fs::File, io::BufWriter, sync::Arc, time::Instant};

#[derive(Clone, Parser)]
struct Args {
    /// Turbulence model
    #[arg(short,long,value_enum,default_value_t = Turbulence::DomeSeeingAtmosphere)]
    turbulence: Turbulence,
    /// Atmosphere or dome seeing sample size
    #[arg(short, long)]
    n_sample: Option<usize>,
    /// CFD case index
    #[arg(short, long)]
    cfd_case_id: Option<usize>,
    /// Display FWHM values
    #[arg(short, long)]
    verbose: bool,
    /// Do not save data to file
    #[arg(long)]
    no_save: bool,
}
#[derive(Clone, ValueEnum)]
pub enum Turbulence {
    Atmosphere,
    DomeSeeing,
    GroundLayerAtmosphere,
    DomeSeeingAtmosphere,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    dotenvy::dotenv()?;

    match args.turbulence {
        Turbulence::Atmosphere | Turbulence::GroundLayerAtmosphere => {
            let mut gmt = Gmt::builder().build()?;
            let v_src = Source::builder().band("Vs");

            let mut atmb = Atmosphere::builder();
            if let Turbulence::GroundLayerAtmosphere = args.turbulence {
                atmb = atmb
                    .remove_turbulence_layer(1)
                    .remove_turbulence_layer(1)
                    .remove_turbulence_layer(1)
                    .remove_turbulence_layer(1)
                    .remove_turbulence_layer(1)
                    .remove_turbulence_layer(1);
            } else {
                atmb = atmb.remove_turbulence_layer(0);
            }

            let mut v_pssn = PSSnBuilder::<Terr>::from(atmb.clone())
                .source(v_src.clone())
                .build()?;
            let mut v_src = v_src.build()?;

            let mut fwhm = Fwhm::new();
            fwhm.build(&mut v_src);

            let mut atm = atmb.build()?;

            let now = Instant::now();
            let n_sample = args.n_sample.unwrap_or(2000);
            let pb = ProgressBar::new(n_sample as u64); //ds.len() as u64);
            pb.set_style(
                ProgressStyle::with_template("[{eta_precise}] {bar:50.cyan/blue} {pos:>5}/{len:5}")
                    .unwrap(),
            );

            for _ in 0..n_sample {
                pb.inc(1);
                v_src
                    .through(&mut gmt)
                    .xpupil()
                    .through(&mut atm)
                    .through(&mut v_pssn);
                atm.reset();
            }
            pb.finish();
            let filename = "atmosphere_fwhm.pkl".to_string();
            let fwhm_data = FwhmData::new(fwhm, v_pssn);
            if args.verbose {
                println!("{} in {}s", fwhm_data, now.elapsed().as_secs());
            }
            if !args.no_save {
                let mut buffer = BufWriter::new(File::create(filename)?);
                serde_pickle::to_writer(&mut buffer, &fwhm_data, Default::default())?;
            }
        }
        Turbulence::DomeSeeing | Turbulence::DomeSeeingAtmosphere => {
            let store_prime: Arc<dyn ObjectStore> = Arc::new(
                object_store::aws::AmazonS3Builder::from_env()
                    .with_region(env::var("REGION")?)
                    .with_bucket_name(env::var("BUCKET")?)
                    .build()?,
            );

            // let mut tasks = Vec::with_capacity(60);
            let multi_prime = MultiProgress::new();

            let cfd_cases: Box<dyn Iterator<Item = (usize, CfdCase<CFD_YEAR>)>> =
                if let Some(id) = args.cfd_case_id {
                    let cfd_case = Baseline::<CFD_YEAR>::default()
                        .into_iter()
                        .nth(id)
                        .expect("found CFD id={id}, export id within range [0,60[");
                    Box::new(std::iter::once(cfd_case).enumerate())
                } else {
                    Box::new(Baseline::<CFD_YEAR>::default().into_iter().enumerate())
                };

            for (_i, cfd_case) in cfd_cases {
                let store = store_prime.clone();
                let multi = multi_prime.clone();
                let Args {
                    turbulence,
                    n_sample,
                    verbose,
                    no_save,
                    ..
                } = args.clone();

                async move {
                    // crseo::set_gpu(i as i32 % 8i32);
                    let mut gmt = Gmt::builder().build()?;
                    let v_src = Source::builder().band("Vs");
                    let mut v_pssn = PSSnBuilder::<Terr>::from(Atmosphere::builder())
                        .source(v_src.clone())
                        .build()?;
                    let mut v_src = v_src.build()?;

                    let mut fwhm = Fwhm::new();
                    fwhm.build(&mut v_src);

                    // let now = Instant::now();

                    // let cfd_case = CfdCase::<CFD_YEAR>::colloquial(30, 0, "OS", 7)?;
                    let cfd_path = ObjectPath::from(
                        Baseline::<CFD_YEAR>::path()?
                            .join(cfd_case.to_string())
                            .to_str()
                            .unwrap(),
                    );
                    // println!("CFD case: {}", cfd_path);

                    let mut dsb = DomeSeeing::builder(cfd_path);
                    if let Some(n_sample) = n_sample {
                        dsb = dsb.sample_size(n_sample);
                    }
                    let mut ds = dsb.store(store).build().await?;
                    // println!("CFD dome seeing sample size: {}", ds.len());

                    let pb =
                        multi.add(ProgressBar::new(n_sample.unwrap_or_else(|| ds.len()) as u64));
                    pb.set_style(
                        ProgressStyle::with_template(&format!(
                            "[{} {{eta_precise}}] {{bar:50.cyan/blue}} {{pos:>5}}/{{len:5}}",
                            cfd_case
                        ))
                        .unwrap(),
                    );

                    let now = Instant::now();

                    if let Turbulence::DomeSeeing = turbulence {
                        while let Some(opd) = ds.async_next().await {
                            pb.inc(1);
                            v_src
                                .through(&mut gmt)
                                .xpupil()
                                .add(opd.as_slice())
                                .through(&mut v_pssn);
                        }
                    } else {
                        let mut atm = Atmosphere::builder().remove_turbulence_layer(0).build()?;
                        while let Some(opd) = ds.async_next().await {
                            pb.inc(1);
                            v_src
                                .through(&mut gmt)
                                .xpupil()
                                .through(&mut atm)
                                .add(opd.as_slice())
                                .through(&mut v_pssn);
                            atm.reset();
                        }
                    } // pb.finish();
                    let filename = format!("{}_fwhm.pkl", cfd_case);
                    let fwhm_data = FwhmData::new(fwhm, v_pssn);
                    if verbose {
                        println!("{} in {}s", fwhm_data, now.elapsed().as_secs());
                    }
                    if !no_save {
                        let mut buffer = BufWriter::new(File::create(filename)?);
                        serde_pickle::to_writer(&mut buffer, &fwhm_data, Default::default())?;
                    }
                    Result::<_, anyhow::Error>::Ok(pb)
                }
                .await?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FwhmData {
    atmosphere_otf: Vec<f32>,
    telescope_error_otf: Vec<f32>,
    atm_fwhm_x0: f64,
    atm_fwhm_x1: f64,
    atm_fwhm_n: f64,
}

impl FwhmData {
    pub fn new(mut fwhm: Fwhm, mut v_pssn: PSSn<Terr>) -> Self {
        let atm_fwhm_x0 =
            Fwhm::atmosphere(500e-9, v_pssn.r0() as f64, v_pssn.oscale as f64).to_mas();
        let atmosphere_otf = v_pssn.atmosphere_otf();
        let atm_fwhm_x1 = fwhm.from_complex_otf(&atmosphere_otf)[0].to_mas();
        let telescope_error_otf = v_pssn.telescope_error_otf();
        let atm_fwhm_n = fwhm.from_complex_otf(&telescope_error_otf)[0].to_mas();
        Self {
            atmosphere_otf,
            telescope_error_otf,
            atm_fwhm_x0,
            atm_fwhm_x1,
            atm_fwhm_n,
        }
    }
}

impl Display for FwhmData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Atm. FWHM [mas]: {:.3}/{:.3}/{:.3}",
            self.atm_fwhm_x0, self.atm_fwhm_x1, self.atm_fwhm_n,
        )
    }
}

// #[derive(Debug)]
// pub struct TaskError {
//     err: Box<dyn Error>,
//     pb: ProgressBar,
// }
// impl Display for TaskError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "FWHM task faile due to {}", self.err)
//     }
// }
// impl Error for TaskError {}
