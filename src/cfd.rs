//! CFD database model based on Rust types

use std::{
    env, fmt,
    path::{Path, PathBuf},
};
use strum_macros::EnumIter;

#[derive(thiserror::Error, Debug)]
pub enum CfdError {
    #[error("zenith angle {0} is not recognized, expected 0, 30 or 60 degree")]
    ZenithAngle(u32),
    #[error("azimuth angle {0} is not recognized, expected 0, 45, 90, 135 or 180 degree")]
    Azimuth(u32),
    #[error(r#"enclosure {0} is not recognized, expected "os", "cd" or "cs""#)]
    Enclosure(String),
    #[error(r#"wind speed {0} is not recognized, expected 2, 7, 12m 17 or 22 m/s"#)]
    WindSpeed(u32),
    #[error("Failed to read CFD data file")]
    ReadDataFile(#[from] glob::GlobError),
    #[error("Data file not recognized")]
    DataFileGlob(#[from] glob::PatternError),
    #[error("{0} data not available")]
    DataFile(String),
}

type Result<T> = std::result::Result<T, CfdError>;

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
            _ => Err(CfdError::ZenithAngle(zenith_angle)),
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
            _ => Err(CfdError::Azimuth(azimuth)),
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
        match enclosure {
            "os" => Ok(OpenStowed),
            "nos" => Ok(NewMeshOpenStowed),
            "cd" => Ok(ClosedDeployed),
            "cs" => Ok(ClosedStowed),
            _ => Err(CfdError::Enclosure(enclosure.into())),
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
            _ => Err(CfdError::WindSpeed(wind_speed)),
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
/// Data file collections available in the CFD database
#[derive(Debug)]
pub enum CfdDataFile<const YEAR: u32> {
    M1Pressure,
    M2Pressure,
    TemperatureField,
    OpticalPathDifference,
    TelescopePressure,
}
impl CfdDataFile<2021> {
    pub fn pattern(self) -> String {
        use CfdDataFile::*;
        String::from(match self {
            M1Pressure => "M1p_M1p_",
            M2Pressure => "M2p_M2p_",
            TemperatureField => "optvol_optvol_",
            OpticalPathDifference => "optvol_optvol_",
            TelescopePressure => "Telescope_p_telescope_",
        })
    }
    pub fn glob(
        self,
        cfd_case: CfdCase<2021>,
    ) -> std::result::Result<impl Iterator<Item = glob::GlobResult>, CfdError> {
        use CfdDataFile::*;
        let cfd_path = Baseline::<2021>::default_path().join(cfd_case.to_string());
        Ok(match self {
            M1Pressure => glob::glob(
                cfd_path
                    .join("pressures")
                    .join("M1p_M1p_*.csv.z")
                    .to_str()
                    .unwrap(),
            )?,
            M2Pressure => glob::glob(
                cfd_path
                    .join("pressures")
                    .join("M2p_M2p_*.csv.z")
                    .to_str()
                    .unwrap(),
            )?,
            TemperatureField => glob::glob(
                cfd_path
                    .join("optvol")
                    .join("optvol_optvol_*.csv.gz")
                    .to_str()
                    .unwrap(),
            )?,
            OpticalPathDifference => glob::glob(
                cfd_path
                    .join("optvol")
                    .join("optvol_optvol_*.npz")
                    .to_str()
                    .unwrap(),
            )?,
            TelescopePressure => glob::glob(
                cfd_path
                    .join("pressures")
                    .join("Telescope_p_telescope_*.csv.z")
                    .to_str()
                    .unwrap(),
            )?,
        })
    }
}
impl CfdDataFile<2020> {
    pub fn glob(
        self,
        cfd_case: CfdCase<2021>,
    ) -> std::result::Result<impl Iterator<Item = glob::GlobResult>, CfdError> {
        use CfdDataFile::*;
        let cfd_path = Baseline::<2021>::default_path().join(cfd_case.to_string());
        match self {
            M1Pressure => Ok(glob::glob(
                cfd_path.join("M1_data_Mod_M1_Data_*.csv").to_str().unwrap(),
            )?),
            M2Pressure => Ok(glob::glob(
                cfd_path.join("M2_data_Mod_M2_Data_*.csv").to_str().unwrap(),
            )?),
            TemperatureField => Ok(glob::glob(
                cfd_path.join("OPDData_OPD_Data_*.csv.gz").to_str().unwrap(),
            )?),
            OpticalPathDifference => Ok(glob::glob(
                cfd_path.join("OPDData_OPD_Data_*.npz").to_str().unwrap(),
            )?),
            _ => Err(CfdError::DataFile(format!("{:?}", self))),
        }
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
    ///
    pub fn to_pretty_string(&self) -> String {
        let z: f64 = self.zenith.clone().into();
        let a: f64 = self.azimuth.clone().into();
        format!(
            "{} zenith - {} azimuth - {} - {}m/s",
            z,
            a,
            self.enclosure.to_pretty_string(),
            self.wind_speed,
        )
    }
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
impl fmt::Display for CfdCase<2021> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}_{}{}",
            self.zenith, self.azimuth, self.enclosure, self.wind_speed
        )
    }
}
impl fmt::Display for CfdCase<2020> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
}
/// The whole CFD baseline  for a given year: 2020 or 2021
pub struct Baseline<const YEAR: u32>(Vec<CfdCase<YEAR>>);
use strum::IntoEnumIterator;
impl Default for Baseline<2020> {
    fn default() -> Self {
        Self(
            ZenithAngle::iter()
                .flat_map(|zenith_angle| Self::at_zenith(zenith_angle).0)
                .collect(),
        )
    }
}
impl Default for Baseline<2021> {
    fn default() -> Self {
        Self(
            ZenithAngle::iter()
                .flat_map(|zenith| {
                    Azimuth::iter()
                        .map(|azimuth| {
                            CfdCase::new(zenith, azimuth, Enclosure::OpenStowed, WindSpeed::Two)
                        })
                        .chain(Azimuth::iter().map(|azimuth| {
                            CfdCase::new(zenith, azimuth, Enclosure::OpenStowed, WindSpeed::Seven)
                        }))
                        .collect::<Vec<_>>()
                })
                .chain(
                    WindSpeed::iter()
                        .skip(2)
                        .take(2)
                        .filter_map(|wind_speed| match wind_speed {
                            WindSpeed::Two => Some(
                                Azimuth::iter()
                                    .take(3)
                                    .map(|azimuth| {
                                        CfdCase::new(
                                            ZenithAngle::Thirty,
                                            azimuth,
                                            Enclosure::OpenStowed,
                                            wind_speed,
                                        )
                                    })
                                    .collect::<Vec<CfdCase<2021>>>(),
                            ),
                            WindSpeed::Seven => Some(
                                Azimuth::iter()
                                    .take(4)
                                    .map(|azimuth| {
                                        CfdCase::new(
                                            ZenithAngle::Thirty,
                                            azimuth,
                                            Enclosure::OpenStowed,
                                            wind_speed,
                                        )
                                    })
                                    .collect::<Vec<CfdCase<2021>>>(),
                            ),
                            WindSpeed::Twelve => Some(
                                Azimuth::iter()
                                    .filter(|azimuth| *azimuth != Azimuth::OneThirtyFive)
                                    .map(|azimuth| {
                                        CfdCase::new(
                                            ZenithAngle::Thirty,
                                            azimuth,
                                            Enclosure::ClosedDeployed,
                                            wind_speed,
                                        )
                                    })
                                    .chain(
                                        Azimuth::iter()
                                            .filter(|azimuth| *azimuth != Azimuth::Zero)
                                            .map(|azimuth| {
                                                CfdCase::new(
                                                    ZenithAngle::Zero,
                                                    azimuth,
                                                    Enclosure::ClosedDeployed,
                                                    wind_speed,
                                                )
                                            }),
                                    )
                                    .collect::<Vec<CfdCase<2021>>>(),
                            ),
                            _ => None,
                        })
                        .flatten(),
                )
                .collect::<Vec<CfdCase<2021>>>(),
        )
    }
}
impl<const YEAR: u32> IntoIterator for Baseline<YEAR> {
    type Item = CfdCase<YEAR>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        if cfg!(feature = "xcase") {
            self.0
                .into_iter()
                .filter(|c| {
                    !(*c == CfdCase::new(
                        ZenithAngle::Zero,
                        Azimuth::Ninety,
                        Enclosure::ClosedDeployed,
                        WindSpeed::Seventeen,
                    ) || *c
                        == CfdCase::new(
                            ZenithAngle::Zero,
                            Azimuth::OneEighty,
                            Enclosure::ClosedDeployed,
                            WindSpeed::Seven,
                        )
                        || *c
                            == CfdCase::new(
                                ZenithAngle::Sixty,
                                Azimuth::Zero,
                                Enclosure::OpenStowed,
                                WindSpeed::Two,
                            ))
                })
                .collect::<Vec<CfdCase<YEAR>>>()
        } else {
            self.0
        }
        .into_iter()
    }
}
impl Baseline<2020> {
    pub fn default_path() -> PathBuf {
        Path::new("/fsx/Baseline2020").to_path_buf()
    }
    pub fn path() -> PathBuf {
        Baseline::<2020>::default_path()
    }
    pub fn at_zenith(zenith_angle: ZenithAngle) -> Self {
        let mut cfd_cases = vec![];
        for (wind_speed, enclosure) in Self::configuration(zenith_angle.clone()) {
            for azimuth in Azimuth::iter() {
                cfd_cases.push(CfdCase::<2020>::new(
                    zenith_angle.clone(),
                    azimuth,
                    enclosure.clone(),
                    wind_speed.clone(),
                ));
            }
        }
        Self(cfd_cases)
    }
    pub fn configuration(_: ZenithAngle) -> Vec<(WindSpeed, Enclosure)> {
        vec![
            (WindSpeed::Two, Enclosure::OpenStowed),
            (WindSpeed::Seven, Enclosure::OpenStowed),
            (WindSpeed::Twelve, Enclosure::ClosedDeployed),
            (WindSpeed::Seventeen, Enclosure::ClosedDeployed),
        ]
    }
    /// Finds the CFD 2020 case that matches a CFD 2021 case
    pub fn find(cfd_case_21: CfdCase<2021>) -> Option<CfdCase<2020>> {
        Self::default().into_iter().find(|cfd_case_20| {
            match (cfd_case_21.zenith.clone(), cfd_case_21.wind_speed.clone()) {
                (ZenithAngle::Sixty, WindSpeed::Twelve | WindSpeed::Seventeen) => {
                    cfd_case_20.zenith == cfd_case_21.zenith
                        && cfd_case_20.azimuth == cfd_case_21.azimuth
                        && cfd_case_20.wind_speed == cfd_case_21.wind_speed
                        && cfd_case_20.enclosure == Enclosure::ClosedDeployed
                }
                _ => {
                    cfd_case_20.zenith == cfd_case_21.zenith
                        && cfd_case_20.azimuth == cfd_case_21.azimuth
                        && cfd_case_20.wind_speed == cfd_case_21.wind_speed
                        && cfd_case_20.enclosure == cfd_case_21.enclosure
                }
            }
        })
    }
}
impl Baseline<2021> {
    pub fn default_path() -> PathBuf {
        Path::new("/fsx/CASES").to_path_buf()
    }
    pub fn path() -> PathBuf {
        env::var("CFD_REPO").map_or_else(
            |_| Baseline::<2021>::default_path(),
            |p| Path::new(&p).to_path_buf(),
        )
    }
    pub fn at_zenith(zenith_angle: ZenithAngle) -> Self {
        let mut cfd_cases = vec![];
        for (wind_speed, enclosure) in Self::configuration(zenith_angle.clone()) {
            for azimuth in Azimuth::iter() {
                cfd_cases.push(CfdCase::<2021>::new(
                    zenith_angle.clone(),
                    azimuth,
                    enclosure.clone(),
                    wind_speed.clone(),
                ));
            }
        }
        Self(cfd_cases)
    }
    pub fn configuration(zenith_angle: ZenithAngle) -> Vec<(WindSpeed, Enclosure)> {
        match zenith_angle {
            ZenithAngle::Sixty => vec![
                (WindSpeed::Two, Enclosure::OpenStowed),
                (WindSpeed::Seven, Enclosure::OpenStowed),
                (WindSpeed::Seven, Enclosure::ClosedStowed),
                (WindSpeed::Twelve, Enclosure::ClosedStowed),
                (WindSpeed::Seventeen, Enclosure::ClosedStowed),
            ],
            _ => vec![
                (WindSpeed::Two, Enclosure::OpenStowed),
                (WindSpeed::Seven, Enclosure::OpenStowed),
                (WindSpeed::Seven, Enclosure::ClosedDeployed),
                (WindSpeed::Twelve, Enclosure::ClosedDeployed),
                (WindSpeed::Seventeen, Enclosure::ClosedDeployed),
            ],
        }
    }
    /// Finds the CFD 2021 case that matches the given CFD case
    pub fn find(cfd_case_21: CfdCase<2021>) -> Option<CfdCase<2021>> {
        Self::default().into_iter().find(|cfd_case_20| {
            match (cfd_case_21.zenith.clone(), cfd_case_21.wind_speed.clone()) {
                (ZenithAngle::Sixty, WindSpeed::Twelve | WindSpeed::Seventeen) => {
                    cfd_case_20.zenith == cfd_case_21.zenith
                        && cfd_case_20.azimuth == cfd_case_21.azimuth
                        && cfd_case_20.wind_speed == cfd_case_21.wind_speed
                        && cfd_case_20.enclosure == Enclosure::ClosedDeployed
                }
                _ => {
                    cfd_case_20.zenith == cfd_case_21.zenith
                        && cfd_case_20.azimuth == cfd_case_21.azimuth
                        && cfd_case_20.wind_speed == cfd_case_21.wind_speed
                        && cfd_case_20.enclosure == cfd_case_21.enclosure
                }
            }
        })
    }
    /// Mount cases
    pub fn mount() -> Self {
        Self(
            WindSpeed::iter()
                .take(3)
                .filter_map(|wind_speed| match wind_speed {
                    WindSpeed::Two => Some(
                        Azimuth::iter()
                            .take(3)
                            .map(|azimuth| {
                                CfdCase::new(
                                    ZenithAngle::Thirty,
                                    azimuth,
                                    Enclosure::OpenStowed,
                                    wind_speed,
                                )
                            })
                            .collect::<Vec<CfdCase<2021>>>(),
                    ),
                    WindSpeed::Seven => Some(
                        Azimuth::iter()
                            .take(4)
                            .map(|azimuth| {
                                CfdCase::new(
                                    ZenithAngle::Thirty,
                                    azimuth,
                                    Enclosure::OpenStowed,
                                    wind_speed,
                                )
                            })
                            .collect::<Vec<CfdCase<2021>>>(),
                    ),
                    WindSpeed::Twelve => Some(
                        Azimuth::iter()
                            .filter(|azimuth| *azimuth != Azimuth::OneThirtyFive)
                            .map(|azimuth| {
                                CfdCase::new(
                                    ZenithAngle::Thirty,
                                    azimuth,
                                    Enclosure::ClosedDeployed,
                                    wind_speed,
                                )
                            })
                            .collect::<Vec<CfdCase<2021>>>(),
                    ),
                    _ => None,
                })
                .flatten()
                .collect::<Vec<CfdCase<2021>>>(),
        )
    }
    /// REDO cases
    pub fn redo() -> Self {
        Self(vec![
            CfdCase::new(
                ZenithAngle::Zero,
                Azimuth::Ninety,
                Enclosure::ClosedDeployed,
                WindSpeed::Seventeen,
            ),
            CfdCase::new(
                ZenithAngle::Zero,
                Azimuth::OneEighty,
                Enclosure::ClosedDeployed,
                WindSpeed::Seven,
            ),
            CfdCase::new(
                ZenithAngle::Sixty,
                Azimuth::Zero,
                Enclosure::OpenStowed,
                WindSpeed::Two,
            ),
        ])
    }
    /// REDO cases
    pub fn thbound2() -> Self {
        Self(vec![
            CfdCase::new(
                ZenithAngle::Thirty,
                Azimuth::FortyFive,
                Enclosure::ClosedDeployed,
                WindSpeed::Seven,
            ),
            CfdCase::new(
                ZenithAngle::Thirty,
                Azimuth::FortyFive,
                Enclosure::OpenStowed,
                WindSpeed::Seven,
            ),
            CfdCase::new(
                ZenithAngle::Thirty,
                Azimuth::OneThirtyFive,
                Enclosure::ClosedDeployed,
                WindSpeed::Seven,
            ),
            CfdCase::new(
                ZenithAngle::Thirty,
                Azimuth::OneThirtyFive,
                Enclosure::OpenStowed,
                WindSpeed::Seven,
            ),
        ])
    }
    /// Extra cases (22m/s)
    pub fn extras(self) -> Self {
        let mut cases = self.0;
        cases.append(&mut vec![
            CfdCase::new(
                ZenithAngle::Thirty,
                Azimuth::Zero,
                Enclosure::ClosedDeployed,
                WindSpeed::TwentyTwo,
            ),
            CfdCase::new(
                ZenithAngle::Thirty,
                Azimuth::FortyFive,
                Enclosure::ClosedDeployed,
                WindSpeed::TwentyTwo,
            ),
        ]);
        Self(cases)
    }
}
