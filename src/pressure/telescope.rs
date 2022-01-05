use super::PressureError;
use flate2::read::GzDecoder;
use itertools::Itertools;
use itertools::MinMaxResult::MinMax;
use serde::Deserialize;
use std::cmp::Ordering;
use std::fmt::Display;
use std::{fs::File, io::Read, path::Path};

fn partition(data: &[f64]) -> Option<(Vec<f64>, f64, Vec<f64>)> {
    match data.len() {
        0 => None,
        _ => {
            let (pivot_slice, tail) = data.split_at(1);
            let pivot = pivot_slice[0];
            let (left, right) = tail.iter().fold((vec![], vec![]), |mut splits, next| {
                {
                    let (ref mut left, ref mut right) = &mut splits;
                    if next < &pivot {
                        left.push(*next);
                    } else {
                        right.push(*next);
                    }
                }
                splits
            });

            Some((left, pivot, right))
        }
    }
}

fn select(data: &[f64], k: usize) -> Option<f64> {
    let part = partition(data);

    match part {
        None => None,
        Some((left, pivot, right)) => {
            let pivot_idx = left.len();

            match pivot_idx.cmp(&k) {
                Ordering::Equal => Some(pivot),
                Ordering::Greater => select(&left, k),
                Ordering::Less => select(&right, k - (pivot_idx + 1)),
            }
        }
    }
}

fn median(data: &[f64]) -> Option<f64> {
    let size = data.len();

    match size {
        even if even % 2 == 0 => {
            let fst_med = select(data, (even / 2) - 1);
            let snd_med = select(data, even / 2);

            match (fst_med, snd_med) {
                (Some(fst), Some(snd)) => Some((fst + snd) / 2.0),
                _ => None,
            }
        }
        odd => select(data, odd / 2),
    }
}

type Result<T> = std::result::Result<T, PressureError>;

#[derive(Deserialize, Debug)]
struct Record {
    #[serde(rename = "Pressure (Pa)")]
    pressure: f64,
    #[serde(rename = "Area in TCS[i] (m^2)")]
    area_i: f64,
    #[serde(rename = "Area in TCS[j] (m^2)")]
    area_j: f64,
    #[serde(rename = "Area in TCS[k] (m^2)")]
    area_k: f64,
    #[serde(rename = "X (m)")]
    x: f64,
    #[serde(rename = "Y (m)")]
    y: f64,
    #[serde(rename = "Z (m)")]
    z: f64,
}

#[derive(Default)]
pub struct Telescope {
    // The pressure file
    pub filename: String,
    // the segment surface pressure [Pa]
    pub pressure: Vec<f64>,
    // the area vector along the surface normal
    pub area_ijk: Vec<[f64; 3]>,
    // the (x,y,z) coordinate where the pressure is applied
    pub xyz: Vec<[f64; 3]>,
}
impl Telescope {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data_path = Path::new(path.as_ref());
        let mut contents = String::new();
        let csv_file = File::open(&data_path)?;
        let mut gz = GzDecoder::new(csv_file);
        gz.read_to_string(&mut contents)?;
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());
        let mut telescope: Telescope = Default::default();
        telescope.filename =
            String::from(data_path.file_name().map(|x| x.to_str()).flatten().unwrap());
        for result in rdr.deserialize() {
            let row: Record = result?;
            telescope.pressure.push(row.pressure);
            telescope
                .area_ijk
                .push([row.area_i, row.area_j, row.area_k]);
            telescope.xyz.push([row.x, row.y, row.z]);
        }
        Ok(telescope)
    }
    pub fn len(&self) -> usize {
        self.pressure.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn pressure_iter(&self) -> impl Iterator<Item = &f64> {
        self.pressure.iter()
    }
    pub fn mean_pressure(&self) -> f64 {
        self.pressure_iter().sum::<f64>() / self.len() as f64
    }
    pub fn median_pressure(&self) -> Option<f64> {
        median(&self.pressure)
    }
    pub fn minmax_pressure(&self) -> Option<(f64, f64)> {
        match self.pressure_iter().minmax() {
            MinMax(x, y) => Some((*x, *y)),
            _ => None,
        }
    }
    pub fn area_ijk_iter(&self) -> impl Iterator<Item = &[f64; 3]> {
        self.area_ijk.iter()
    }
    pub fn xyz_iter(&self) -> impl Iterator<Item = &[f64; 3]> {
        self.xyz.iter()
    }
    pub fn x_iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.xyz.iter().map(|xyz| xyz[0])
    }
    pub fn y_iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.xyz.iter().map(|xyz| xyz[1])
    }
    pub fn z_iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.xyz.iter().map(|xyz| xyz[2])
    }
    pub fn minmax_x(&self) -> Option<(f64, f64)> {
        match self.x_iter().minmax() {
            MinMax(x, y) => Some((x, y)),
            _ => None,
        }
    }
    pub fn minmax_y(&self) -> Option<(f64, f64)> {
        match self.y_iter().minmax() {
            MinMax(x, y) => Some((x, y)),
            _ => None,
        }
    }
    pub fn minmax_z(&self) -> Option<(f64, f64)> {
        match self.z_iter().minmax() {
            MinMax(x, y) => Some((x, y)),
            _ => None,
        }
    }
    pub fn area_mag(&self) -> Vec<f64> {
        self.area_ijk
            .iter()
            .map(|ijk| ijk.iter().map(|x| x * x).sum::<f64>().sqrt())
            .collect()
    }
    pub fn total_area(&self) -> f64 {
        self.area_mag().into_iter().sum::<f64>()
    }
}
impl Display for Telescope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} [{:#}]:", self.filename, self.len())?;
        writeln!(f, " - pressure:")?;
        writeln!(f, "  - mean  : {:.3}pa", self.mean_pressure())?;
        writeln!(f, "  - median: {:.3}pa", self.median_pressure().unwrap())?;
        self.minmax_pressure()
            .map(|x| writeln!(f, "  - minmax: {:.3?}pa", x))
            .transpose()?;
        writeln!(f, " - total area: {:.3}m^2", self.total_area())?;
        writeln!(f, " - volume:")?;
        self.minmax_x()
            .map(|x| writeln!(f, "  - x minmax: {:.3?}m", x))
            .transpose()?;
        self.minmax_y()
            .map(|x| writeln!(f, "  - y minmax: {:.3?}m", x))
            .transpose()?;
        self.minmax_z()
            .ok_or(std::fmt::Error)
            .map(|x| write!(f, "  - z minmax: {:.3?}m", x))?
    }
}
