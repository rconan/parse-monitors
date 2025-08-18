use std::{fs::create_dir_all, path::Path, time::Instant};

use clap::{Parser, ValueEnum};
use colorous;
use crseo::{Builder, FromBuilder, Gmt, Imaging, Source, imaging::Detector};
use gmt_dos_clients_domeseeing::DomeSeeing;
use image::save_buffer;
use indicatif::{ProgressBar, ProgressStyle};
use parse_monitors::{
    CFD_YEAR,
    cfd::{Baseline, BaselineTrait, CfdCase},
};

const N_SAMPLE: usize = 100;
const DETECTOR_SIZE: usize = 200;

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

/// Save a single frame as a PNG image with CUBEHELIX colormap
fn save_frame_as_png(
    frame: &[f32],
    filename: &str,
    min_val: f32,
    max_val: f32,
) -> anyhow::Result<()> {
    let rgb_data = frame_to_rgb(frame, min_val, max_val);
    save_buffer(
        filename,
        &rgb_data,
        DETECTOR_SIZE as u32,
        DETECTOR_SIZE as u32,
        image::ColorType::Rgb8,
    )?;
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
fn save_all_frames(frames: &[Vec<f32>], frames_dir: &Path) -> anyhow::Result<()> {
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
        save_frame_as_png(frame, filename.to_str().unwrap(), global_min, global_max)?;
        save_pb.inc(1);
    }

    save_pb.finish_with_message("All frames saved");
    Ok(())
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

    // Setup CFD case and dome seeing
    let cfd_case = CfdCase::<CFD_YEAR>::colloquial(30, 0, "os", 7)?;
    let cfd_path = Baseline::<CFD_YEAR>::path()?.join(cfd_case.to_string());
    let ds = DomeSeeing::builder(&cfd_path).build()?;

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

    // Generate reference frame (no turbulence)
    v_src.through(&mut gmt).xpupil().through(&mut imgr);
    let frame0: Vec<f32> = imgr.frame().into();

    // Save reference frame with its own normalization
    let (frame0_min, frame0_max) = find_global_extrema(&[frame0.clone()]);
    save_frame_as_png(&frame0, "psf.png", frame0_min, frame0_max)?;
    println!("Saved frame0 as psf.png");

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
            for (frame_count, opd) in ds.enumerate() {
                imgr.reset();
                v_src
                    .through(&mut gmt)
                    .xpupil()
                    .add(opd.as_slice())
                    .through(&mut imgr);
                let frame: Vec<f32> = imgr.frame().into();

                all_frames.push(frame);
                process_pb.inc(1);

                if frame_count + 1 >= N_SAMPLE {
                    break;
                }
            }

            process_pb.finish_with_message("PSF processing complete");
            let frame_count = all_frames.len();

            // Save all turbulence frames with consistent normalization
            save_all_frames(&all_frames, frames_dir)?;

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
            for (frame_count, opd) in ds.enumerate() {
                v_src
                    .through(&mut gmt)
                    .xpupil()
                    .add(opd.as_slice())
                    .through(&mut imgr);

                process_pb.inc(1);

                if frame_count + 1 >= N_SAMPLE {
                    break;
                }
            }

            process_pb.finish_with_message("PSF processing complete");
            let frame: Vec<f32> = imgr.frame().into();
            let (frame_min, frame_max) = find_global_extrema(&[frame.clone()]);
            save_frame_as_png(&frame, "long_exposure_psf.png", frame_min, frame_max)?;
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
