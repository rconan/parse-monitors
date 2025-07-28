//! Collection of routines to build the CFD report

use crate::cfd::{self, BaselineError};
use crate::MonitorsError;
use rayon::prelude::*;
use std::{
    io,
    marker::{Send, Sync},
    path::PathBuf,
};
use strum::IntoEnumIterator;

pub mod domeseeing;
pub use domeseeing::DomeSeeingPart;
pub mod htc;
pub use htc::HTC;
pub mod windloads;
pub use windloads::WindLoads;

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("failed to create wind loads report: {1:?}")]
    Creating(#[source] io::Error, PathBuf),
    #[error("failed to write wind loads report: {1:?}")]
    Writing(#[source] io::Error, PathBuf),
    #[error("wind loads CFD baseline error")]
    Baseline(#[from] BaselineError),
    #[error("wind loads glob error")]
    Glob(#[from] glob::GlobError),
    #[error("wind loads pattern error")]
    Pattern(#[from] glob::PatternError),
    #[error("wind loads monitor error")]
    Monitors(#[from] MonitorsError),
}

pub trait Report<const CFD_YEAR: u32>: Send + Sync {
    type Error: std::fmt::Debug;
    fn part_name(&self) -> String;
    fn chapter_section(
        &self,
        cfd_case: cfd::CfdCase<CFD_YEAR>,
        ri_pic_idx: Option<usize>,
    ) -> Result<String, Self::Error>;
    fn chapter(
        &self,
        zenith_angle: cfd::ZenithAngle,
        cfd_cases_subset: Option<&[cfd::CfdCase<CFD_YEAR>]>,
    ) -> Result<(), Self::Error>;
    fn part(&self) -> Result<(), Self::Error> {
        cfd::ZenithAngle::iter()
            .collect::<Vec<cfd::ZenithAngle>>()
            .into_par_iter()
            .for_each(|zenith_angle| {
                println!(" - {} @ {:?}", self.part_name(), zenith_angle);
                self.chapter(zenith_angle, None).unwrap();
            });
        Ok(())
    }
    fn part_with(
        &self,
        may_be_cfd_cases_subset: Option<&[cfd::CfdCase<CFD_YEAR>]>,
    ) -> Result<(), Self::Error> {
        if let Some(cfd_cases_subset) = may_be_cfd_cases_subset {
            cfd::ZenithAngle::iter()
                .collect::<Vec<cfd::ZenithAngle>>()
                .into_par_iter()
                .for_each(|zenith_angle| {
                    println!(" - {} @ {:?}", self.part_name(), zenith_angle);
                    self.chapter(zenith_angle, Some(cfd_cases_subset)).unwrap();
                });
            Ok(())
        } else {
            self.part()
        }
    }
}
