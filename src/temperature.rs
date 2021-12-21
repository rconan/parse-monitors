use flate2::read::GzDecoder;
use serde::Deserialize;
use std::io::prelude::*;
use std::{fs::File, path::Path};

#[derive(thiserror::Error, Debug)]
pub enum TemperatureError {
    #[error("Failed to open the pressure file")]
    Io(#[from] std::io::Error),
    #[error("Failed to deserialize the CSV file")]
    Csv(#[from] csv::Error),
}
type Result<T> = std::result::Result<T, TemperatureError>;

#[derive(Deserialize, Debug)]
struct Record {
    #[serde(rename = "Temperature (K)")]
    temperature: f64,
    #[serde(rename = "X (m)")]
    x: f64,
    #[serde(rename = "Y (m)")]
    y: f64,
    #[serde(rename = "Z (m)")]
    z: f64,
}

#[derive(Debug)]
pub struct Temperature {
    // the temperature [K]
    temperature: Vec<f64>,
    // the (x,y,z) coordinate where the temperature is monitored
    xyz: Vec<[f64; 3]>,
}
impl Default for Temperature {
    fn default() -> Self {
        Self {
            temperature: Vec::with_capacity(6546592),
            xyz: Vec::with_capacity(6546592),
        }
    }
}
impl Temperature {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let csv_file = File::open(path)?;
        let mut gz = GzDecoder::new(csv_file);
        let mut contents = String::new();
        gz.read_to_string(&mut contents)?;
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());
        let mut this = Self::default();
        for result in rdr.deserialize() {
            let record: Record = result?;
            this.temperature.push(record.temperature);
            this.xyz.push([record.x, record.y, record.z]);
        }
        Ok(this)
    }
    /// Iterator over the x coordinate
    fn xyz_iter(&self, axis: usize) -> impl Iterator<Item = f64> + '_ {
        self.xyz.iter().map(move |v| v[axis])
    }
    pub fn x_iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.xyz_iter(0)
    }
    /// Returns the range of the x cooordinate
    pub fn x_range(&self) -> (f64, f64) {
        (
            self.x_iter().fold(std::f64::INFINITY, f64::min),
            self.x_iter().fold(std::f64::NEG_INFINITY, f64::max),
        )
    }
    /// Iterator over the y coordinate
    pub fn y_iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.xyz_iter(1)
    }
    /// Iterator over the (x,y) coordinates
    pub fn xy_iter(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.x_iter().zip(self.y_iter())
    }
    /// Returns the range of the y cooordinate
    pub fn y_range(&self) -> (f64, f64) {
        (
            self.y_iter().fold(std::f64::INFINITY, f64::min),
            self.y_iter().fold(std::f64::NEG_INFINITY, f64::max),
        )
    }
    /// Iterator over the z coordinate
    pub fn z_iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.xyz_iter(2)
    }
    /// Returns the range of the z cooordinate
    pub fn z_range(&self) -> (f64, f64) {
        (
            self.z_iter().fold(std::f64::INFINITY, f64::min),
            self.z_iter().fold(std::f64::NEG_INFINITY, f64::max),
        )
    }
    /// Iterator over the (x,y) coordinates of a given segment
    pub fn temperature_iter(&self) -> impl Iterator<Item = &f64> + '_ {
        self.temperature.iter()
    }
}
