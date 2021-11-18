//! Collection of routines to build the CFD report

use crate::cfd;
use rayon::prelude::*;
use std::error::Error;
use std::marker::{Send, Sync};
use strum::IntoEnumIterator;

pub mod domeseeing;
pub use domeseeing::DomeSeeingPart;
pub mod htc;
pub use htc::HTC;
pub mod windloads;
pub use windloads::WindLoads;

pub trait Report<const CFD_YEAR: u32>: Send + Sync {
    fn part_name(&self) -> String;
    fn chapter_section(&self, cfd_case: cfd::CfdCase<CFD_YEAR>) -> Result<String, Box<dyn Error>>;
    fn chapter(&self, zenith_angle: cfd::ZenithAngle) -> Result<(), Box<dyn Error>>;
    fn part(&self) -> Result<(), Box<dyn Error>> {
        cfd::ZenithAngle::iter()
            .collect::<Vec<cfd::ZenithAngle>>()
            .into_par_iter()
            .for_each(|zenith_angle| {
                println!(" - {} @ {:?}", self.part_name(), zenith_angle);
                self.chapter(zenith_angle).unwrap();
            });
        Ok(())
    }
}
