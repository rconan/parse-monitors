use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use bzip2::bufread::BzDecoder;
use serde::Deserialize;

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|&x| x * x).sum::<f64>().sqrt()
}

#[derive(Deserialize, Debug, PartialEq)]
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
impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.area.partial_cmp(&other.area)
    }
}
#[derive(Deserialize, Debug, PartialEq)]
struct GeometryRecord {
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
impl GeometryRecord {
    fn area_ijk(&self) -> [f64; 3] {
        [self.area_i, self.area_j, self.area_k]
    }
}
impl PartialOrd for GeometryRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        norm(&self.area_ijk()).partial_cmp(&norm(&other.area_ijk()))
    }
}
/// M1 segments surface pressure
#[derive(Default)]
pub struct Pressure {
    /// the segment surface pressure
    pressure: Vec<f64>,
    /// the area magnitude the pressure is applied to
    area: Vec<f64>,
    /// the area vector along the surface normal
    area_ijk: Vec<[f64; 3]>,
    /// the (x,y,z) coordinate where the pressure is applied
    xyz: Vec<[f64; 3]>,
}
impl Pressure {
    /// Loads the pressure data
    pub fn load(csv_pressure: String, csv_geometry: String) -> Result<Self, Box<dyn Error>> {
        let this_pa = Self::load_pressure(csv_pressure)?;
        let this_aijk = Self::load_geometry(csv_geometry)?;
        let max_diff_area = this_pa
            .area
            .iter()
            .zip(this_aijk.area_ijk.iter().map(|x| norm(x)))
            .map(|(a0, a1)| (*a0 - a1).abs())
            .fold(std::f64::NEG_INFINITY, f64::max);
        assert!(
            max_diff_area < 1e-14,
            "Area magnitude do no match area vector: {}",
            max_diff_area
        );
        Ok(Self {
            pressure: this_pa.pressure,
            area: this_pa.area,
            area_ijk: this_aijk.area_ijk,
            xyz: this_aijk.xyz,
        })
    }
    pub fn decompress(path: PathBuf) -> Result<String, Box<dyn Error>> {
        let csv_file = File::open(path)?;
        let buf = BufReader::new(csv_file);
        let mut bz2 = BzDecoder::new(buf);
        let mut contents = String::new();
        bz2.read_to_string(&mut contents)?;
        Ok(contents)
    }
    /// Loads the pressure from a csv bz2-compressed file
    pub fn load_pressure(contents: String) -> Result<Self, Box<dyn Error>> {
        let mut this = Pressure::default();
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());
        let mut rows = Vec::<Record>::new();
        for result in rdr.deserialize() {
            rows.push(result?);
        }
        rows.sort_by(|a, b| a.partial_cmp(b).unwrap());
        rows.into_iter().for_each(|row| {
            this.area.push(row.area);
            this.pressure.push(row.pressure);
        });
        Ok(this)
    }
    /// Loads the areas and coordinates vector from a csv file
    pub fn load_geometry(contents: String) -> Result<Self, Box<dyn Error>> {
        let mut this = Pressure::default();
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());
        let mut rows = Vec::<GeometryRecord>::new();
        for result in rdr.deserialize() {
            rows.push(result?);
        }
        rows.sort_by(|a, b| a.partial_cmp(b).unwrap());
        rows.into_iter().for_each(|row| {
            this.area_ijk.push([row.area_i, row.area_j, row.area_k]);
            this.xyz.push([row.x, row.y, row.z]);
        });
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
    /// Transforms the coordinates into the segment local coordinate system
    pub fn to_local(&mut self, sid: usize) -> &mut Self {
        self.xyz.iter_mut().for_each(|v| {
            *v = geotrans::m1_any_to_oss(sid, *v).into();
        });
        self
    }
    /// Transforms the coordinates into the OSS
    pub fn from_local(&mut self, sid: usize) -> &mut Self {
        self.xyz.iter_mut().for_each(|v| {
            *v = geotrans::oss_to_any_m1(sid, *v).into();
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
    /// Iterator over the pressures and area vectors
    pub fn paijk_iter(&self) -> impl Iterator<Item = (&f64, &[f64; 3])> {
        self.pressure.iter().zip(self.area_ijk.iter())
    }
    /// Iterator over the pressures, area vectors and coordinates
    pub fn p_aijk_xyz(&self) -> impl Iterator<Item = (&f64, &[f64; 3], &[f64; 3])> {
        self.pressure
            .iter()
            .zip(self.area_ijk.iter())
            .zip(self.xyz.iter())
            .map(|((a, b), c)| (a, b, c))
    }
    /// Returns the z forces of a given segment
    pub fn forces(&mut self, sid: usize) -> Vec<[f64; 3]> {
        let xy: Vec<_> = self.to_local(sid).xy_iter().map(|(x, y)| (x, y)).collect();
        self.from_local(sid)
            .paijk_iter()
            .zip(xy)
            .filter(|(_, (x, y))| x.hypot(*y) < 4.5_f64)
            .map(|((p, a), _)| [p * a[0], p * a[1], p * a[2]])
            .collect()
    }
    /// Returns the center of pressure of a given segment
    pub fn center_of_pressure(&mut self, sid: usize) -> [f64; 3] {
        let xy: Vec<_> = self.to_local(sid).xy_iter().map(|(x, y)| (x, y)).collect();
        let (mut cs, s) = self
            .from_local(sid)
            .p_aijk_xyz()
            .zip(xy)
            .filter(|(_, (x, y))| x.hypot(*y) < 4.5_f64)
            .fold(([0f64; 3], [0f64; 3]), |(mut cs, mut s), ((p, a, c), _)| {
                for k in 0..3 {
                    let df = p * a[k];
                    cs[k] += df * c[k];
                    s[k] += df;
                }
                (cs, s)
            });
        cs.iter_mut().zip(s).for_each(|(cs, s)| *cs /= s);
        cs
    }
    /// Returns the sum of the forces of a given segment
    pub fn segment_force(&mut self, sid: usize) -> [f64; 3] {
        self.forces(sid).into_iter().fold([0f64; 3], |mut s, a| {
            s.iter_mut().zip(a).for_each(|(s, a)| *s += a);
            s
        })
    }
    /// Returns the center of pressure and the force and moment applied at this location for a given segment
    pub fn segment_pressure_integral(&mut self, sid: usize) -> ([f64; 3], ([f64; 3], [f64; 3])) {
        let xy: Vec<_> = self.to_local(sid).xy_iter().map(|(x, y)| (x, y)).collect();
        let (mut cop, force) = self
            .from_local(sid)
            .p_aijk_xyz()
            .zip(xy)
            .filter(|(_, (x, y))| x.hypot(*y) < 4.5_f64)
            .fold(([0f64; 3], [0f64; 3]), |(mut cs, mut s), ((p, a, c), _)| {
                for k in 0..3 {
                    let df = p * a[k];
                    cs[k] += df * c[k];
                    s[k] += df;
                }
                (cs, s)
            });
        cop.iter_mut().zip(force).for_each(|(cs, s)| *cs /= s);
        let moment = [
            cop[1] * force[2] - cop[2] * force[1],
            cop[2] * force[0] - cop[0] * force[2],
            cop[0] * force[1] - cop[1] * force[0],
        ];
        (cop, (force, moment))
    }
    /// Returns the sum of the forces of a all the segments
    pub fn segments_force(&mut self) -> [f64; 3] {
        (1..=7)
            .map(|sid| self.segment_force(sid))
            .fold([0f64; 3], |mut s, a| {
                s.iter_mut().zip(a).for_each(|(s, a)| *s += a);
                s
            })
    }
    /// Returns the sum of the vectors in [`Iterator`]
    pub fn sum_vectors<'a>(&'a mut self, vec: impl Iterator<Item = &'a [f64; 3]>) -> [f64; 3] {
        vec.fold([0f64; 3], |mut s, a| {
            s.iter_mut().zip(a).for_each(|(s, a)| *s += a);
            s
        })
    }
}
