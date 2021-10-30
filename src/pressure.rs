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
struct Record {
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

/// M1 segments surface pressure
#[derive(Default)]
pub struct Pressure {
    /// the segment surface pressure
    pressure: Vec<f64>,
    /// the area the pressure is applied to
    area: Vec<f64>,
    /// the x coordinate where the pressure is applied
    x: Vec<f64>,
    /// the y coordinate where the pressure is applied
    y: Vec<f64>,
    /// the z coordinate where the pressure is applied
    z: Vec<f64>,
}

impl Pressure {
    /// Loads the pressure from a csv bz2-compressed file
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
    /// Iterator over the x coordinate
    pub fn x_iter(&self) -> impl Iterator<Item = &f64> {
        self.x.iter()
    }
    /// Returns the range of the x cooordinate
    pub fn x_range(&self) -> (f64, f64) {
        (
            self.x.iter().cloned().fold(std::f64::INFINITY, f64::min),
            self.x
                .iter()
                .cloned()
                .fold(std::f64::NEG_INFINITY, f64::max),
        )
    }
    /// Iterator over the y coordinate
    pub fn y_iter(&self) -> impl Iterator<Item = &f64> {
        self.y.iter()
    }
    /// Iterator over the (x,y) coordinates
    pub fn xy_iter(&self) -> impl Iterator<Item = (&f64, &f64)> {
        self.x.iter().zip(self.y.iter())
    }
    /// Returns the range of the y cooordinate
    pub fn y_range(&self) -> (f64, f64) {
        (
            self.y.iter().cloned().fold(std::f64::INFINITY, f64::min),
            self.y
                .iter()
                .cloned()
                .fold(std::f64::NEG_INFINITY, f64::max),
        )
    }
    /// Iterator over the z coordinate
    pub fn z_iter(&self) -> impl Iterator<Item = &f64> {
        self.z.iter()
    }
    /// Returns the range of the z cooordinate
    pub fn z_range(&self) -> (f64, f64) {
        (
            self.z.iter().cloned().fold(std::f64::INFINITY, f64::min),
            self.z
                .iter()
                .cloned()
                .fold(std::f64::NEG_INFINITY, f64::max),
        )
    }
    /// Transforms the coordinates into the segment local coordinate system
    pub fn to_local(&mut self, sid: usize) -> &mut Self {
        self.x
            .iter_mut()
            .zip(self.y.iter_mut().zip(self.z.iter_mut()))
            .for_each(|(x, (y, z))| {
                let v: Vec<f64> = geotrans::m1_any_to_oss(sid, [*x, *y, *z]).into();
                *x = v[0];
                *y = v[1];
                *z = v[2];
            });
        self
    }
    /// Transforms the coordinates into the OSS
    pub fn from_local(&mut self, sid: usize) -> &mut Self {
        self.x
            .iter_mut()
            .zip(self.y.iter_mut().zip(self.z.iter_mut()))
            .for_each(|(x, (y, z))| {
                let v: Vec<f64> = geotrans::oss_to_any_m1(sid, [*x, *y, *z]).into();
                *x = v[0];
                *y = v[1];
                *z = v[2];
            });
        self
    }
    /// Returns the sum of the z forces of all the segments
    pub fn total_force(&self) -> f64 {
        self.pressure
            .iter()
            .zip(self.area.iter())
            .map(|(p, a)| -p * a)
            .sum()
    }
    /// Iterator over the pressures and areas
    pub fn pa_iter(&self) -> impl Iterator<Item = (&f64, &f64)> {
        self.pressure.iter().zip(self.area.iter())
    }
    /// Returns the z forces of a given segment
    pub fn forces(&mut self, sid: usize) -> Vec<f64> {
        let xy: Vec<_> = self
            .to_local(sid)
            .xy_iter()
            .map(|(x, y)| (*x, *y))
            .collect();
        self.from_local(sid)
            .pa_iter()
            .zip(xy)
            .filter(|(_, (x, y))| x.hypot(*y) < 4.5_f64)
            .map(|((p, a), _)| -p * a)
            .collect()
    }
    /// Returns the sum of the z forces of a given segment
    pub fn segment_force(&mut self, sid: usize) -> f64 {
        self.forces(sid).into_iter().sum()
    }
    /// Returns the sum of the z forces of a all the segments
    pub fn segments_force(&mut self) -> Vec<f64> {
        (1..=7)
            .map(|sid| self.forces(sid).into_iter().sum())
            .collect()
    }
}
