//! Collection of routines to build the CFD report

use crate::cfd;
use std::error::Error;
use strum::IntoEnumIterator;

pub mod domeseeing;
pub use domeseeing::DomeSeeingPart;
pub mod htc;
pub use htc::HTC;
pub mod windloads;
pub use windloads::WindLoads;

pub trait Report<const CFD_YEAR: u32> {
    fn chapter_section(&self, cfd_case: cfd::CfdCase<CFD_YEAR>) -> Result<String, Box<dyn Error>>;
    fn chapter(&self, zenith_angle: cfd::ZenithAngle) -> Result<(), Box<dyn Error>>;
    fn part(&self) -> Result<(), Box<dyn Error>> {
        for zenith_angle in cfd::ZenithAngle::iter() {
            self.chapter(zenith_angle)?;
        }
        Ok(())
    }
}
