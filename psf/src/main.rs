use std::{env, fs::create_dir_all, iter, path::Path, time::Instant};

use clap::{Parser, ValueEnum};
use colorous;
use crseo::{Atmosphere, Builder, FromBuilder, Gmt, Imaging, Source, imaging::Detector};
use gmt_dos_clients_domeseeing::DomeSeeing;
use gmt_lom::RigidBodyMotions;
use image::{ImageBuffer, Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use parse_monitors::{
    CFD_YEAR,
    cfd::{Baseline, BaselineTrait, CfdCase},
};
use skyangle::Conversion;

const N_SAMPLE: usize = 100;
const DETECTOR_SIZE: usize = 1000;

#[derive(Debug, Clone, ValueEnum)]
enum Exposure {
    Short,
    Long,
}

#[derive(Parser)]
#[command(name = "psf")]
#[command(about = "Generate PSF frames from GMT CFD dome seeing data")]
struct Args {
    /// Exposure type: short or long
    #[arg(long, value_enum, default_value_t = Exposure::Short)]
    exposure: Exposure,

    /// Enable dome seeing turbulence effects
    #[arg(long)]
    domeseeing: bool,

    /// Enable wind loads effects
    #[arg(long)]
    windloads: bool,

    /// Enable atmospheric turbulence effects
    #[arg(long)]
    atmosphere: bool,
}

/// Draw a dashed circle on an RGB image with 50% transparency
fn draw_dashed_seeing_circle(image: &mut RgbImage, center: (i32, i32), radius: i32) {
    let white = Rgb([255u8, 255u8, 255u8]);
    let dash_length = 8;
    let gap_length = 6;

    // Calculate circle circumference and number of dashes
    let circumference = (2.0 * std::f32::consts::PI * radius as f32) as i32;
    let pattern_length = dash_length + gap_length;

    for i in 0..circumference {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / circumference as f32;
        let x = center.0 + (radius as f32 * angle.cos()) as i32;
        let y = center.1 + (radius as f32 * angle.sin()) as i32;

        // Check if we're in a dash or gap
        let position_in_pattern = i % pattern_length;
        if position_in_pattern < dash_length {
            if x >= 0 && x < image.width() as i32 && y >= 0 && y < image.height() as i32 {
                let pixel = image.get_pixel_mut(x as u32, y as u32);
                // Apply 50% transparency blend
                pixel[0] = ((pixel[0] as u16 + white[0] as u16) / 2) as u8;
                pixel[1] = ((pixel[1] as u16 + white[1] as u16) / 2) as u8;
                pixel[2] = ((pixel[2] as u16 + white[2] as u16) / 2) as u8;
            }
        }
    }
}

/// Draw a dotted circle on an RGB image with 50% transparency (for GMT segment diffraction limit)
fn draw_dotted_segment_circle(image: &mut RgbImage, center: (i32, i32), radius: i32) {
    let white = Rgb([255u8, 255u8, 255u8]);
    let dot_size = 2; // 2 pixels per dot
    let gap_length = 4; // 4 pixels gap between dots

    // Calculate circle circumference and number of dots
    let circumference = (2.0 * std::f32::consts::PI * radius as f32) as i32;
    let pattern_length = dot_size + gap_length;

    for i in 0..circumference {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / circumference as f32;
        let x = center.0 + (radius as f32 * angle.cos()) as i32;
        let y = center.1 + (radius as f32 * angle.sin()) as i32;

        // Check if we're in a dot or gap
        let position_in_pattern = i % pattern_length;
        if position_in_pattern < dot_size {
            if x >= 0 && x < image.width() as i32 && y >= 0 && y < image.height() as i32 {
                let pixel = image.get_pixel_mut(x as u32, y as u32);
                // Apply 50% transparency blend
                pixel[0] = ((pixel[0] as u16 + white[0] as u16) / 2) as u8;
                pixel[1] = ((pixel[1] as u16 + white[1] as u16) / 2) as u8;
                pixel[2] = ((pixel[2] as u16 + white[2] as u16) / 2) as u8;
            }
        }
    }
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

/// Save a single frame as a PNG image with CUBEHELIX colormap and circle overlays
fn save_frame_as_png(
    frame: &[f32],
    filename: &str,
    min_val: f32,
    max_val: f32,
    seeing_radius_pixels: Option<f32>,
    segment_diff_lim_radius_pixels: Option<f32>,
) -> anyhow::Result<()> {
    let rgb_data = frame_to_rgb(frame, min_val, max_val);
    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        DETECTOR_SIZE as u32,
        DETECTOR_SIZE as u32,
        rgb_data,
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

    let center = (DETECTOR_SIZE as i32 / 2, DETECTOR_SIZE as i32 / 2);

    // Draw seeing circle (dashed) if radius is provided
    if let Some(radius) = seeing_radius_pixels {
        draw_dashed_seeing_circle(&mut image, center, radius as i32);
    }

    // Draw GMT segment diffraction limit circle (dotted) if radius is provided
    if let Some(radius) = segment_diff_lim_radius_pixels {
        draw_dotted_segment_circle(&mut image, center, radius as i32);
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
        save_frame_as_png(
            frame,
            filename.to_str().unwrap(),
            global_min,
            global_max,
            seeing_radius_pixels,
            segment_diff_lim_radius_pixels,
        )?;
        save_pb.inc(1);
    }

    save_pb.finish_with_message("All frames saved");
    Ok(())
}

// ray tracing through the GMT
fn ray_trace(v_src: &mut Source, gmt: &mut Gmt, (opd, rbms): (Option<Vec<f64>>, Option<Vec<f64>>)) {
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
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    println!(
        "🎯 Using {} exposure ({} frames)",
        match args.exposure {
            Exposure::Short => "short",
            Exposure::Long => "long",
        },
        N_SAMPLE
    );

    // Setup GMT optics and imaging
    let mut gmt = Gmt::builder().build()?;
    let mut v_src = Source::builder().band("Vs").build()?;
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
    )?;
    println!("Saved frame0 as psf.png");

    // CFD case
    let cfd_case = CfdCase::<CFD_YEAR>::colloquial(30, 0, "os", 7)?;
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

    // Create progress bar for frame processing
    let process_pb = ProgressBar::new(N_SAMPLE as u64);
    process_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    process_pb.set_message("Processing PSF frames");

    match args.exposure {
        Exposure::Short => {
            for data in ds_iter.zip(m12_rbms_iter) {
                imgr.reset();
                ray_trace(&mut v_src, &mut gmt, data);
                v_src.through(&mut imgr);
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
            )?;

            println!();
            println!(
                "✅ Processing completed in {:.2}s",
                now.elapsed().as_secs_f64()
            );
            println!("📁 Saved {} frames to ./frames/ directory", frame_count);
            println!("🖼️  Reference PSF saved as psf.png");
            println!();
            println!("🎬 To create an animated GIF at 5Hz, run:");
            println!("   convert -delay 20 -loop 0 frames/frame_*.png psf_animation.gif");
        }
        Exposure::Long => {
            imgr.reset();
            for data in ds_iter.zip(m12_rbms_iter) {
                ray_trace(&mut v_src, &mut gmt, data);
                v_src.through(&mut imgr);
                process_pb.inc(1);
            }

            process_pb.finish_with_message("PSF processing complete");
            let frame: Vec<f32> = imgr.frame().into();
            let (frame_min, frame_max) = find_global_extrema(&[frame.clone()]);
            save_frame_as_png(
                &frame,
                "long_exposure_psf.png",
                frame_min,
                frame_max,
                Some(seeing_radius_pixels as f32),
                Some(segment_diff_lim_radius_pixels as f32),
            )?;
            println!("Saved frame as psf.png");

            println!();
            println!(
                "✅ Processing completed in {:.2}s",
                now.elapsed().as_secs_f64()
            );
            println!("🖼️  Reference PSF saved as psf.png");
            println!("🖼️  Long exposure PSF saved as long_exposure_psf.png");
        }
    }
    Ok(())
}
