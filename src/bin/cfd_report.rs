use chrono::Local;
use std::{error::Error, fmt, fs::File, io::Write, path::Path};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tectonic;

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
pub struct CfdCase {
    pub zenith: ZenithAngle,
    pub azimuth: Azimuth,
    pub enclosure: Enclosure,
    pub wind_speed: WindSpeed,
}
impl CfdCase {
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
impl fmt::Display for CfdCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}_{}{}",
            self.zenith, self.azimuth, self.enclosure, self.wind_speed
        )
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut zenith_chapters = vec![];
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

        zenith_chapters.push(format!(
            r#"
\chapter{{{}}}
"#,
            zenith_angle.chapter_title()
        ));

        for (wind_speed, enclosure) in configs {
            for azimuth in Azimuth::iter() {
                let cfd_case = CfdCase::new(
                    zenith_angle.clone(),
                    azimuth,
                    enclosure.clone(),
                    wind_speed.clone(),
                );

                zenith_chapters.push(format!(
                    r#"
\clearpage
\section{{{}}}
"#,
                    cfd_case.to_pretty_string()
                ));

                let data_path = Path::new("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/")
                    .join(&cfd_case.to_string())
                    .join("TOTAL_FORCES.png");
                zenith_chapters.push(format!(
                    r#"
\includegraphics[width=\textwidth]{{{:?}}}
"#,
                    data_path
                ));
            }
        }
    }

    let latex = format!(
        r#"
\documentclass{{report}}
\usepackage{{graphicx}}
\addtolength{{\textwidth}}{{3cm}}
\addtolength{{\headheight}}{{5mm}}
\addtolength{{\evensidemargin}}{{-2cm}}
\addtolength{{\oddsidemargin}}{{-1cm}}
\title{{GMT CFD Baseline 2021}}
\author{{R. Conan, K. Vogiatzis, H. Fitzpatrick}}
\date{{{:?}}}
\begin{{document}}
\maketitle
\tableofcontents
\listoffigures
\listoftables
{}
\end{{document}}
"#,
        &Local::now().to_rfc2822(),
        zenith_chapters.join("\n"),
    );

    let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
    let mut doc = File::create("report/gmto.cfd2021.pdf")?;
    doc.write_all(&pdf_data)?;

    Ok(())
}
