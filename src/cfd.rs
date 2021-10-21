use std::{
    fmt,
    path::{Path, PathBuf},
};
use strum_macros::EnumIter;

/// CFD Telescope zenith pointing angle
#[derive(EnumIter, Clone)]
pub enum ZenithAngle {
    Zero,
    Thirty,
    Sixty,
}
impl ZenithAngle {
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
#[derive(EnumIter, Clone)]
pub enum Azimuth {
    Zero,
    FortyFive,
    Ninety,
    OneThirtyFive,
    OneEighty,
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
#[derive(Clone)]
pub enum Enclosure {
    OpenStowed,
    ClosedDeployed,
    ClosedStowed,
}
impl Enclosure {
    pub fn to_pretty_string(&self) -> String {
        match self {
            Enclosure::OpenStowed => "Open vents/Stowed wind screen".to_string(),
            Enclosure::ClosedDeployed => "Closed vents/Deployed wind screen".to_string(),
            Enclosure::ClosedStowed => "Closed vents/Stowed wind screen".to_string(),
        }
    }
}
impl fmt::Display for Enclosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Enclosure::OpenStowed => write!(f, "OS"),
            Enclosure::ClosedDeployed => write!(f, "CD"),
            Enclosure::ClosedStowed => write!(f, "CS"),
        }
    }
}
/// CFD wind speed
#[derive(Clone)]
pub enum WindSpeed {
    Two,
    Seven,
    Twelve,
    Seventeen,
    TwentyTwo,
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
/// CFD case for a given year: 2020 or 2021
pub struct CfdCase<const YEAR: u32> {
    pub zenith: ZenithAngle,
    pub azimuth: Azimuth,
    pub enclosure: Enclosure,
    pub wind_speed: WindSpeed,
}
impl<const YEAR: u32> CfdCase<YEAR> {
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
        let mut cfd_cases = vec![];
        for zenith_angle in ZenithAngle::iter() {
            let configs = vec![
                (WindSpeed::Two, Enclosure::OpenStowed),
                (WindSpeed::Seven, Enclosure::OpenStowed),
                (WindSpeed::Twelve, Enclosure::ClosedDeployed),
                (WindSpeed::Seventeen, Enclosure::ClosedDeployed),
            ];
            for (wind_speed, enclosure) in configs {
                for azimuth in Azimuth::iter() {
                    cfd_cases.push(CfdCase::<2020>::new(
                        zenith_angle.clone(),
                        azimuth,
                        enclosure.clone(),
                        wind_speed.clone(),
                    ));
                }
            }
        }
        Self(cfd_cases)
    }
}
impl Default for Baseline<2021> {
    fn default() -> Self {
        let mut cfd_cases = vec![];
        for zenith_angle in ZenithAngle::iter() {
            let configs = match zenith_angle {
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
            };
            for (wind_speed, enclosure) in configs {
                for azimuth in Azimuth::iter() {
                    cfd_cases.push(CfdCase::<2021>::new(
                        zenith_angle.clone(),
                        azimuth,
                        enclosure.clone(),
                        wind_speed.clone(),
                    ));
                }
            }
        }
        Self(cfd_cases)
    }
}
impl<const YEAR: u32> IntoIterator for Baseline<YEAR> {
    type Item = CfdCase<YEAR>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl Baseline<2020> {
    pub fn path() -> PathBuf {
        Path::new("/fsx/Baseline2020").to_path_buf()
    }
    pub fn at_zenith(zenith_angle: ZenithAngle) -> Self {
        let mut cfd_cases = vec![];
        let configs = vec![
            (WindSpeed::Two, Enclosure::OpenStowed),
            (WindSpeed::Seven, Enclosure::OpenStowed),
            (WindSpeed::Twelve, Enclosure::ClosedDeployed),
            (WindSpeed::Seventeen, Enclosure::ClosedDeployed),
        ];
        for (wind_speed, enclosure) in configs {
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
}
impl Baseline<2021> {
    pub fn path() -> PathBuf {
        Path::new("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES").to_path_buf()
    }
    pub fn at_zenith(zenith_angle: ZenithAngle) -> Self {
        let mut cfd_cases = vec![];
        let configs = match zenith_angle {
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
        };
        for (wind_speed, enclosure) in configs {
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
}
