#![allow(unused)]

use std::{f32::consts::PI, fmt::Display};

#[allow(unused_imports)]
use crate::black_ice::common::vertex::*;
use super::engine::gamesys::Base;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct Ang3 {
    pub y : f32,
    pub p : f32,
    pub r : f32
}

impl Display for Ang3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ang3({x}, {y}, {z})", x=self.y, y=self.p, z=self.r)
    }
}

impl Ang3 {

    pub fn new(y: f32, p: f32, r:f32) -> Ang3 {
        Ang3 { 
            y: y % 360.0, 
            p: p % 360.0, 
            r: r % 360.0
        }
    }
}


#[derive(Copy, Clone)]
pub struct Quat {
    pub x : f32,
    pub y : f32,
    pub z : f32,
    pub w : f32
}

impl Base for Ang3 {}
impl Base for Quat {}

impl Default for Quat {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default(), z: Default::default(), w: Default::default() }
    }
}

pub trait QuatConstructor<T> {
    fn new(x: T, y: T, z: T, w: T) -> Quat;
}

impl QuatConstructor<f32> for Quat {
    fn new(x: f32, y: f32, z: f32, w: f32) -> Quat {
        return Quat { x: x, y: y, z: z, w: w };
    }
}

pub trait QuaternionMath {



}

impl Display for Quat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Quat({x}, {y}, {z}, {w})", x=self.x, y=self.y, z=self.z, w=self.w)
    }
}

impl Quat {

    /// ypr as degrees!!
    pub fn euler(ang: Ang3) -> Self
    {
        let y = ang.y;
        let p = ang.p;
        let r = ang.r;
        // let cr = (r * deg_to_rad * 0.5).cos();
        // let sr = (r * deg_to_rad * 0.5).sin();
        // let cp = (p * deg_to_rad * 0.5).cos();
        // let sp = (p * deg_to_rad * 0.5).sin();
        // let cy = (y * deg_to_rad * 0.5).cos();
        // let sy = (y * deg_to_rad * 0.5).sin();
        // Self { 
        //     x: cr * cp * cy + sr * sp * sy, 
        //     y: sr * cp * cy - cr * sp * sy, 
        //     z: cr * sp * cy + sr * cp * sy, 
        //     w: cr * cp * sy - sr * sp * cy 
        // }

        let _x = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), y.to_radians());
        let _y = Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), p.to_radians());
        let _z = Quat::axis_angle(Vec3::new(1.0, 0.0, 0.0), r.to_radians());

        
        let tmp = _x * _y * _z;

        // println!("{}", tmp.to_euler());
        tmp
    }

    pub fn axis_angle(vector: Vec3, angle: f32) -> Self
    {
        let s = (angle / 2.0).sin();
        let c = (angle / 2.0).cos();

        Self { x: vector.x * s, y: vector.y * s, z: vector.z * s, w: c }
    }

    pub fn mult(&mut self, rhs: Self){
        
        self.x = self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y;
        self.y = self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x;
        self.z = self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w;
        self.w = self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z;
    
    }

    pub fn conjugate(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
    }

    pub fn to_euler(&self) -> Ang3 {
        Ang3 {
            y: (
                (2.0 * (self.w * self.z + self.y * self.x)) / (1.0 - 2.0 *(self.z.powi(2) + self.y.powi(2)))
            ).atan().to_degrees(),
            p: ( 
                {
                    let v = 2.0 * (self.w * self.y - self.z * self.x);
                    if v > 1.0 {
                        1.0
                    }
                    else if v < -1.0 {
                        -1.0
                    }
                    else {
                        v
                    }
                }
            ).asin().to_degrees(),
            r: (
                (2.0 * (self.w * self.x + self.z * self.y)) / (1.0 - 2.0 * (self.y.powi(2) + self.x.powi(2)))
            ).atan().to_degrees()
        }
    }

    /// simply gets the axis for the rotation
    pub fn as_vec3(&self) -> Vec3 {
        let v = Vec3::new(self.x, self.y, self.z);
        v
    }
}


impl std::ops::Mul for Quat {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut this = self.clone();
        this.mult(rhs);

        this
    }
}
impl std::ops::MulAssign for Quat {
    fn mul_assign(&mut self, rhs: Self) {
        self.mult(rhs);
    }
}

impl std::ops::Mul<Vec3> for Quat {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        use super::matrices::*;
        self.to_mat33() * rhs
    }
}