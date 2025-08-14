use std::{error::Error, fmt::Display, io, path::PathBuf};

use parse_monitors::{
    CFD_YEAR,
    cfd::{BaselineError, CfdCase, CfdError},
    report::{self, domeseeing::DomeSeeingPartError, htc::HTCError, windloads::WindLoadsError},
};

#[derive(Debug)]
pub struct ReportPathError {
    path: PathBuf,
    source: io::Error,
}
impl ReportPathError {
    pub fn new(path: PathBuf, source: io::Error) -> Self {
        Self { path, source }
    }
}
impl Display for ReportPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to create report folder: {:?}", self.path)
    }
}
impl Error for ReportPathError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub struct PressureMapsError<const Y: u32> {
    case: CfdCase<Y>,
    source: CfdError,
}
impl<const Y: u32> PressureMapsError<Y> {
    pub fn new(case: CfdCase<Y>, source: CfdError) -> Self {
        Self { case, source }
    }
}
impl<const Y: u32> Display for PressureMapsError<Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to generated pressure maps for {}", self.case)
    }
}
impl<const Y: u32> Error for PressureMapsError<Y> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub enum ReportError {
    ReportPath(ReportPathError),
    PressureMaps(PressureMapsError<{ CFD_YEAR }>),
    DomeSeeing(DomeSeeingPartError),
    WindLoads(WindLoadsError),
    HTC(HTCError),
    Baseline(BaselineError),
    Report(report::ReportError),
}
impl Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to compile CFD report")
    }
}
impl Error for ReportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReportError::DomeSeeing(e) => Some(e),
            ReportError::WindLoads(e) => Some(e),
            ReportError::HTC(e) => Some(e),
            ReportError::ReportPath(e) => Some(e),
            ReportError::Baseline(e) => Some(e),
            ReportError::Report(e) => Some(e),
            ReportError::PressureMaps(e) => Some(e),
        }
    }
}
impl From<DomeSeeingPartError> for ReportError {
    fn from(value: DomeSeeingPartError) -> Self {
        ReportError::DomeSeeing(value)
    }
}
impl From<WindLoadsError> for ReportError {
    fn from(value: WindLoadsError) -> Self {
        ReportError::WindLoads(value)
    }
}
impl From<HTCError> for ReportError {
    fn from(value: HTCError) -> Self {
        ReportError::HTC(value)
    }
}
impl From<ReportPathError> for ReportError {
    fn from(value: ReportPathError) -> Self {
        ReportError::ReportPath(value)
    }
}
impl From<BaselineError> for ReportError {
    fn from(value: BaselineError) -> Self {
        ReportError::Baseline(value)
    }
}
impl From<report::ReportError> for ReportError {
    fn from(value: report::ReportError) -> Self {
        ReportError::Report(value)
    }
}
impl From<PressureMapsError<{ CFD_YEAR }>> for ReportError {
    fn from(value: PressureMapsError<{ CFD_YEAR }>) -> Self {
        ReportError::PressureMaps(value)
    }
}
