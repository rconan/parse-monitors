/*!
# PSF Configuration

This module provides the [`Config`] type for configuring PSF rendering parameters
and metadata overlays for scientific visualization.

## Features

- Configurable seeing and diffraction limit circle overlays
- Text overlays for CFD case information and turbulence effects
- PSSN value display with wavelength information
- Frame numbering for animated sequences
- Builder pattern for flexible configuration
*/

use std::rc::Rc;

use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};

/// Errors that can occur during PSF configuration operations
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Font loading failed
    #[error("Failed to load font")]
    Font,
}

/// Configuration for PSF rendering including visualization parameters and metadata
/// 
/// This type uses interior mutability through [`Rc`] to enable shared configuration
/// across multiple PSF instances while supporting builder-pattern modifications.
/// 
/// # Example
/// 
/// ```rust,no_run
/// use psf::Config;
/// 
/// let config = Config::new(50.0, 25.0, 500.0)
///     .cfd_case("30deg_0deg_os_7ms")  
///     .turbulence_effects("dome seeing + wind loads");
/// ```
#[derive(Debug, Default)]
pub struct Config {
    pub(crate) seeing_radius_pixels: f32,
    pub(crate) segment_diff_lim_radius_pixels: f32,
    pub(crate) wavelength_nm: f64,
    pub(crate) cfd_case: Option<String>,
    pub(crate) turbulence_effects: Option<String>,
}
impl Config {
    /// Create a new PSF configuration with core visualization parameters
    /// 
    /// # Parameters
    /// 
    /// - `seeing_radius_pixels` - Atmospheric seeing radius in detector pixels for circle overlay
    /// - `segment_diff_lim_radius_pixels` - GMT segment diffraction limit radius in pixels for circle overlay  
    /// - `wavelength_nm` - Observation wavelength in nanometers for PSSN display
    /// 
    /// # Returns
    /// 
    /// Reference-counted configuration instance for shared usage across PSF objects
    pub fn new(
        seeing_radius_pixels: f32,
        segment_diff_lim_radius_pixels: f32,
        wavelength_nm: f64,
    ) -> Rc<Self> {
        Rc::new(Self {
            seeing_radius_pixels,
            segment_diff_lim_radius_pixels,
            wavelength_nm,
            ..Default::default()
        })
    }
    
    /// Add CFD case information to be displayed in text overlay
    /// 
    /// # Parameters
    /// 
    /// - `value` - CFD case description (e.g., "30deg_0deg_os_7ms")
    /// 
    /// # Returns
    /// 
    /// New configuration instance with CFD case metadata
    pub fn cfd_case(self: Rc<Self>, value: impl ToString) -> Rc<Self> {
        let &Self {
            seeing_radius_pixels,
            segment_diff_lim_radius_pixels,
            wavelength_nm,
            ref turbulence_effects,
            ..
        } = &*self;
        Rc::new(Self {
            seeing_radius_pixels,
            segment_diff_lim_radius_pixels,
            wavelength_nm,
            cfd_case: Some(value.to_string()),
            turbulence_effects: turbulence_effects.clone(),
        })
    }
    
    /// Add turbulence effects description to be displayed in text overlay
    /// 
    /// # Parameters
    /// 
    /// - `value` - Description of turbulence effects (e.g., "dome seeing + wind loads")
    /// 
    /// # Returns
    /// 
    /// New configuration instance with turbulence effects metadata
    pub fn turbulence_effects(self: Rc<Self>, value: impl ToString) -> Rc<Self> {
        let &Self {
            seeing_radius_pixels,
            segment_diff_lim_radius_pixels,
            wavelength_nm,
            ref cfd_case,
            ..
        } = &*self;
        Rc::new(Self {
            seeing_radius_pixels,
            segment_diff_lim_radius_pixels,
            wavelength_nm,
            cfd_case: cfd_case.clone(),
            turbulence_effects: Some(value.to_string()),
        })
    }
    /// Draw PSSN and metadata text overlays in the top left corner of the image
    /// 
    /// Renders white text overlays including:
    /// - CFD case information (if configured)
    /// - Turbulence effects description (if configured) 
    /// - PSSN value with wavelength
    /// - Frame number (if provided)
    /// 
    /// # Parameters
    /// 
    /// - `image` - Mutable reference to RGB image to modify
    /// - `pssn_value` - Point Spread Function Strehl Number value to display
    /// - `frame_number` - Optional frame number for animated sequences
    /// 
    /// # Returns
    /// 
    /// Result indicating success or font loading error
    pub fn draw_pssn_text(
        &self,
        image: &mut RgbImage,
        pssn_value: f64,
        frame_number: Option<usize>,
    ) -> Result<(), ConfigError> {
        // Use system default font (typically DejaVu Sans on Linux)
        let font_data: &[u8] = include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        let font = Font::try_from_bytes(font_data).ok_or_else(|| ConfigError::Font)?;

        let scale = Scale::uniform(20.0); // 24 pixel font
        let white = Rgb([255u8, 255u8, 255u8]);

        // Position in top left corner with some padding
        let x = 5i32;
        let mut y = 5i32;

        // Draw CFD case if provided
        if let Some(case) = &self.cfd_case {
            let cfd_text = format!("CFD: {}", case);
            draw_text_mut(image, white, x, y, scale, &font, &cfd_text);
            y += 30;
        }

        // Draw turbulence effects if provided
        if let Some(effects) = &self.turbulence_effects {
            let effects_text = format!("Effects: {}", effects);
            draw_text_mut(image, white, x, y, scale, &font, &effects_text);
            y += 30;
        }

        // Draw PSSN text
        let pssn_text = format!("PSSN@{:.0}nm: {:.5}", self.wavelength_nm, pssn_value);
        draw_text_mut(image, white, x, y, scale, &font, &pssn_text);

        // Draw frame number if provided
        if let Some(frame_num) = frame_number {
            y += 30; // Move down for next line
            let frame_text = format!("frame {:03}", frame_num);
            draw_text_mut(image, white, x, y, scale, &font, &frame_text);
        }

        Ok(())
    }
}
