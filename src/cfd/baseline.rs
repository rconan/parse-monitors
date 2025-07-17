use std::{
    env::{self, VarError},
    fs, io,
    path::{Path, PathBuf},
};

use strum::IntoEnumIterator;

use crate::cfd::CfdCaseError;

use super::{Azimuth, CfdCase, Enclosure, WindSpeed, ZenithAngle};

#[derive(Debug, thiserror::Error)]
pub enum BaselineError {
    #[error(r#""CFD_CASE" env var is not set"#)]
    Env(#[from] VarError),
    #[error("{0}")]
    ReadFile(#[source] io::Error, String),
    #[error("CFD case error")]
    CfdCase(#[from] CfdCaseError),
}
type Result<T> = std::result::Result<T, BaselineError>;

/// The whole CFD baseline  for a given year: 2020, 2021 or 2025
#[derive(Debug)]
pub struct Baseline<const YEAR: u32>(Vec<CfdCase<YEAR>>);
impl<const YEAR: u32> Baseline<YEAR> {
    /// Read a list of cases from a file which path is given by the env variable `CFD_CASES`
    ///
    /// The file must have one case per line
    pub fn from_env() -> Result<Self> {
        let filename = env::var("CFD_CASES")?;
        let contents =
            fs::read_to_string(&filename).map_err(|e| BaselineError::ReadFile(e, filename))?;
        let items = contents
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .map(|case| CfdCase::try_from(case.as_str()))
            .collect::<std::result::Result<Vec<CfdCase<YEAR>>, CfdCaseError>>()?;
        Ok(Self(items))
    }
}
impl<const YEAR: u32> From<Vec<CfdCase<YEAR>>> for Baseline<YEAR> {
    fn from(cfd_cases: Vec<CfdCase<YEAR>>) -> Self {
        Baseline::<YEAR>(cfd_cases)
    }
}
impl<const YEAR: u32> FromIterator<CfdCase<YEAR>> for Baseline<YEAR> {
    fn from_iter<T: IntoIterator<Item = CfdCase<YEAR>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const YEAR: u32> Default for Baseline<YEAR> {
    fn default() -> Self {
        Self(
            ZenithAngle::iter()
                .flat_map(|zenith_angle| <Self as BaselineTrait<YEAR>>::at_zenith(zenith_angle).0)
                .collect(),
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
pub trait BaselineTrait<const YEAR: u32>:
    Default + From<Vec<CfdCase<YEAR>>> + IntoIterator<Item = CfdCase<YEAR>>
{
    /// Returns the default path to the CFD cases repository
    // fn path() -> PathBuf;
    /// Return the path from the "CFD_REPO" environment variable if it is set,
    /// otherwise returns the default path
    fn path() -> PathBuf {
        env::var("CFD_REPO")
            .map(|p| Path::new(&p).to_path_buf())
            .expect(r#""CFD_REPO" is not set"#)
    }
    /// Returns pairs of [WindSpeed] and [Enclosure] configuration for the given [ZenithAngle]
    fn configuration(zenith_angle: ZenithAngle) -> Vec<(WindSpeed, Enclosure)> {
        match YEAR {
            2020 => vec![
                (WindSpeed::Two, Enclosure::OpenStowed),
                (WindSpeed::Seven, Enclosure::OpenStowed),
                (WindSpeed::Twelve, Enclosure::ClosedDeployed),
                (WindSpeed::Seventeen, Enclosure::ClosedDeployed),
            ],
            2021 | 2025 => match zenith_angle {
                ZenithAngle::Sixty => vec![
                    (WindSpeed::Two, Enclosure::OpenStowed),
                    (WindSpeed::Seven, Enclosure::OpenStowed),
                    //(WindSpeed::Seven, Enclosure::ClosedStowed),
                    (WindSpeed::Twelve, Enclosure::ClosedStowed),
                    (WindSpeed::Seventeen, Enclosure::ClosedStowed),
                ],
                _ => vec![
                    (WindSpeed::Two, Enclosure::OpenStowed),
                    (WindSpeed::Seven, Enclosure::OpenStowed),
                    //(WindSpeed::Seven, Enclosure::ClosedDeployed),
                    (WindSpeed::Twelve, Enclosure::ClosedDeployed),
                    (WindSpeed::Seventeen, Enclosure::ClosedDeployed),
                ],
            },
            _ => vec![],
        }
    }
    /// Returns a CFD baseline reduced to the given [ZenithAngle]
    fn at_zenith(zenith_angle: ZenithAngle) -> Self {
        let mut cfd_cases = vec![];
        for (wind_speed, enclosure) in Self::configuration(zenith_angle.clone()) {
            for azimuth in Azimuth::iter() {
                cfd_cases.push(CfdCase::<YEAR>::new(
                    zenith_angle.clone(),
                    azimuth,
                    enclosure.clone(),
                    wind_speed.clone(),
                ));
            }
        }
        cfd_cases.into()
    }
    /// Finds the CFD case from `OTHER_YEAR` that matches a CFD baseline case in `YEAR`
    fn find<const OTHER_YEAR: u32>(cfd_case_21: CfdCase<OTHER_YEAR>) -> Option<CfdCase<YEAR>> {
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
impl<const YEAR: u32> BaselineTrait<YEAR> for Baseline<YEAR> {}
// impl BaselineTrait<2020> for Baseline<2020> {
//     fn path() -> PathBuf {
//         // Path::new("/fsx/Baseline2020").to_path_buf()
//         Path::new(&env::var("CFD_REPO_2020").expect("CFD_REPO_2020 is not set")).to_path_buf()
//     }

//     // fn configuration(_: ZenithAngle) -> Vec<(WindSpeed, Enclosure)> {
//     //     vec![
//     //         (WindSpeed::Two, Enclosure::OpenStowed),
//     //         (WindSpeed::Seven, Enclosure::OpenStowed),
//     //         (WindSpeed::Twelve, Enclosure::ClosedDeployed),
//     //         (WindSpeed::Seventeen, Enclosure::ClosedDeployed),
//     //     ]
//     // }
// }
// impl BaselineTrait<2021> for Baseline<2021> {
//     fn path() -> PathBuf {
//         env::var("CFD_REPO")
//             .map(|p| Path::new(&p).to_path_buf())
//             .expect(r#""CFD_REPO is not set""#)
//     }

//     // fn configuration(zenith_angle: ZenithAngle) -> Vec<(WindSpeed, Enclosure)> {
//     //     match zenith_angle {
//     //         ZenithAngle::Sixty => vec![
//     //             (WindSpeed::Two, Enclosure::OpenStowed),
//     //             (WindSpeed::Seven, Enclosure::OpenStowed),
//     //             //(WindSpeed::Seven, Enclosure::ClosedStowed),
//     //             (WindSpeed::Twelve, Enclosure::ClosedStowed),
//     //             (WindSpeed::Seventeen, Enclosure::ClosedStowed),
//     //         ],
//     //         _ => vec![
//     //             (WindSpeed::Two, Enclosure::OpenStowed),
//     //             (WindSpeed::Seven, Enclosure::OpenStowed),
//     //             //(WindSpeed::Seven, Enclosure::ClosedDeployed),
//     //             (WindSpeed::Twelve, Enclosure::ClosedDeployed),
//     //             (WindSpeed::Seventeen, Enclosure::ClosedDeployed),
//     //         ],
//     //     }
//     // }
// }
impl<const CFD_YEAR: u32> Baseline<CFD_YEAR> {
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
                            .collect::<Vec<CfdCase<CFD_YEAR>>>(),
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
                            .collect::<Vec<CfdCase<CFD_YEAR>>>(),
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
                            .collect::<Vec<CfdCase<CFD_YEAR>>>(),
                    ),
                    _ => None,
                })
                .flatten()
                .collect::<Vec<CfdCase<CFD_YEAR>>>(),
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn baseline() -> std::result::Result<(), Box<dyn Error>> {
        let cases: Vec<_> = Baseline::<2025>::from_env()?
            // .unwrap_or_default()
            .into_iter()
            .collect();
        println!("{cases:?}");
        Ok(())
    }
}
