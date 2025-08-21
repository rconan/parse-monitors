/*!
# PSF Collections

This module provides the [`PSFs`] type for managing collections of PSF frames
with shared configuration and batch processing capabilities.

## Features

- Global normalization across all frames for consistent visualization
- Progress bars for batch operations  
- Automatic frame numbering and metadata management
- Efficient storage and processing of large PSF datasets
*/

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

/// Errors that can occur during PSF collection operations
#[derive(Debug, thiserror::Error)]
pub enum PSFsError {
    /// Failed to create output directory for frames
    #[error("failed to create frames directory {1:?}")]
    CreateFrameDir(#[source] io::Error, PathBuf),
    /// Failed to process or save individual PSF frame
    #[error("failed to save a frame")]
    PsfError(#[from] PSFError),
}

/// Collection of PSF frames with shared configuration and batch processing
/// 
/// This type manages multiple [`PSF`] instances with consistent configuration
/// and provides utilities for global normalization and batch export operations.
/// 
/// # Example
/// 
/// ```rust,no_run
/// use psf::{Config, PSFs};
/// 
/// let config = Config::new(50.0, 25.0, 500.0);
/// let mut psfs = PSFs::new(&config);
/// 
/// // Add frames with metadata
/// psfs.push(frame_data1, pssn_value1);
/// psfs.push(frame_data2, pssn_value2);
/// 
/// // Export all frames with global normalization
/// psfs.save_all_frames()?;
/// ```
#[derive(Debug, Default)]
pub struct PSFs {
    psfs: Vec<PSF>,
    config: Rc<Config>,
}

/// Find global minimum and maximum values across all frames for consistent normalization
/// 
/// # Parameters
/// 
/// - `frames` - Slice of frame data references to analyze
/// 
/// # Returns
/// 
/// Tuple of (global_min, global_max) values across all frames
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
    /// Create a new PSF collection with shared configuration
    /// 
    /// # Parameters
    /// 
    /// - `config` - Shared configuration for all PSF frames
    /// 
    /// # Returns
    /// 
    /// Empty PSF collection ready for frame addition
    pub fn new(config: &Rc<Config>) -> Self {
        Self {
            config: config.clone(),
            ..Default::default()
        }
    }
    
    /// Add a new PSF frame to the collection with automatic numbering
    /// 
    /// # Parameters
    /// 
    /// - `frame` - Raw PSF intensity data as flat vector (DETECTOR_SIZE²)
    /// - `pssn` - Normalized point source sensitivity for an integration up to this frame
    pub fn push(&mut self, frame: Vec<f32>, pssn: f64) {
        let i = self.psfs.len();
        self.psfs.push(
            PSF::new(&self.config, frame)
                .pssn_value(pssn)
                .frame_number(i),
        );
    }
    
    /// Get the number of PSF frames in the collection
    pub fn len(&self) -> usize {
        self.psfs.len()
    }
    
    /// Create a summed (long exposure) PSF from all frames in the collection
    /// 
    /// # Returns
    /// 
    /// Single PSF representing the sum of all individual frames, 
    /// using the PSSN value from the last frame
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
    /// Export all PSF frames as PNG images with global normalization and progress tracking
    /// 
    /// Creates a `frames/` directory and saves each PSF as `frame_XXXXXX.png` 
    /// with consistent normalization across all frames for proper visualization
    /// of temporal variations. Shows progress bar during export process.
    /// 
    /// # Returns
    /// 
    /// Result indicating success or failure of the batch export operation
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
