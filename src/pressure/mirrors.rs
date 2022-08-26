//! M1 and M2 segments surface pressure
//!
//! Analyze segments surface wind pressure from pressure files either *M1p_M1p_\*.csv.bz2* or
//! *M2p_M2p_\*.csv.bz2* for M1 or M2, respectively.

use super::PressureError;
use geotrans::{Segment, SegmentTrait, Transform, TransformMut, M1, M2};
use serde::Deserialize;
use std::{fs::File, io::Read, marker::PhantomData, path::PathBuf};

type Result<T> = std::result::Result<T, PressureError>;

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|&x| x * x).sum::<f64>().sqrt()
}

#[derive(Deserialize, Debug, PartialEq)]
struct Record {
    #[serde(rename = "Area in TCS[i] (m^2)")]
    area_i: f64,
    #[serde(rename = "Area in TCS[j] (m^2)")]
    area_j: f64,
    #[serde(rename = "Area in TCS[k] (m^2)")]
    area_k: f64,
    //#[serde(rename = "Area: Magnitude (m^2)")]
    //area: f64,
    #[serde(rename = "Pressure (Pa)")]
    pressure: f64,
    #[serde(rename = "X (m)")]
    x: f64,
    #[serde(rename = "Y (m)")]
    y: f64,
    #[serde(rename = "Z (m)")]
    z: f64,
}
/*impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.area.partial_cmp(&other.area)
    }
}*/
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
pub struct Pressure<M>
where
    M: Default,
    Segment<M>: SegmentTrait,
{
    // the segment surface pressure [Pa]
    pressure: Vec<f64>,
    // the area magnitude the pressure is applied to
    area: Vec<f64>,
    // the area vector along the surface normal
    area_ijk: Vec<[f64; 3]>,
    // the (x,y,z) coordinate where the pressure is applied
    xyz: Vec<[f64; 3]>,
    // segment data filter
    segment_filter: Vec<Vec<bool>>,
    // segment data filter
    segment_filter_size: Vec<f64>,
    mirror: PhantomData<M>,
}
pub trait MirrorProperties {
    fn exo_radius(&self) -> f64;
    fn mirror(&self) -> String;
    fn center_hole(&self) -> Option<f64>;
}
impl MirrorProperties for Pressure<M1> {
    fn exo_radius(&self) -> f64 {
        4.5
    }
    fn mirror(&self) -> String {
        String::from("M1")
    }
    fn center_hole(&self) -> Option<f64> {
        Some(2.75 * 0.5)
    }
}
impl MirrorProperties for Pressure<M2> {
    fn exo_radius(&self) -> f64 {
        0.55
    }
    fn mirror(&self) -> String {
        String::from("M2")
    }
    fn center_hole(&self) -> Option<f64> {
        None
    }
}
impl<M> Pressure<M>
where
    M: Default,
    Segment<M>: SegmentTrait,
    Pressure<M>: MirrorProperties,
{
    /// Loads the pressure data
    pub fn load(csv_pressure: String) -> Result<Self> {
        let this_pa = Self::load_pressure(csv_pressure)?;
        /*let this_aijk = Self::load_geometry(csv_geometry)?;
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
        );*/
        let mut this = Self {
            pressure: this_pa.pressure,
            area: this_pa
                .area_ijk
                .iter()
                .map(|a| a.iter().fold(0f64, |s, &a| s + a * a).sqrt())
                .collect(),
            area_ijk: this_pa.area_ijk,
            xyz: this_pa.xyz,
            segment_filter: Vec::new(),
            segment_filter_size: Vec::new(),
            mirror: PhantomData,
        };
        let mut n = 0;
        let xr = this.exo_radius();
        for sid in 1..=7 {
            let sf = this
                .to_local(sid)?
                .xy_iter()
                .map(|(x, y)| x.hypot(y) < xr)
                .collect::<Vec<bool>>();
            let n_sf = sf
                .iter()
                .filter_map(|sf| if *sf { Some(1usize) } else { None })
                .sum::<usize>();
            this.segment_filter.push(sf);
            this.segment_filter_size.push(n_sf as f64);
            this.from_local(sid);
            n += n_sf;
        }
        assert!(
            n == this.pressure.len(),
            "Data length ({}) and segment filter length ({}) do not match",
            this.pressure.len(),
            n
        );
        Ok(this)
    }
    #[cfg(feature = "bzip2")]
    pub fn decompress(path: PathBuf) -> Result<String> {
        let csv_file = File::open(path)?;
        let buf = std::io::BufReader::new(csv_file);
        let mut bz2 = bzip2::bufread::BzDecoder::new(buf);
        let mut contents = String::new();
        bz2.read_to_string(&mut contents)?;
        Ok(contents)
    }
    #[cfg(not(feature = "bzip2"))]
    pub fn decompress(path: PathBuf) -> Result<String> {
        let csv_file = File::open(path)?;
        let mut gz = flate2::read::GzDecoder::new(csv_file);
        let mut contents = String::new();
        gz.read_to_string(&mut contents)?;
        Ok(contents)
    }
    /// Loads the pressure from a csv bz2-compressed file
    pub fn load_pressure(contents: String) -> Result<Self> {
        let mut this = Pressure::default();
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());
        let mut rows = Vec::<Record>::new();
        for result in rdr.deserialize() {
            rows.push(result?);
        }
        //rows.sort_by(|a, b| a.partial_cmp(b).unwrap());
        rows.into_iter().for_each(|row| {
            //this.area.push(row.area);
            this.pressure.push(row.pressure);
            this.area_ijk.push([row.area_i, row.area_j, row.area_k]);
            this.xyz.push([row.x, row.y, row.z]);
        });
        Ok(this)
    }
    /// Loads the areas and coordinates vector from a csv file
    pub fn load_geometry(contents: String) -> Result<Self> {
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
    pub fn len(&self) -> usize {
        self.pressure.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    /// Iterator over the (x,y) coordinates of a given segment
    pub fn segment_xy(&self, sid: usize) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.x_iter()
            .zip(self.y_iter())
            .zip(self.segment_filter.get(sid - 1).unwrap().iter())
            .filter(|(_, &f)| f)
            .map(|(xy, _)| xy)
    }
    /// Iterator over the pressure and areas of a given segment
    pub fn segment_pa(&self, sid: usize) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.pa_iter()
            .zip(self.segment_filter.get(sid - 1).unwrap().iter())
            .filter_map(|((p, a), &f)| f.then(|| (*p, *a)))
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
    pub fn to_local(&mut self, sid: usize) -> Result<&mut Self> {
        self.xyz
            .iter_mut()
            .map(|v| v.fro(Segment::<M>::new(sid as i32)))
            .collect::<std::result::Result<Vec<()>, geotrans::Error>>()?;
        Ok(self)
    }
    /// Transforms the coordinates into the OSS
    pub fn from_local(&mut self, sid: usize) -> &mut Self {
        self.xyz.iter_mut().for_each(|v| {
            v.to(Segment::<M>::new(sid as i32)).unwrap();
        });
        self
    }
    /// Filter that is apply locally to segment `sid` selecting samplea within the radii `r_in`<0> and `r_out`<`exo_radius`>
    pub fn local_radial_filter(
        &self,
        sid: usize,
        r_in: Option<f64>,
        r_out: Option<f64>,
    ) -> impl Iterator<Item = bool> + '_ {
        let xr = self.exo_radius();
        self.xyz
            .iter()
            .map(move |v| v.fro(Segment::<M>::new(sid as i32)).unwrap())
            .map(|v| v[0].hypot(v[1]))
            .map(move |r| r >= r_in.unwrap_or_default() && r < r_out.unwrap_or(xr))
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
    /// Iterator over the pressures, area vectors and coordinates for the given segment
    pub fn segment_p_aijk_xyz(
        &self,
        sid: usize,
    ) -> impl Iterator<Item = (&f64, &[f64; 3], &[f64; 3])> {
        self.pressure
            .iter()
            .zip(self.area_ijk.iter())
            .zip(self.xyz.iter())
            .zip(self.segment_filter.get(sid - 1).unwrap().iter())
            .filter_map(|(((a, b), c), f)| f.then(|| (a, b, c)))
    }
    /// Return the area of a given segment
    pub fn segment_area(&self, sid: usize) -> f64 {
        self.area
            .iter()
            .zip(self.segment_filter.get(sid - 1).unwrap())
            .filter(|(_, &f)| f)
            .fold(0f64, |aa, (a, _)| aa + *a)
    }
    /// Return the area of each segment
    pub fn segments_area(&self) -> Vec<f64> {
        (1..=7).map(|sid| self.segment_area(sid)).collect()
    }
    /// Return the mirror area
    pub fn mirror_area(&self) -> f64 {
        self.area.iter().fold(0f64, |aa, a| aa + *a)
    }
    /// Returns the vectors of forces of a given segment
    pub fn forces(&mut self, sid: usize) -> Result<Vec<[f64; 3]>> {
        let xy: Vec<_> = self.to_local(sid)?.xy_iter().map(|(x, y)| (x, y)).collect();
        let xr = self.exo_radius();
        Ok(self
            .from_local(sid)
            .paijk_iter()
            .zip(xy)
            .filter(|(_, (x, y))| x.hypot(*y) < xr)
            .map(|((p, a), _)| [p * a[0], p * a[1], p * a[2]])
            .collect())
    }
    /// Returns the average pressure over a given segment
    pub fn average_pressure(&self, sid: usize) -> f64 {
        let (pa, aa) = self
            .pa_iter()
            .zip(self.segment_filter.get(sid - 1).unwrap().iter())
            .filter(|(_, &f)| f)
            .fold((0f64, 0f64), |(pa, aa), ((p, a), _)| (pa + p * a, aa + a));
        pa / aa
    }
    /// Returns the average ASM different pressure over a given segment
    pub fn asm_differential_pressure(&self, sid: usize) -> (f64, Vec<f64>) {
        let p_mean = self.average_pressure(sid);
        (
            p_mean,
            self.pressure
                .iter()
                .zip(self.segment_filter.get(sid - 1).unwrap().iter())
                .filter_map(|(&p, &f)| f.then(|| (p - 1.1 * p_mean) / 3.))
                .collect::<Vec<f64>>(),
        )
    }
    /// Returns the pressure variance over a given segment
    pub fn pressure_var(&mut self, sid: usize) -> f64 {
        let p_mean = self.average_pressure(sid);
        let (pa, aa) = self
            .pa_iter()
            .zip(self.segment_filter.get(sid - 1).unwrap().iter())
            .filter(|(_, &f)| f)
            .fold((0f64, 0f64), |(pa, aa), ((p, a), _)| {
                let p_net = p - p_mean;
                (pa + p_net * p_net * a, aa + a)
            });
        pa / aa
    }
    /// Returns the pressure standard deviation over a given segment
    pub fn pressure_std(&mut self, sid: usize) -> f64 {
        self.pressure_var(sid).sqrt()
    }
    /// Returns the average pressure over each segment
    pub fn segments_average_pressure(&mut self) -> Vec<f64> {
        (1..=7).map(|sid| self.average_pressure(sid)).collect()
    }
    /// Returns the pressure variance over each segment
    pub fn segments_pressure_var(&mut self) -> Vec<f64> {
        (1..=7).map(|sid| self.pressure_var(sid)).collect()
    }
    /// Returns the pressure standard deviation over each segment
    pub fn segments_pressure_std(&mut self) -> Vec<f64> {
        (1..=7).map(|sid| self.pressure_std(sid)).collect()
    }
    /// Returns the average mean pressure over all segment
    pub fn mirror_average_pressure(&mut self) -> f64 {
        let (pa, aa) = self
            .pa_iter()
            .fold((0f64, 0f64), |(pa, aa), (p, a)| (pa + p * a, aa + a));
        pa / aa
    }
    /// Returns the average pressure variance over all segment
    pub fn mirror_average_pressure_var(&mut self) -> f64 {
        let p_mean = self.mirror_average_pressure();
        let (pa, aa) = self.pa_iter().fold((0f64, 0f64), |(pa, aa), (p, a)| {
            let p_net = p - p_mean;
            (pa + p_net * p_net * a, aa + a)
        });
        pa / aa
    }
    /// Returns the average pressure standart deviation over all segment
    pub fn mirror_average_pressure_std(&mut self) -> f64 {
        self.mirror_average_pressure_var().sqrt()
    }
    /// Returns the average mean pressure over all segment within the given `radius`
    pub fn mirror_average_pressure_within(&self, radius: f64) -> f64 {
        let (pa, aa) = self
            .pa_iter()
            .zip(self.xy_iter())
            .filter(|(_, (x, y))| x.hypot(*y) < radius)
            .fold((0f64, 0f64), |(pa, aa), ((p, a), _)| (pa + p * a, aa + a));
        pa / aa
    }
    /// Returns the average mean pressure over all segment with data filtered with `select`
    pub fn mirror_average_pressure_by(&self, select: impl Iterator<Item = bool>) -> f64 {
        let (pa, aa) = self
            .pa_iter()
            .zip(select)
            .filter(|(_, m)| *m)
            .fold((0f64, 0f64), |(pa, aa), ((p, a), _)| (pa + p * a, aa + a));
        pa / aa
    }
    /// Returns the average pressure variance over all segment within the given `radius`
    pub fn mirror_average_pressure_var_within(&self, radius: f64) -> f64 {
        let p_mean = self.mirror_average_pressure_within(radius);
        let (pa, aa) = self
            .pa_iter()
            .zip(self.xy_iter())
            .filter(|(_, (x, y))| x.hypot(*y) < radius)
            .fold((0f64, 0f64), |(pa, aa), ((p, a), _)| {
                let p_net = p - p_mean;
                (pa + p_net * p_net * a, aa + a)
            });
        pa / aa
    }
    /// Returns the average pressure variance over all segment with data filtered with `select`
    pub fn mirror_average_pressure_var_by(
        &self,
        p_mean: f64,
        select: impl Iterator<Item = bool>,
    ) -> f64 {
        //        let b: Vec<_> = select.collect();
        let (pa, aa) = self.pa_iter().zip(select).filter(|(_, m)| *m).fold(
            (0f64, 0f64),
            |(pa, aa), ((p, a), _)| {
                let p_net = p - p_mean;
                (pa + p_net * p_net * a, aa + a)
            },
        );
        pa / aa
    }
    /// Returns the center of pressure of a given segment
    pub fn center_of_pressure(&mut self, sid: usize) -> Result<[f64; 3]> {
        let xy: Vec<_> = self.to_local(sid)?.xy_iter().map(|(x, y)| (x, y)).collect();
        let xr = self.exo_radius();
        let (mut cs, s) = self
            .from_local(sid)
            .p_aijk_xyz()
            .zip(xy)
            .filter(|(_, (x, y))| x.hypot(*y) < xr)
            .fold(([0f64; 3], [0f64; 3]), |(mut cs, mut s), ((p, a, c), _)| {
                for k in 0..3 {
                    let df = p * a[k];
                    cs[k] += df * c[k];
                    s[k] += df;
                }
                (cs, s)
            });
        cs.iter_mut().zip(s).for_each(|(cs, s)| *cs /= s);
        Ok(cs)
    }
    /// Returns the sum of the forces of a given segment
    pub fn segment_force(&mut self, sid: usize) -> Result<[f64; 3]> {
        Ok(self.forces(sid)?.into_iter().fold([0f64; 3], |mut s, a| {
            s.iter_mut().zip(a).for_each(|(s, a)| *s += a);
            s
        }))
    }
    /// Returns the center of pressure and the force and moment applied at this location for a given segment
    pub fn segment_pressure_integral(
        &mut self,
        sid: usize,
    ) -> Result<([f64; 3], ([f64; 3], [f64; 3]))> {
        let xy: Vec<_> = self.to_local(sid)?.xy_iter().map(|(x, y)| (x, y)).collect();
        let xr = self.exo_radius();
        let (mut cop, force) = self
            .from_local(sid)
            .p_aijk_xyz()
            .zip(xy)
            .filter(|(_, (x, y))| x.hypot(*y) < xr)
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
        Ok((cop, (force, moment)))
    }
    /// Returns the sum of the forces of all the segments
    pub fn segments_force(&mut self) -> Result<[f64; 3]> {
        Ok((1..=7)
            .map(|sid| self.segment_force(sid))
            .collect::<Result<Vec<[f64; 3]>>>()?
            .into_iter()
            .fold([0f64; 3], |mut s, a| {
                s.iter_mut().zip(a).for_each(|(s, a)| *s += a);
                s
            }))
    }
    /// Returns the sum of the vectors in [`Iterator`]
    pub fn sum_vectors<'a>(&'a mut self, vec: impl Iterator<Item = &'a [f64; 3]>) -> [f64; 3] {
        vec.fold([0f64; 3], |mut s, a| {
            s.iter_mut().zip(a).for_each(|(s, a)| *s += a);
            s
        })
    }
    #[cfg(feature = "plot")]
    /// Display the pressure map
    pub fn pressure_map(&mut self, cfd_case_path: PathBuf) {
        let mut triangles = vec![];
        let mut tri_pressure = vec![];
        /*let average_pressure = self.mirror_average_pressure();
        let min_pressure = self.pressure.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_pressure = self
            .pressure
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);*/
        for sid in 1..=7 {
            let mut del = triangle_rs::Delaunay::builder()
                .add_nodes(
                    &self
                        .segment_xy(sid)
                        .flat_map(|(x, y)| vec![x, y])
                        .collect::<Vec<f64>>(),
                )
                .set_switches("Q")
                .build();
            match (sid, self.center_hole()) {
                (7, Some(radius)) => {
                    del.filter_within_circle(radius, None);
                }
                _ => (),
            }
            let pa: Vec<_> = self
                .segment_pa(sid)
                //                .map(|(p, a)| (if p < 0f64 { f64::NAN } else { p }, a))
                .collect();
            tri_pressure.extend(del.triangle_iter().map(|t| {
                t.iter().fold(0f64, |s, i| s + pa[*i].0 * pa[*i].1)
                    / t.iter().fold(0f64, |s, i| s + pa[*i].1)
            }));
            triangles.extend(del.triangle_vertex_iter());
        }
        let path = cfd_case_path
            .join("report")
            .join(format!("{}_pressure_map.png", self.mirror().to_lowercase()));
        let filename = format!("{}", path.as_path().display());
        let _: complot::tri::Heatmap = (
            triangles.into_iter().zip(tri_pressure.into_iter()),
            complot::complot!(filename),
        )
            .into();
    }
}
