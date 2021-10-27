use std::{error::Error, fs::File, ops::Deref, path::Path};

use serde::Deserialize;
use serde_pickle as pickle;

/// Photometric band
pub enum Band {
    V,
    H,
}
/// Dome seeing data
#[derive(Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "Time")]
    pub time: f64,
    #[serde(rename = "V SE PSSn")]
    v_se_pssn: f64,
    #[serde(rename = "H SE PSSn")]
    pub h_se_pssn: f64,
    #[serde(rename = "WFE RMS")]
    pub wfe_rms: Vec<f64>,
    #[serde(rename = "tip-tilt")]
    pub tip_tilt: Vec<f64>,
    #[serde(rename = "segment tip-tilt")]
    pub segment_tip_tilt: Vec<f64>,
    #[serde(rename = "segment piston")]
    pub segment_piston: Vec<f64>,
    #[serde(rename = "V LE PSSn")]
    pub v_le_pssn: Option<f64>,
    #[serde(rename = "H LE PSSn")]
    pub h_le_pssn: Option<f64>,
    #[serde(rename = "V FRAME")]
    pub v_frame: Option<Vec<f64>>,
    #[serde(rename = "H FRAME")]
    pub h_frame: Option<Vec<f64>>,
}
/// Time series of dome seeing data
#[derive(Deserialize)]
pub struct DomeSeeing(Vec<Data>);
impl Deref for DomeSeeing {
    type Target = Vec<Data>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DomeSeeing {
    /// Load the dome seeing time series from a "domeseeing_PSSN.rs.pkl" file
    pub fn load<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>,
    {
        let mut file = File::open(Path::new(&path).join("domeseeing_PSSN.rs.pkl"))?;
        Ok(Self(pickle::from_reader(&mut file, Default::default())?))
    }
    /// Returns the time vector and the wavefront error RMS [m]
    pub fn wfe_rms(&self) -> (Vec<f64>, Vec<f64>) {
        self.iter().map(|ds| (ds.time, ds.wfe_rms[0])).unzip()
    }
    /// Returns the time vector and the instantenous PSSn vector
    pub fn se_pssn(&self, band: Band) -> (Vec<f64>, Vec<f64>) {
        match band {
            Band::V => self.iter().map(|ds| (ds.time, ds.v_se_pssn)).unzip(),
            Band::H => self.iter().map(|ds| (ds.time, ds.h_se_pssn)).unzip(),
        }
    }
    /// Returns the time vector and the long cumulative exposure PSSn vector
    pub fn le_pssn(&self, band: Band) -> (Vec<f64>, Vec<f64>) {
        match band {
            Band::V => self
                .iter()
                .filter_map(|ds| ds.v_le_pssn.map(|x| (ds.time, x)))
                .unzip(),
            Band::H => self
                .iter()
                .filter_map(|ds| ds.h_le_pssn.map(|x| (ds.time, x)))
                .unzip(),
        }
    }
    /// Returns the PSSn
    pub fn pssn(&self, band: Band) -> Option<f64> {
        match band {
            Band::V => self.iter().filter_map(|ds| ds.v_le_pssn).last(),
            Band::H => self.iter().filter_map(|ds| ds.h_le_pssn).last(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn load_dome_seeing() {
        let ds = DomeSeeing::load("data").unwrap();
        println!("Dome Seeing entry #1: {:?}", ds[0]);
    }
    #[test]
    fn dome_seeing_pssn() {
        let ds = DomeSeeing::load("data").unwrap();
        println!(
            "Dome Seeing PSSn V:{:?}, H:{:?}",
            ds.pssn(Band::V),
            ds.pssn(Band::H)
        );
    }
}
