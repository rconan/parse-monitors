use std::{env, fs::create_dir_all, iter, path::Path, time::Instant};

use clap::{Parser, ValueEnum};
use colorous;
use crseo::{
    Atmosphere, Builder, FromBuilder, Gmt, Imaging, PSSn, PSSnEstimates, Source,
    imaging::Detector,
    pssn::{PSSnBuilder, TelescopeError},
};
use gmt_dos_clients_domeseeing::DomeSeeing;
use gmt_lom::RigidBodyMotions;
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_hollow_circle_mut, draw_text_mut};
use indicatif::{ProgressBar, ProgressStyle};
use parse_monitors::{
    CFD_YEAR,
    cfd::{Baseline, BaselineTrait, CfdCase},
};
use rusttype::{Font, Scale};
use skyangle::Conversion;

const N_SAMPLE: usize = 100;
const DETECTOR_SIZE: usize = 800;

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

/// Draw PSSN text overlay in the top left corner of the image
fn draw_pssn_text(
    image: &mut RgbImage,
    pssn_value: f64,
    wavelength_nm: f64,
    frame_number: Option<usize>,
    cfd_case: Option<&str>,
    turbulence_effects: Option<&str>,
) -> anyhow::Result<()> {
    // Use system default font (typically DejaVu Sans on Linux)
    let font_data: &[u8] = include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
    let font =
        Font::try_from_bytes(font_data).ok_or_else(|| anyhow::anyhow!("Failed to load font"))?;

    let scale = Scale::uniform(24.0); // 24 pixel font
    let white = Rgb([255u8, 255u8, 255u8]);

    // Position in top left corner with some padding
    let x = 5i32;
    let mut y = 5i32;

    // Draw CFD case if provided
    if let Some(case) = cfd_case {
        let cfd_text = format!("CFD: {}", case);
        draw_text_mut(image, white, x, y, scale, &font, &cfd_text);
        y += 30;
    }

    // Draw turbulence effects if provided
    if let Some(effects) = turbulence_effects {
        let effects_text = format!("Effects: {}", effects);
        draw_text_mut(image, white, x, y, scale, &font, &effects_text);
        y += 30;
    }

    // Draw PSSN text
    let pssn_text = format!("PSSN@{:.0}nm: {:.5}", wavelength_nm, pssn_value);
    draw_text_mut(image, white, x, y, scale, &font, &pssn_text);

    // Draw frame number if provided
    if let Some(frame_num) = frame_number {
        y += 30; // Move down for next line
        let frame_text = format!("frame {:03}", frame_num);
        draw_text_mut(image, white, x, y, scale, &font, &frame_text);
    }

    Ok(())
}

/// Normalize frame data to 0.0-1.0 range and apply CUBEHELIX colormap
fn frame_to_rgb(frame: &[f32], min_val: f32, max_val: f32) -> Vec<u8> {
    let range = max_val - min_val;
    let normalized: Vec<f64> = if range > 0.0 {
        frame
            .iter()
            .map(|&x| ((x - min_val) / range) as f64)
            .collect()
    } else {
        vec![0.5f64; frame.len()]
    };

    normalized
        .iter()
        .flat_map(|&value| {
            let color = colorous::CUBEHELIX.eval_continuous(value);
            [color.r, color.g, color.b]
        })
        .collect()
}

/// Save a single frame as a PNG image with CUBEHELIX colormap, circle overlays, and PSSN text
fn save_frame_as_png(
    frame: &[f32],
    filename: &str,
    min_val: f32,
    max_val: f32,
    seeing_radius_pixels: Option<f32>,
    segment_diff_lim_radius_pixels: Option<f32>,
    pssn_value: Option<f64>,
    wavelength_nm: Option<f64>,
    frame_number: Option<usize>,
    cfd_case: Option<&str>,
    turbulence_effects: Option<&str>,
) -> anyhow::Result<()> {
    let rgb_data = frame_to_rgb(frame, min_val, max_val);
    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        DETECTOR_SIZE as u32,
        DETECTOR_SIZE as u32,
        rgb_data,
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

    let center = (DETECTOR_SIZE as i32 / 2, DETECTOR_SIZE as i32 / 2);

    // Draw seeing circle (hollow) if radius is provided
    if let Some(radius) = seeing_radius_pixels {
        let white = Rgb([255u8, 255u8, 255u8]);
        draw_hollow_circle_mut(&mut image, center, radius as i32, white);
    }

    // Draw GMT segment diffraction limit circle (hollow) if radius is provided
    if let Some(radius) = segment_diff_lim_radius_pixels {
        let white = Rgb([255u8, 255u8, 255u8]);
        draw_hollow_circle_mut(&mut image, center, radius as i32, white);
    }

    // Draw PSSN text if values are provided
    if let (Some(pssn), Some(wl)) = (pssn_value, wavelength_nm) {
        draw_pssn_text(
            &mut image,
            pssn,
            wl,
            frame_number,
            cfd_case,
            turbulence_effects,
        )?;
    }

    image.save(filename)?;
    Ok(())
}

/// Find global min and max values across all frames
fn find_global_extrema(frames: &[Vec<f32>]) -> (f32, f32) {
    let global_max = frames
        .iter()
        .flat_map(|frame| frame.iter())
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let global_min = frames
        .iter()
        .flat_map(|frame| frame.iter())
        .copied()
        .fold(f32::INFINITY, f32::min);
    (global_min, global_max)
}

/// Process all frames and save them as PNG images
fn save_all_frames(
    frames: &[Vec<f32>],
    frames_dir: &Path,
    seeing_radius_pixels: Option<f32>,
    segment_diff_lim_radius_pixels: Option<f32>,
    pssn_values: &[f64],
    wavelength_nm: f64,
    cfd_case: Option<&str>,
    turbulence_effects: Option<&str>,
) -> anyhow::Result<()> {
    let (global_min, global_max) = find_global_extrema(frames);

    // Create progress bar for saving frames
    let save_pb = ProgressBar::new(frames.len() as u64);
    save_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    save_pb.set_message("Saving frames");

    for (i, frame) in frames.iter().enumerate() {
        let filename = frames_dir.join(format!("frame_{:06}.png", i));
        let pssn_value = pssn_values.get(i).copied();
        save_frame_as_png(
            frame,
            filename.to_str().unwrap(),
            global_min,
            global_max,
            seeing_radius_pixels,
            segment_diff_lim_radius_pixels,
            pssn_value,
            Some(wavelength_nm),
            Some(i), // Pass frame number for animated frames
            cfd_case,
            turbulence_effects,
        )?;
        save_pb.inc(1);
    }

    save_pb.finish_with_message("All frames saved");
    Ok(())
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
    let gmt_segment_diff_lim = (1.22 * v_src.wavelength() / 8.365).to_mas();
    println!("GMT diffraction limited FWHM: {:.0}mas", gmt_diff_lim);
    println!(
        "GMT segment diffraction limited FWHM: {:.0}mas",
        gmt_segment_diff_lim
    );
    let atm = Atmosphere::builder().build()?;
    let seeing = (0.98 * v_src.wavelength() / atm.r0()).to_mas();
    println!("Atmosphere seeing: {:.0}mas", seeing);

    // Calculate seeing radius in pixels (diameter = 2 * radius, so radius = seeing / 2 / px)
    let seeing_radius_pixels = (seeing / 2.0) / px as f64;
    // Calculate GMT segment diff lim radius in pixels
    let segment_diff_lim_radius_pixels = (gmt_segment_diff_lim / 2.0) / px as f64;
    // println!("Seeing radius in pixels: {:.1}px", seeing_radius_pixels);
    // println!("GMT segment diff lim radius in pixels: {:.1}px", segment_diff_lim_radius_pixels);

    // Generate reference frame (no turbulence)
    v_src.through(&mut gmt).xpupil().through(&mut imgr);
    let frame0: Vec<f32> = imgr.frame().into();

    // Save reference frame with its own normalization
    let (frame0_min, frame0_max) = find_global_extrema(&[frame0.clone()]);
    save_frame_as_png(
        &frame0,
        "psf.png",
        frame0_min,
        frame0_max,
        Some(seeing_radius_pixels as f32),
        Some(segment_diff_lim_radius_pixels as f32),
        None, // No PSSN for reference frame
        None,
        None, // No frame number for reference frame
        None, // No CFD case for reference frame
        None, // No turbulence effects for reference frame
    )?;
    println!("Saved frame0 as psf.png");

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
    let cfd_case_str = cfd_case.to_string();

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

    // any transients dome seeing or wind loads?
    if ds.is_none() && m12_rbms.is_none() {
        return Ok(());
    }

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
    let mut all_frames = Vec::new();
    let mut all_pssns = Vec::new();

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
        all_pssns.push(v_pssn.estimates()[0]);
        all_frames.push(imgr.frame().into());
        process_pb.inc(1);
    }

    process_pb.finish_with_message("PSF processing complete");
    let frame_count = all_frames.len();

    // Save all turbulence frames with consistent normalization
    save_all_frames(
        &all_frames,
        frames_dir,
        Some(seeing_radius_pixels as f32),
        Some(segment_diff_lim_radius_pixels as f32),
        &all_pssns,
        wavelength_nm,
        Some(&cfd_case_str),
        turbulence_effects,
    )?;

    let summed_frame = all_frames
        .into_iter()
        .fold(vec![0f32; DETECTOR_SIZE.pow(2)], |mut s, f| {
            s.iter_mut().zip(f.into_iter()).for_each(|(s, f)| {
                *s += f;
            });
            s
        });
    let (frame_min, frame_max) = find_global_extrema(&[summed_frame.clone()]);
    save_frame_as_png(
        &summed_frame,
        "long_exposure_psf.png",
        frame_min,
        frame_max,
        Some(seeing_radius_pixels as f32),
        Some(segment_diff_lim_radius_pixels as f32),
        all_pssns.last().copied(),
        Some(wavelength_nm),
        None, // No frame number for long exposure
        Some(&cfd_case_str),
        turbulence_effects,
    )?;

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
