use std::{
    fs::create_dir_all,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{Config, DETECTOR_SIZE, psfs::psf::PSFError};

mod psf;
pub use psf::PSF;

#[derive(Debug, thiserror::Error)]
pub enum PSFsError {
    #[error("failed to create frames directory {1:?}")]
    CreateFrameDir(#[source] io::Error, PathBuf),
    #[error("failed to save a frame")]
    PsfError(#[from] PSFError),
}

#[derive(Debug, Default)]
pub struct PSFs {
    psfs: Vec<PSF>,
    config: Rc<Config>,
}
/// Find global min and max values across all frames
pub fn find_global_extrema(frames: &[&[f32]]) -> (f32, f32) {
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
impl PSFs {
    pub fn new(config: &Rc<Config>) -> Self {
        Self {
            config: config.clone(),
            ..Default::default()
        }
    }
    pub fn push(&mut self, frame: Vec<f32>, pssn: f64) {
        let i = self.psfs.len();
        self.psfs.push(
            PSF::new(&self.config, frame)
                .pssn_value(pssn)
                .frame_number(i),
        );
    }
    pub fn len(&self) -> usize {
        self.psfs.len()
    }
    pub fn sum(&self) -> PSF {
        let summed_frame = self.psfs.iter().map(|psf| &psf.frame).fold(
            vec![0f32; DETECTOR_SIZE.pow(2)],
            |mut s, f| {
                s.iter_mut().zip(f.into_iter()).for_each(|(s, f)| {
                    *s += f;
                });
                s
            },
        );
        PSF::new(&self.config, summed_frame)
            .pssn_value(self.psfs.last().and_then(|psf| psf.pssn_value).unwrap())
    }
    /// Process all frames and save them as PNG images
    pub fn save_all_frames(&self) -> Result<(), PSFsError> {
        let frames: Vec<_> = self.psfs.iter().map(|psf| psf.frame.as_slice()).collect();
        let global_minmax = find_global_extrema(&frames);
        let n_frame = self.psfs.len();

        // Create progress bar for saving frames
        let save_pb = ProgressBar::new(n_frame as u64);
        save_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
        save_pb.set_message("Saving frames");

        // Setup output directory
        let frames_dir = Path::new("frames");
        create_dir_all(frames_dir)
            .map_err(|e| PSFsError::CreateFrameDir(e, frames_dir.to_path_buf()))?;

        for (i, psf) in self.psfs.iter().enumerate() {
            let filename = frames_dir.join(format!("frame_{:06}.png", i));
            psf.save_frame_as_png(filename, Some(global_minmax))?;
            save_pb.inc(1);
        }

        save_pb.finish_with_message("All frames saved");
        Ok(())
    }
}
