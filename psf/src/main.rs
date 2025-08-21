/*!
#  CFD Dome Seeing & Wind Loads PSFs

```shell
export CUDACXX=/usr/local/cuda/bin/nvcc
export FEM_REPO=~/mnt/20250506_1715_zen_30_M1_202110_FSM_202305_Mount_202305_pier_202411_M1_actDamping/
export CFD_REPO=~/maua/CASES/
export GMT_MODES_PATH=~/Dropbox/AWS/CEO/gmtMirrors/
cargo r -r -- --help
```
*/

use std::{env, fs::create_dir_all, iter, path::Path, time::Instant};

use clap::{Parser, ValueEnum};
use crseo::{
    Atmosphere, Builder, FromBuilder, Gmt, Imaging, PSSn, PSSnEstimates, Source,
    imaging::Detector,
    pssn::{PSSnBuilder, TelescopeError},
};
use gmt_dos_clients_domeseeing::DomeSeeing;
use gmt_lom::RigidBodyMotions;
use indicatif::{ProgressBar, ProgressStyle};
use parse_monitors::{
    CFD_YEAR,
    cfd::{Baseline, BaselineTrait, CfdCase},
};
use psf::{Config, DETECTOR_SIZE, PSF, PSFs};
use skyangle::Conversion;

const N_SAMPLE: usize = 100;

#[derive(Debug, Clone, ValueEnum)]
enum Exposure {
    Short,
    Long,
}

#[derive(Debug, Clone, ValueEnum)]
enum ZenithAngle {
    #[value(name = "0")]
    Zero = 0,
    #[value(name = "30")]
    Thirty = 30,
    #[value(name = "60")]
    Sixty = 60,
}

impl From<ZenithAngle> for u32 {
    fn from(zen: ZenithAngle) -> u32 {
        zen as u32
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum AzimuthAngle {
    #[value(name = "0")]
    Zero = 0,
    #[value(name = "45")]
    FortyFive = 45,
    #[value(name = "90")]
    Ninety = 90,
    #[value(name = "135")]
    OneThirtyFive = 135,
    #[value(name = "180")]
    OneEighty = 180,
}

impl From<AzimuthAngle> for u32 {
    fn from(az: AzimuthAngle) -> u32 {
        az as u32
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum WindSpeed {
    #[value(name = "2")]
    Two = 2,
    #[value(name = "7")]
    Seven = 7,
    #[value(name = "12")]
    Twelve = 12,
    #[value(name = "17")]
    Seventeen = 17,
}

impl From<WindSpeed> for u32 {
    fn from(ws: WindSpeed) -> u32 {
        ws as u32
    }
}

#[derive(Parser)]
#[command(name = "psf")]
#[command(about = "Generate PSF frames from GMT CFD dome seeing data")]
struct Args {
    /// Enable dome seeing turbulence effects
    #[arg(long)]
    domeseeing: bool,

    /// Enable wind loads effects
    #[arg(long)]
    windloads: bool,

    /// Zenith angle in degrees (0, 30, or 60)
    #[arg(long, value_enum, default_value_t = ZenithAngle::Thirty)]
    zenith_angle: ZenithAngle,

    /// Azimuth angle in degrees (0, 45, 90, 135, or 180)
    #[arg(long, value_enum, default_value_t = AzimuthAngle::Zero)]
    azimuth_angle: AzimuthAngle,

    /// Wind speed in m/s (2, 7, 12, or 17)
    #[arg(long, value_enum, default_value_t = WindSpeed::Seven)]
    wind_speed: WindSpeed,
}

/// Determine enclosure configuration based on wind speed and zenith angle
fn get_enclosure_config(wind_speed: u32, zenith_angle: u32) -> &'static str {
    if wind_speed <= 7 {
        "os" // open sky for wind <= 7 m/s
    } else if zenith_angle < 60 {
        "cd" // closed dome for wind > 7 m/s and zenith < 60°
    } else {
        "cs" // closed sky for wind > 7 m/s and zenith >= 60°
    }
}

// ray tracing through the GMT
fn ray_trace(
    v_src: &mut Source,
    gmt: &mut Gmt,
    v_pssn: &mut PSSn<TelescopeError>,
    (opd, rbms): (Option<Vec<f64>>, Option<Vec<f64>>),
) {
    // updating M1 & M2 rigid body motions
    if let Some((m1_rbm, m2_rbm)) = rbms
        .as_ref()
        .map(|x| x.split_at(42))
        .map(|(m1_rbm, m2_rbm)| (Some(m1_rbm), Some(m2_rbm)))
    {
        gmt.update42(m1_rbm, m2_rbm, None, None);
    }

    v_src.through(gmt).xpupil();
    // adding dome seeing OPD map to the wavefront
    if let Some(opd) = opd {
        v_src.add(opd.as_slice());
    }
    v_src.through(v_pssn);
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    // Setup GMT optics and imaging
    let mut gmt = Gmt::builder().build()?;
    let v_src = Source::builder().band("Vs");
    let mut v_pssn = PSSnBuilder::<TelescopeError>::default()
        .source(v_src.clone())
        .build()?;
    let mut v_src = v_src.build()?;

    // Get wavelength in nanometers for PSSN display
    let wavelength_nm = v_src.wavelength() * 1e9; // Convert meters to nanometers

    let mut imgr = Imaging::builder()
        .detector(
            Detector::default()
                .n_px_imagelet(DETECTOR_SIZE)
                .n_px_framelet(DETECTOR_SIZE)
                .osf(4),
        )
        .build()?;
    // pixel scale
    let px = imgr.pixel_scale(&v_src).to_mas();
    println!(
        "Detector: pixel scale: {:.0}mas, FOV: {:.2}arcsec",
        px,
        imgr.field_of_view(&v_src).to_arcsec()
    );
    let gmt_diff_lim = (1.22 * v_src.wavelength() / 25.5).to_mas();
    let gmt_segment_diff_lim = (1.22 * v_src.wavelength() / 8.365).to_mas() as f32;
    println!("GMT diffraction limited FWHM: {:.0}mas", gmt_diff_lim);
    println!(
        "GMT segment diffraction limited FWHM: {:.0}mas",
        gmt_segment_diff_lim
    );
    let atm = Atmosphere::builder().build()?;
    let seeing = (0.98 * v_src.wavelength() / atm.r0()).to_mas() as f32;
    println!("Atmosphere seeing: {:.0}mas", seeing);

    // Calculate seeing radius in pixels (diameter = 2 * radius, so radius = seeing / 2 / px)
    let seeing_radius_pixels = (seeing / 2.0) / px;
    // Calculate GMT segment diff lim radius in pixels
    let segment_diff_lim_radius_pixels = (gmt_segment_diff_lim / 2.0) / px;
    // println!("Seeing radius in pixels: {:.1}px", seeing_radius_pixels);
    // println!("GMT segment diff lim radius in pixels: {:.1}px", segment_diff_lim_radius_pixels);
    let config = Config::new(
        seeing_radius_pixels,
        segment_diff_lim_radius_pixels,
        wavelength_nm,
    );

    // Generate reference frame (no turbulence)
    v_src.through(&mut gmt).xpupil().through(&mut imgr);
    let frame0: Vec<f32> = imgr.frame().into();

    // Save reference frame with its own normalization
    PSF::new(&config, frame0).save("psf.png")?;
    println!("Saved frame0 as psf.png");

    // any transients dome seeing or wind loads?
    if !args.domeseeing && !args.windloads {
        return Ok(());
    }

    // CFD case - extract values from arguments
    let zenith = u32::from(args.zenith_angle);
    let azimuth = u32::from(args.azimuth_angle);
    let wind_speed = u32::from(args.wind_speed);
    let enclosure = get_enclosure_config(wind_speed, zenith);

    println!("CFD Configuration:");
    println!("  Zenith angle: {}°", zenith);
    println!("  Azimuth angle: {}°", azimuth);
    println!("  Wind speed: {} m/s", wind_speed);
    println!("  Enclosure: {}", enclosure);

    let cfd_case = CfdCase::<CFD_YEAR>::colloquial(zenith, azimuth, enclosure, wind_speed)?;
    let config = config.cfd_case(cfd_case);
    // let cfd_case_str = cfd_case.to_string();

    // dome seeing
    let ds = if args.domeseeing {
        let cfd_path = Baseline::<CFD_YEAR>::path()?.join(cfd_case.to_string());
        Some(DomeSeeing::builder(&cfd_path).build()?)
    } else {
        None
    };
    // wind loads
    let m12_rbms = if args.windloads {
        let rbms_path = Path::new(&env::var("FEM_REPO")?)
            .join("cfd")
            .join(cfd_case.to_string())
            .join("m1_m2_rbms.parquet");
        println!("loads M1 & M1 RBMs from {rbms_path:?}");
        Some(
            RigidBodyMotions::from_parquet(
                rbms_path,
                Some("M1RigidBodyMotions"),
                Some("M2RigidBodyMotions"),
            )?
            .into_data(),
        )
    } else {
        None
    };

    // Generate turbulence effects string
    let turbulence_effects = match (ds.is_some(), m12_rbms.is_some()) {
        (true, true) => Some("Dome Seeing + Wind Loads"),
        (true, false) => Some("Dome Seeing"),
        (false, true) => Some("Wind Loads"),
        (false, false) => None,
    };
    let config = if let Some(value) = turbulence_effects {
        config.turbulence_effects(value)
    } else {
        config
    };

    // dome seeing OPD iterator `N_SAMPLE` @ 5Hz
    let ds_iter: Box<dyn Iterator<Item = Option<Vec<f64>>>> = if let Some(ds) = ds {
        Box::new(ds.map(|opd| Some(opd)).take(N_SAMPLE))
    } else {
        Box::new(iter::repeat_n(None, N_SAMPLE))
    };
    // M1 & M2 RBMs iterator `N_SAMPLE` @ 5Hz
    // The RBMs are sampled at 1kHz and ramped up from zero
    // reaching steady state after 3s
    // The 1st 5s (5000 samples) are skipped and the RBMs are
    // downsampled by a factor 1000/5=200
    let m12_rbms_iter: Box<dyn Iterator<Item = Option<Vec<f64>>>> =
        if let Some(m12_rbms) = &m12_rbms {
            Box::new(
                m12_rbms
                    .column_iter()
                    .skip(5000)
                    .step_by(200)
                    .map(|x| Some(x.as_slice().to_vec()))
                    .take(N_SAMPLE),
            )
        } else {
            Box::new(iter::repeat_n(None, N_SAMPLE))
        };

    // Setup output directory
    let frames_dir = Path::new("frames");
    create_dir_all(frames_dir)?;

    // Process turbulence-affected frames
    let now = Instant::now();
    // let mut all_frames = Vec::new();
    // let mut all_pssns = Vec::new();
    let mut psfs = PSFs::new(&config);

    // Create progress bar for frame processing
    let process_pb = ProgressBar::new(N_SAMPLE as u64);
    process_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    process_pb.set_message("Processing PSF frames");

    for data in ds_iter.zip(m12_rbms_iter) {
        imgr.reset();
        ray_trace(&mut v_src, &mut gmt, &mut v_pssn, data);
        v_src.through(&mut imgr);
        psfs.push(imgr.frame().into(), v_pssn.estimates()[0]);
        // all_pssns.push(v_pssn.estimates()[0]);
        // all_frames.push();
        process_pb.inc(1);
    }

    process_pb.finish_with_message("PSF processing complete");
    let frame_count = psfs.len();

    // Save all turbulence frames with consistent normalization
    psfs.save_all_frames()?;
    psfs.sum().save("long_exposure_psf.png")?;

    println!();
    println!(
        "✅ Processing completed in {:.2}s",
        now.elapsed().as_secs_f64()
    );
    println!("📁 Saved {} frames to ./frames/ directory", frame_count);
    println!("🖼️  Reference PSF saved as psf.png");
    println!("🖼️  Long exposure PSF saved as long_exposure_psf.png");
    println!();
    println!("🎬 To create an animated GIF at 5Hz, run:");
    println!("   convert -delay 20 -loop 0 frames/frame_*.png psf_animation.gif");

    Ok(())
}
