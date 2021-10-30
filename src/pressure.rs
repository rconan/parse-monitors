use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    time::Instant,
};

use bzip2::bufread::BzDecoder;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Record {
    #[serde(rename = "Area: Magnitude (m^2)")]
    area: f64,
    #[serde(rename = "Pressure (Pa)")]
    pressure: f64,
    #[serde(rename = "X (m)")]
    x: f64,
    #[serde(rename = "Y (m)")]
    y: f64,
    #[serde(rename = "Z (m)")]
    z: f64,
}

#[derive(Default)]
pub struct Pressure {
    area: Vec<f64>,
    pressure: Vec<f64>,
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
}

impl Pressure {
    pub fn load<S: Into<String>>(path: S) -> Result<Self, Box<dyn Error>> {
        let csv_file = File::open(Path::new(&path.into()))?;
        log::info!("Loading {:?}...", csv_file);
        let now = Instant::now();
        let buf = BufReader::new(csv_file);
        let mut bz2 = BzDecoder::new(buf);
        let mut contents = String::new();
        bz2.read_to_string(&mut contents)?;
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());
        let mut pressure = Pressure::default();
        for result in rdr.deserialize() {
            let record: Record = result?;
            pressure.area.push(record.area);
            pressure.pressure.push(record.pressure);
            pressure.x.push(record.x);
            pressure.y.push(record.y);
            pressure.z.push(record.z);
        }
        log::info!("... loaded in {:}s", now.elapsed().as_secs());
        Ok(pressure)
    }
    pub fn x_iter(&self) -> impl Iterator<Item = &f64> {
        self.x.iter()
    }
    pub fn y_iter(&self) -> impl Iterator<Item = &f64> {
        self.y.iter()
    }
    pub fn z_iter(&self) -> impl Iterator<Item = &f64> {
        self.z.iter()
    }
    pub fn total_absolute_force(&self) -> f64 {
        self.pressure
            .iter()
            .zip(self.area.iter())
            .map(|(p, a)| p * a)
            .sum()
    }
}
