use std::{fmt, num::ParseIntError, path::Path};
use strum_macros::EnumIter;

#[derive(Debug, thiserror::Error)]
pub enum CfdCaseError {
    #[error("zenith angle {0} is not recognized, expected 0, 30 or 60 degree")]
    ZenithAngle(u32),
    #[error("azimuth angle {0} is not recognized, expected 0, 45, 90, 135 or 180 degree")]
    Azimuth(u32),
    #[error(r#"enclosure {0} is not recognized, expected "os", "cd" or "cs""#)]
    Enclosure(String),
    #[error(r#"wind speed {0} is not recognized, expected 2, 7, 12m 17 or 22 m/s"#)]
    WindSpeed(u32),
    #[error("invalid CFD case name regex")]
    Regex(#[from] regex::Error),
    #[error("{0} doesn't match expected pattern")]
    CfdPattern(String),
    #[error("CFD case parsing error")]
    CfdParser(#[from] ParseIntError),
}
type Result<T> = std::result::Result<T, CfdCaseError>;

/// CFD Telescope zenith pointing angle
#[derive(EnumIter, Clone, Copy, PartialEq, Debug)]
pub enum ZenithAngle {
    Zero,
    Thirty,
    Sixty,
}
impl ZenithAngle {
    /// Get a new `ZenithAngle` chosen from 0, 30 or 60 degrees
    pub fn new(zenith_angle: u32) -> Result<Self> {
        use ZenithAngle::*;
        match zenith_angle {
            0 => Ok(Zero),
            30 => Ok(Thirty),
            60 => Ok(Sixty),
            _ => Err(CfdCaseError::ZenithAngle(zenith_angle)),
        }
    }
    pub fn chapter_title(&self) -> String {
        let z: f64 = self.into();
        format!("Zenith angle: {} degree", z)
    }
}
impl From<ZenithAngle> for f64 {
    fn from(zen: ZenithAngle) -> Self {
        match zen {
            ZenithAngle::Zero => 0f64,
            ZenithAngle::Thirty => 30f64,
            ZenithAngle::Sixty => 60f64,
        }
    }
}
impl From<&ZenithAngle> for f64 {
    fn from(zen: &ZenithAngle) -> Self {
        match zen {
            ZenithAngle::Zero => 0f64,
            ZenithAngle::Thirty => 30f64,
            ZenithAngle::Sixty => 60f64,
        }
    }
}
impl fmt::Display for ZenithAngle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZenithAngle::Zero => write!(f, "zen00"),
            ZenithAngle::Thirty => write!(f, "zen30"),
            ZenithAngle::Sixty => write!(f, "zen60"),
        }
    }
}
/// CFD Telescope azimuth angle (wrt. NNE wind)
#[derive(EnumIter, Clone, Copy, PartialEq, Debug)]
pub enum Azimuth {
    Zero,
    FortyFive,
    Ninety,
    OneThirtyFive,
    OneEighty,
}
impl Azimuth {
    /// Get a new `Azimuth` chosen from 0, 45, 90, 135 or 180 degrees
    pub fn new(azimuth: u32) -> Result<Self> {
        use Azimuth::*;
        match azimuth {
            0 => Ok(Zero),
            45 => Ok(FortyFive),
            90 => Ok(Ninety),
            135 => Ok(OneThirtyFive),
            180 => Ok(OneEighty),
            _ => Err(CfdCaseError::Azimuth(azimuth)),
        }
    }
    pub fn sin_cos(&self) -> (f64, f64) {
        let v: f64 = self.into();
        v.to_radians().sin_cos()
    }
}
impl From<Azimuth> for f64 {
    fn from(azi: Azimuth) -> Self {
        use Azimuth::*;
        match azi {
            Zero => 0f64,
            FortyFive => 45f64,
            Ninety => 90f64,
            OneThirtyFive => 135f64,
            OneEighty => 180f64,
        }
    }
}
impl From<&Azimuth> for f64 {
    fn from(azi: &Azimuth) -> Self {
        use Azimuth::*;
        match azi {
            Zero => 0f64,
            FortyFive => 45f64,
            Ninety => 90f64,
            OneThirtyFive => 135f64,
            OneEighty => 180f64,
        }
    }
}
impl fmt::Display for Azimuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Azimuth::*;
        match self {
            Zero => write!(f, "az000"),
            FortyFive => write!(f, "az045"),
            Ninety => write!(f, "az090"),
            OneThirtyFive => write!(f, "az135"),
            OneEighty => write!(f, "az180"),
        }
    }
}
/// Enclosure vents and wind screen configuration combinations
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Enclosure {
    OpenStowed,
    NewMeshOpenStowed,
    ClosedDeployed,
    ClosedStowed,
}
impl Enclosure {
    /// Get a new `Enclosure` chosen from "os", "cd" or "cs"
    pub fn new(enclosure: &str) -> Result<Self> {
        use Enclosure::*;
        match enclosure.to_lowercase().as_str() {
            "os" => Ok(OpenStowed),
            "nos" => Ok(NewMeshOpenStowed),
            "cd" => Ok(ClosedDeployed),
            "cs" => Ok(ClosedStowed),
            _ => Err(CfdCaseError::Enclosure(enclosure.into())),
        }
    }
    pub fn to_pretty_string(&self) -> String {
        match self {
            Enclosure::OpenStowed => "Open vents/Stowed wind screen".to_string(),
            Enclosure::NewMeshOpenStowed => "New mesh/Open vents/Stowed wind screen".to_string(),
            Enclosure::ClosedDeployed => "Closed vents/Deployed wind screen".to_string(),
            Enclosure::ClosedStowed => "Closed vents/Stowed wind screen".to_string(),
        }
    }
}
impl fmt::Display for Enclosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Enclosure::OpenStowed => write!(f, "OS"),
            Enclosure::NewMeshOpenStowed => write!(f, "NOS"),
            Enclosure::ClosedDeployed => write!(f, "CD"),
            Enclosure::ClosedStowed => write!(f, "CS"),
        }
    }
}
/// CFD wind speed
#[derive(EnumIter, Copy, PartialEq, Clone, Debug)]
pub enum WindSpeed {
    Two,
    Seven,
    Twelve,
    Seventeen,
    TwentyTwo,
}
impl WindSpeed {
    /// Get a new `WindSpeed` chosen from 0, 2, 7, 12, 17 or 22m/s
    fn new(wind_speed: u32) -> Result<Self> {
        use WindSpeed::*;
        match wind_speed {
            2 => Ok(Two),
            7 => Ok(Seven),
            12 => Ok(Twelve),
            17 => Ok(Seventeen),
            22 => Ok(TwentyTwo),
            _ => Err(CfdCaseError::WindSpeed(wind_speed)),
        }
    }
}
impl fmt::Display for WindSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WindSpeed::*;
        match self {
            Two => write!(f, "2"),
            Seven => write!(f, "7"),
            Twelve => write!(f, "12"),
            Seventeen => write!(f, "17"),
            TwentyTwo => write!(f, "22"),
        }
    }
}
impl From<WindSpeed> for f64 {
    fn from(wind_speed: WindSpeed) -> Self {
        use WindSpeed::*;
        (match wind_speed {
            Two => 2,
            Seven => 7,
            Twelve => 12,
            Seventeen => 17,
            TwentyTwo => 22,
        } as f64)
    }
}
/// CFD case for a given year: 2020 or 2021
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CfdCase<const YEAR: u32> {
    pub zenith: ZenithAngle,
    pub azimuth: Azimuth,
    pub enclosure: Enclosure,
    pub wind_speed: WindSpeed,
}
impl<const YEAR: u32> CfdCase<YEAR> {
    /// A new CFD case
    pub fn new(
        zenith: ZenithAngle,
        azimuth: Azimuth,
        enclosure: Enclosure,
        wind_speed: WindSpeed,
    ) -> Self {
        Self {
            zenith,
            azimuth,
            enclosure,
            wind_speed,
        }
    }
    /// A new CFD case, it will return an error if the values are not found in the CFD database
    pub fn colloquial(
        zenith_angle: u32,
        azimuth: u32,
        enclosure: &str,
        wind_speed: u32,
    ) -> Result<Self> {
        Ok(CfdCase::<YEAR>::new(
            ZenithAngle::new(zenith_angle)?,
            Azimuth::new(azimuth)?,
            Enclosure::new(enclosure)?,
            WindSpeed::new(wind_speed)?,
        ))
    }
    /// Pretty print the CFD case
    pub fn to_pretty_string(&self) -> String {
        let z: f64 = self.zenith.clone().into();
        let a: f64 = self.azimuth.clone().into();
        format!(
            "{} deg zenith - {} deg azimuth - {} - {}m/s",
            z,
            a,
            self.enclosure.to_pretty_string(),
            self.wind_speed,
        )
    }
    /// Format the CFD case as a Latex tabular row
    pub fn to_latex_string(&self) -> String {
        let z: f64 = self.zenith.clone().into();
        let a: f64 = self.azimuth.clone().into();
        format!(
            "{:3} & {:3} & {} & {:>2}",
            z,
            a,
            self.enclosure.to_string().to_lowercase(),
            self.wind_speed.to_string(),
        )
    }
}
impl<const YEAR: u32> fmt::Display for CfdCase<YEAR> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match YEAR {
            2020 => {
                let z: f64 = self.zenith.clone().into();
                let a: f64 = self.azimuth.clone().into();
                write!(
                    f,
                    "b2019_{}z_{}az_{}_{}ms",
                    z,
                    a,
                    self.enclosure.to_string().to_lowercase(),
                    self.wind_speed
                )
            }
            2021 => {
                write!(
                    f,
                    "{}{}_{}{}",
                    self.zenith, self.azimuth, self.enclosure, self.wind_speed
                )
            }
            2025 => {
                write!(
                    f,
                    "{}{}_{}_{}ms",
                    self.zenith, self.azimuth, self.enclosure, self.wind_speed
                )
            }
            _ => Err(fmt::Error),
        }
    }
}
impl<const YEAR: u32> TryFrom<&str> for CfdCase<YEAR> {
    type Error = CfdCaseError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let re = regex::Regex::new(r"^zen(\d+)az(\d+)_(.+)_(\d+)ms$")?;
        let caps = re
            .captures(value)
            .ok_or(CfdCaseError::CfdPattern(value.to_string()))?;
        let zenith_angle: u32 = caps[1].parse()?;
        let azimuth: u32 = caps[2].parse()?;
        let enclosure = &caps[3];
        let wind_speed: u32 = caps[4].parse()?;
        CfdCase::colloquial(zenith_angle, azimuth, enclosure, wind_speed)
    }
}
