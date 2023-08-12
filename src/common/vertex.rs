
use std::{ops, f32::{consts::PI}, convert::TryFrom, fmt::{Display, Formatter, Result}, any::Any };
use crate::{common::engine::gamesys::*};
use serde::*;
use super::matrices::Vec4;

pub type Vertex = [f32; 3];
// Define methods


pub trait V3New<T> {
    fn new(_x: T, _y: T, _z: T) -> Vec3;
}

pub trait V3Meth {
    fn get(&self) -> [f32; 3];
    fn dot(&self, rhs: Vec3) -> f32;
    fn cross(&self, rhs: Vec3) -> Vec3;
    fn magnitude(&self) -> f32;
    fn divide(&self, a: f32) -> Vec3;
    fn times(&self, a: f32) -> Vec3;
    fn scale(&self, a: f32) -> Vec3;
    fn normalized(&self) -> Vec3;
    fn angle_to(&self, rhs: Vec3) -> f32;
    fn angle_v3(&self) -> Vec3;
}



// Define attributes
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Vec3 {
    pub x : f32,
    pub y : f32,
    pub z : f32
}

impl Base for Vec3{

}

impl Display for Vec3{
    fn fmt(&self, f: &mut Formatter) -> Result {
        return write!(f, "{}", format!("({}, {}, {})", self.x.to_string(), self.y.to_string(),  self.z.to_string()) );
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default(), z: Default::default() }
    }
}

impl V3New<f32> for Vec3 {
    fn new(_x: f32, _y: f32, _z: f32) -> Vec3 {
        return Vec3{x: _x, y: _y, z: _z};
    }
}

impl V3New<i16> for Vec3 {
    fn new(_x: i16, _y: i16, _z: i16) -> Vec3 {
        return Vec3::new(f32::from(_x), f32::from(_y), f32::from(_z));
    }
}

impl V3New<Option<i16>> for Vec3 {
    fn new(_x: Option<i16>, _y: Option<i16>, _z: Option<i16>) -> Vec3 {
        return Vec3::new(_x.unwrap(), _y.unwrap(), _z.unwrap());
    }
}

impl V3New<i32> for Vec3 {
    fn new(_x: i32, _y: i32, _z: i32) -> Vec3 {
        return Vec3::new(i16::try_from(_x).ok(), i16::try_from(_y).ok(), i16::try_from(_z).ok());
    }
}
mod mathf {

    pub fn sin(a: f32) -> f32{
        return a.sin();
    }

    pub fn cos(a: f32) -> f32 {
        return a.cos();
    }

    pub fn tan(a: f32) -> f32{
        return a.tan();
    }

    pub fn asin(a: f32) -> f32{
        return a.asin();
    }

    pub fn acos(a: f32) -> f32{
        return a.acos();
    }

    pub fn atan(a: f32) -> f32 {
        return a.atan();
    }

    pub fn atan2(a: f32, b: f32) -> f32{
    return a.atan2(b);
}
}

impl V3Meth for Vec3 {
    fn get(&self) -> [f32; 3]{
        return [self.x,self.y,self.z];
    }

    fn dot(&self, rhs: Vec3) -> f32{
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
    }

    fn cross(&self, rhs: Vec3) -> Vec3{
        let _x = self.y * rhs.z - self.z * rhs.y;
        let _y = -(self.x * rhs.z - self.z * rhs.x);
        let _z = self.x * rhs.y - self.y * rhs.x;
        return Vec3 {x: _x, y: _y, z: _z};
    }

    fn magnitude(&self) -> f32 {
        return (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt();
    }

    fn divide(&self, a: f32) -> Vec3 {
        return Vec3 {x: self.x / a, y: self.y / a, z: self.z / a};
    }

    fn times(&self, a: f32) -> Vec3 {
        return Vec3 {x: self.x * a, y: self.y * a, z: self.z * a};
    }

    fn scale(&self, a: f32) -> Vec3 {
        return self.times(a);
    }

    fn normalized(&self) -> Vec3 {
        let mag = self.magnitude();
        return self.divide(mag);
    }

    fn angle_to(&self, rhs: Vec3) -> f32 {
        let dot = self.dot(rhs);
        let mag_a = self.magnitude();
        let mag_b = rhs.magnitude();

        return mathf::acos(dot / (mag_a * mag_b));
    }

    fn angle_v3(&self) -> Vec3 {
        let normal = self.normalized();
        
        // Get angle in x-axis (left)
        let up = Vec3::new(0,0,1); //[x, z]
        let forward = Vec3::new(0,1,0); //[x, y] [z, y]
        
        let mut yaw = forward.angle_to(Vec3::new(normal.x, normal.y, 0.0));
        let mut pitch = up.angle_to(Vec3::new(0.0, normal.y, normal.z));
        let mut roll = forward.angle_to(Vec3::new(normal.x, 0.0, normal.z));
        
        if normal.x < 0.0{
            yaw = 2.0*PI - yaw;
        }
        if normal.y < 0.0{
            pitch = 2.0*PI - pitch;
        }
        if normal.z < 0.0{
            roll = 2.0*PI - roll;
        }

        return Vec3::new(yaw, pitch, roll);
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        return Vec3 {x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z};
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs:Vec3) -> Vec3 {
        return Vec3 {x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z};
    }
}

impl ops::Mul<Vec3> for Vec3{
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        return Vec3 {x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z};
    }
}


pub trait Vec3Buffer {
    fn to_buffer(&self) -> Vertex;
}

impl Vec3Buffer for Vec3 {
    fn to_buffer(&self) -> Vertex {
        return [self.x, self.y, self.x];
    }
}

pub trait Vec4Buffer {
    fn to_buffer(&self) -> [f32; 4];
}

impl Vec4Buffer for Vec4 {
    fn to_buffer(&self) -> [f32; 4] {
        return [self.x, self.y, self.z, self.w];
    }
}