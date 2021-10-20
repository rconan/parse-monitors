use std::{
    fmt,
    ops::{Add, AddAssign, Div, Sub},
};

#[derive(Default, Debug, Clone)]
pub struct Vector {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}
impl Vector {
    pub fn zero() -> Self {
        Self {
            x: Some(0f64),
            y: Some(0f64),
            z: Some(0f64),
        }
    }
    pub fn magnitude(&self) -> Option<f64> {
        if let Some((x, y, z)) = self.as_tuple() {
            Some((x * x + y * y + z * z).sqrt())
        } else {
            None
        }
    }
}
impl Vector {
    pub fn from_x(value: f64) -> Self {
        Self {
            x: Some(value),
            ..Default::default()
        }
    }
    pub fn from_y(value: f64) -> Self {
        Self {
            y: Some(value),
            ..Default::default()
        }
    }
    pub fn from_z(value: f64) -> Self {
        Self {
            z: Some(value),
            ..Default::default()
        }
    }
    pub fn as_tuple(&self) -> Option<(&f64, &f64, &f64)> {
        match self {
            Vector {
                x: Some(a1),
                y: Some(a2),
                z: Some(a3),
            } => Some((a1, a2, a3)),
            _ => None,
        }
    }
    pub fn into_tuple(self) -> Option<(f64, f64, f64)> {
        match self {
            Vector {
                x: Some(a1),
                y: Some(a2),
                z: Some(a3),
            } => Some((a1, a2, a3)),
            _ => None,
        }
    }
    pub fn as_array(&self) -> [&f64; 3] {
        match self {
            Vector {
                x: Some(a1),
                y: Some(a2),
                z: Some(a3),
            } => Ok([a1, a2, a3]),
            _ => Err(""),
        }
        .unwrap()
    }
    pub fn cross(&self, other: &Vector) -> Option<Vector> {
        if let (Some((a1, a2, a3)), Some((b1, b2, b3))) = (self.as_tuple(), other.as_tuple()) {
            Some(Vector {
                x: Some(a2 * b3 - a3 * b2),
                y: Some(a3 * b1 - a1 * b3),
                z: Some(a1 * b2 - a2 * b1),
            })
        } else {
            None
        }
    }
    pub fn norm_squared(&self) -> Option<f64> {
        if let Some((a1, a2, a3)) = self.as_tuple() {
            Some(a1 * a1 + a2 * a2 + a3 * a3)
        } else {
            None
        }
    }
}
impl Sub for &Vector {
    type Output = Option<Vector>;

    fn sub(self, rhs: Self) -> Self::Output {
        if let (Some((a1, a2, a3)), Some((b1, b2, b3))) = (self.as_tuple(), rhs.as_tuple()) {
            Some(Vector {
                x: Some(a1 - b1),
                y: Some(a2 - b2),
                z: Some(a3 - b3),
            })
        } else {
            None
        }
    }
}
impl Add for &Vector {
    type Output = Option<Vector>;

    fn add(self, rhs: Self) -> Self::Output {
        if let (Some((a1, a2, a3)), Some((b1, b2, b3))) = (self.as_tuple(), rhs.as_tuple()) {
            Some(Vector {
                x: Some(a1 + b1),
                y: Some(a2 + b2),
                z: Some(a3 + b3),
            })
        } else {
            None
        }
    }
}
impl AddAssign<&Vector> for &mut Vector {
    fn add_assign(&mut self, other: &Vector) {
        if let (Some((a1, a2, a3)), Some((b1, b2, b3))) = (self.as_tuple(), other.as_tuple()) {
            **self = Vector {
                x: Some(a1 + b1),
                y: Some(a2 + b2),
                z: Some(a3 + b3),
            }
        }
    }
}
impl Div<f64> for Vector {
    type Output = Option<Self>;

    fn div(self, rhs: f64) -> Self::Output {
        if let Some((a1, a2, a3)) = self.as_tuple() {
            Some(Vector {
                x: Some(a1 / rhs),
                y: Some(a2 / rhs),
                z: Some(a3 / rhs),
            })
        } else {
            None
        }
    }
}
impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:6.3?},{:6.3?},{:6.3?}]", self.x, self.y, self.z)
    }
}
impl From<[f64; 3]> for Vector {
    fn from(v: [f64; 3]) -> Self {
        Vector {
            x: Some(v[0]),
            y: Some(v[1]),
            z: Some(v[2]),
        }
    }
}
