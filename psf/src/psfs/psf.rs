use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use image::{ImageBuffer, ImageError, Rgb};
use imageproc::drawing::draw_hollow_circle_mut;

use super::find_global_extrema;
use crate::{Config, DETECTOR_SIZE, config::ConfigError};

#[derive(Debug, thiserror::Error)]
pub enum PSFError {
    #[error("Failed to create image buffer")]
    Image,
    #[error("Failed to save PSD to png file {1:?}")]
    Save(#[source] ImageError, PathBuf),
    #[error("Failed to invoke config")]
    Config(#[from] ConfigError),
}
type Result<T> = std::result::Result<T, PSFError>;

#[derive(Debug, Default)]
pub struct PSF {
    pub(crate) frame: Vec<f32>,
    pub(crate) pssn_value: Option<f64>,
    pub(crate) frame_number: Option<usize>,
    pub(crate) config: Rc<Config>,
}
impl PSF {
    pub fn new(config: &Rc<Config>, frame: Vec<f32>) -> Self {
        Self {
            frame,
            config: config.clone(),
            ..Default::default()
        }
    }
    pub fn pssn_value(mut self, value: f64) -> Self {
        self.pssn_value = Some(value);
        self
    }
    pub fn frame_number(mut self, value: usize) -> Self {
        self.frame_number = Some(value);
        self
    }
    /// Normalize frame data to 0.0-1.0 range and apply CUBEHELIX colormap
    fn frame_to_rgb(&self, min_val: f32, max_val: f32) -> Vec<u8> {
        let range = max_val - min_val;
        let normalized: Vec<f64> = if range > 0.0 {
            self.frame
                .iter()
                .map(|&x| ((x - min_val) / range) as f64)
                .collect()
        } else {
            vec![0.5f64; self.frame.len()]
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
    pub fn save(&self, filename: impl AsRef<Path>) -> Result<()> {
        self.save_frame_as_png(filename, None)
    }
    /// Save a single frame as a PNG image with CUBEHELIX colormap, circle overlays, and PSSN text
    pub fn save_frame_as_png(
        &self,
        filename: impl AsRef<Path>,
        minmax: Option<(f32, f32)>,
    ) -> Result<()> {
        // let Self {
        //     frame,
        //     pssn_value,
        //     frame_number,
        //     config,
        // } = self;
        let &Config {
            seeing_radius_pixels,
            segment_diff_lim_radius_pixels,
            ..
        } = &*self.config;

        let (min_val, max_val) =
            minmax.unwrap_or_else(|| find_global_extrema(&[self.frame.as_slice()]));

        let rgb_data = self.frame_to_rgb(min_val, max_val);
        let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
            DETECTOR_SIZE as u32,
            DETECTOR_SIZE as u32,
            rgb_data,
        )
        .ok_or_else(|| PSFError::Image)?;

        let center = (DETECTOR_SIZE as i32 / 2, DETECTOR_SIZE as i32 / 2);

        // Draw seeing circle (hollow) if radius is provided
        let white = Rgb([255u8, 255u8, 255u8]);
        draw_hollow_circle_mut(&mut image, center, seeing_radius_pixels as i32, white);

        // Draw GMT segment diffraction limit circle (hollow) if radius is provided
        draw_hollow_circle_mut(
            &mut image,
            center,
            segment_diff_lim_radius_pixels as i32,
            white,
        );

        // Draw PSSN text if values are provided
        if let Some(pssn) = self.pssn_value {
            self.config
                .draw_pssn_text(&mut image, pssn, self.frame_number)?;
        }

        image
            .save(&filename)
            .map_err(|e| PSFError::Save(e, filename.as_ref().to_path_buf()))?;
        Ok(())
    }
}
