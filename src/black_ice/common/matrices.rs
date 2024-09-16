#![allow(unused)]


use crate::black_ice::common::{ vertex::*, New, angles::*};
use std::{*, fmt::Display};



/// Made for rotations only!!
#[derive(Copy, Clone)]
pub struct Matrix33 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3
}

impl Matrix33 {
    pub fn to_mat34(&self) -> Matrix34 {
        Matrix34 { 
            x: Vec4::new(self.x.x, self.x.y, self.x.z, 0.0), 
            y: Vec4::new(self.y.x, self.y.y, self.y.z, 0.0), 
            z: Vec4::new(self.z.x, self.z.y, self.z.z, 0.0) 
        }
    }
}

#[derive(Copy, Clone)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

impl Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec4({x}, {y}, {z}, {w})", x=self.x, y=self.y, z=self.z, w=self.w)
    }
}

impl Vec4 {
    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()
    }

    pub fn normalized(&mut self) {
        let m = self.magnitude();
        *self = Vec4::new(self.x / m, self.y / m, self.z / m, self.w / m);
    }

    pub fn new_from_vec3(vec: Vec3, w: f32) -> Self {
        Self { x: vec.x, y: vec.y, z: vec.z, w: w }
    }

}

/// This is our translations, rotations and scalings
#[derive(Copy, Clone)]
pub struct Matrix34 {

    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4

}

impl Display for Matrix34 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mat34(({xx},{xy},{xz},{xw})\n,({yx},{yy},{yz},{yw})\n,({zx},{zy},{zz},{zw}))",
            xx = self.x.x, xy = self.x.y, xz = self.x.z, xw = self.x.w,
            yx = self.y.x, yy = self.y.y, yz = self.y.z, yw = self.y.w,
            zx = self.z.x, zy = self.z.y, zz = self.z.z, zw = self.z.w
        )
    }
}

impl New<Matrix34> for Matrix34 {
    fn new() -> Matrix34 {
        return Matrix34 { x: Vec4::zero(), y: Vec4::zero(), z: Vec4::zero() };
    }
}

pub trait BasicMatrixOps<T> {

    fn mult(&self, rhs: T) -> T;
    fn add(&self, rhs: T) -> T;
    fn pow(&self, a: i16) -> T;
}

pub trait BasicMatrix {
    fn transform(&self, rhs: Vec3) -> Vec3;
    fn get_x(&self) -> Vec4;
    fn get_y(&self) -> Vec4;
    fn get_z(&self) -> Vec4;
}

impl ops::Mul for Matrix34 {
    type Output = Matrix34;
    fn mul(self, rhs: Self) -> Matrix34 {
        let mut result: Matrix34 = self.clone();
        result.mult(rhs);
        return result;
    }
}

pub trait Vec4Zero {
    fn zero() -> Vec4;
}

impl Vec4Zero for Vec4{
    fn zero() -> Vec4 {
        return Vec4::new(0,0,0,0);
    }
}

pub trait Vec4Constructor<T> {
    fn new(x:T, y:T, z:T, w:T) -> Vec4;
}

impl Vec4Constructor<f32> for Vec4 {
    fn new(x:f32, y:f32, z:f32, w:f32) -> Vec4 {
        return Vec4 { x: x, y: y, z: z, w: w }
    }
}

impl Vec4Constructor<Option<f32>> for Vec4 {
    fn new(x:Option<f32>, y:Option<f32>, z:Option<f32>, w:Option<f32>) -> Vec4 {
        return Vec4::new(x.unwrap(), y.unwrap(), z.unwrap(), w.unwrap());
    }
}

impl Vec4Constructor<i16> for Vec4 {
    fn new(x:i16, y:i16, z:i16, w:i16) -> Vec4 {
        return Vec4::new(f32::try_from(x).ok(), f32::try_from(y).ok(), f32::try_from(z).ok(), f32::try_from(w).ok());
    }
}

pub trait M34 {
    fn rotate(&mut self, rhs: Quat);
    fn translate(&mut self, rhs: Vec3);
    fn scale(&mut self, rhs: Vec3);
    fn mult(&mut self, rhs: Matrix34);
    fn mult_33(&mut self, rhs: Matrix33);
    fn get_scale(&self) -> Vec3;
    fn get_translation(&self) -> Vec3;
    fn get_rotation(&self) -> Quat;
    fn conjugate_transpose(&mut self);
}

pub trait QuatToMat33 {
    fn to_mat33(&self) -> Matrix33;
}

// This scales aswell as rotates!! Need to fix (likely dues to rots being applied to each axis in such a way that they affect the magnitude of each aspect of the final matrix)
impl QuatToMat33 for Quat {
    fn to_mat33(&self) -> Matrix33 {
        let s = ((self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()).powi(-2);
        let x = Vec3::new(
            1.0 - 2.0 * s * (self.y.powi(2) + self.z.powi(2)),
            2.0 * s * (self.x * self.y - self.z * self.w),
            2.0 * s * (self.x * self.z + self.y * self.w)
        );
        
        let y = Vec3::new(
            2.0 * s * (self.x * self.y + self.z * self.w),
            1.0 - 2.0 * s * (self.x.powi(2) + self.z.powi(2)),
            2.0 * s * (self.y * self.z - self.x * self.w)
        );

        let z = Vec3::new(
            2.0 * s * (self.x * self.z - self.y * self.w),
            2.0 * s * (self.y * self.z + self.x * self.w),
            1.0 - 2.0 * s * (self.x.powi(2) + self.y.powi(2))
        );
        // println!("{x}, \n {y}, \n {z}");
        return Matrix33 { x: x, y: y, z: z };
    }
}

pub trait Mat33ToQuat {
    fn to_quat(&self) -> Quat;
}

impl Mat33ToQuat for Matrix33 {
    fn to_quat(&self) -> Quat {
        let w = (1.0 + self.x.x + self.y.y + self.z.z).sqrt() / 2.0;
        return Quat::new(
            (self.z.y - self.y.z)/(4.0 * w),
            (self.x.z - self.z.x)/(4.0 * w),
            (self.y.x - self.x.y)/(4.0 * w),
            w
        );
    }
}

impl M34 for Matrix34 {
    fn mult(&mut self, rhs: Matrix34) {
        let x = Vec4::new(self.x.x, self.y.x, self.z.x, 0.0);
        let y = Vec4::new(self.x.y, self.y.y, self.z.y, 0.0);
        let z = Vec4::new(self.x.z, self.y.z, self.z.z, 0.0);
        let w = Vec4::new(self.x.w, self.y.w, self.z.w, 1.0);

        self.x = Vec4::new(
            x.x * rhs.x.x + x.y * rhs.x.y + x.z * rhs.x.z + x.w * rhs.x.w,
            y.x * rhs.x.x + y.y * rhs.x.y + y.z * rhs.x.z + y.w * rhs.x.w,
            z.x * rhs.x.x + z.y * rhs.x.y + z.z * rhs.x.z + z.w * rhs.x.w,
            w.x * rhs.x.x + w.y * rhs.x.y + w.z * rhs.x.z + w.w * rhs.x.w
        );
        self.y = Vec4::new(
            x.x * rhs.y.x + x.y * rhs.y.y + x.z * rhs.y.z + x.w * rhs.y.w,
            y.x * rhs.y.x + y.y * rhs.y.y + y.z * rhs.y.z + y.w * rhs.y.w,
            z.x * rhs.y.x + z.y * rhs.y.y + z.z * rhs.y.z + z.w * rhs.y.w,
            w.x * rhs.y.x + w.y * rhs.y.y + w.z * rhs.y.z + w.w * rhs.y.w
        );
        self.z = Vec4::new(
            x.x * rhs.z.x + x.y * rhs.z.y + x.z * rhs.z.z + x.w * rhs.z.w,
            y.x * rhs.z.x + y.y * rhs.z.y + y.z * rhs.z.z + y.w * rhs.z.w,
            z.x * rhs.z.x + z.y * rhs.z.y + z.z * rhs.z.z + z.w * rhs.z.w,
            w.x * rhs.z.x + w.y * rhs.z.y + w.z * rhs.z.z + w.w * rhs.z.w
        );

    }
    fn rotate(&mut self, rhs: Quat) {
        let mat = rhs.to_mat33();
        let mat34 = mat.to_mat34();
        let this = self.clone();
        let scale = this.get_scale();
        let mut temp = mat34 * this;
        let s = temp.get_scale();
        temp.x = Vec4::new(temp.x.x * scale.x / s.x, temp.x.y * scale.y / s.y, temp.x.z * scale.z / s.z, temp.x.w);
        temp.y = Vec4::new(temp.y.x * scale.x / s.x, temp.y.y * scale.y / s.y, temp.y.z * scale.z / s.z, temp.y.w);
        temp.z = Vec4::new(temp.z.x * scale.x / s.x, temp.z.y * scale.y / s.y, temp.z.z * scale.z / s.z, temp.z.w);
        *self = temp;
        
    }
    fn conjugate_transpose(&mut self) {
        let x = self.x.clone();
        let y = self.y.clone();
        let z = self.z.clone();
    }
    fn mult_33(&mut self, rhs: Matrix33) {
        let x = Vec3::new(self.x.x, self.y.x, self.z.x);
        let y = Vec3::new(self.x.y, self.y.y, self.z.y);
        let z = Vec3::new(self.x.z, self.y.z, self.z.z);
        let w = Vec3::new(self.x.w, self.y.w, self.z.w);

        self.x = Vec4::new(
            x.x * rhs.x.x + x.y * rhs.x.y + x.z * rhs.x.z,
            y.x * rhs.x.x + y.y * rhs.x.y + y.z * rhs.x.z,
            z.x * rhs.x.x + z.y * rhs.x.y + z.z * rhs.x.z,
            w.x * rhs.x.x + w.y * rhs.x.y + w.z * rhs.x.z
        );
        self.y = Vec4::new(
            x.x * rhs.y.x + x.y * rhs.y.y + x.z * rhs.y.z,
            y.x * rhs.y.x + y.y * rhs.y.y + y.z * rhs.y.z,
            z.x * rhs.y.x + z.y * rhs.y.y + z.z * rhs.y.z,
            w.x * rhs.y.x + w.y * rhs.y.y + w.z * rhs.y.z
        );
        self.z = Vec4::new(
            x.x * rhs.z.x + x.y * rhs.z.y + x.z * rhs.z.z,
            y.x * rhs.z.x + y.y * rhs.z.y + y.z * rhs.z.z,
            z.x * rhs.z.x + z.y * rhs.z.y + z.z * rhs.z.z,
            w.x * rhs.z.x + w.y * rhs.z.y + w.z * rhs.z.z
        );
    }
    fn translate(&mut self, rhs: Vec3) {
        // First we make the translation matrix that we need!
        let mut mat = Matrix34::new();
        mat.x = Vec4::new(1.0, 0.0, 0.0, rhs.x);
        mat.y = Vec4::new(0.0, 1.0, 0.0, rhs.y);
        mat.z = Vec4::new(0.0, 0.0, 1.0, rhs.z);
        self.mult(mat);
    }
    fn scale(&mut self, rhs: Vec3) {
        let mut mat = Matrix34::new();
        mat.x = Vec4::new(rhs.x, 0.0, 0.0, 0.0);
        mat.y = Vec4::new(0.0, rhs.y, 0.0, 0.0);
        mat.z = Vec4::new(0.0, 0.0, rhs.z, 0.0);
        self.mult(mat);
    }
    fn get_translation(&self) -> Vec3 {
        return Vec3::new(self.x.w, self.y.w, self.z.w);
    }
    fn get_scale(&self) -> Vec3 {
        return Vec3::new(
            Vec3::new(self.x.x, self.y.x, self.z.x).magnitude(),
            Vec3::new(self.x.y, self.y.y, self.z.y).magnitude(),
            Vec3::new(self.x.z, self.y.z, self.z.z).magnitude()
        );
    }
    fn get_rotation(&self) -> Quat {
        let scale = self.get_scale();
        let mat: Matrix33 = Matrix33 { x: Vec3::new(self.x.x / scale.x, self.x.y / scale.y, self.x.z / scale.z)
            , y:  Vec3::new(self.y.x / scale.x, self.y.y / scale.y, self.y.z / scale.z)
            , z:  Vec3::new(self.z.x / scale.x, self.z.y / scale.y, self.z.z / scale.z)
        };
        return mat.to_quat();
    }
}

pub trait M33Buffer {
    fn to_buffer(&self) -> [f32; 9];
}

impl M33Buffer for Matrix33 {
    fn to_buffer(&self) -> [f32; 9] {
        return [self.x.x, self.x.y, self.x.z,
        self.y.x, self.y.y, self.y.z,
        self.z.x, self.z.y, self.z.z];
    }
}

pub trait M34Buffer {
    fn to_buffer(&self) -> [f32; 12];
    fn to_buffer44(&self) -> [f32; 16];
}

impl M34Buffer for Matrix34 {
    fn to_buffer(&self) -> [f32; 12] {
        return [self.x.x, self.x.y, self.x.z, self.x.w,
        self.y.x, self.y.y, self.y.z, self.y.w,
        self.z.x, self.z.y, self.z.z, self.z.w];
    }
    fn to_buffer44(&self) -> [f32; 16] {
        return [self.x.x, self.x.y, self.x.z, self.x.w,
        self.y.x, self.y.y, self.y.z, self.y.w,
        self.z.x, self.z.y, self.z.z, self.z.w,
        0.0, 0.0, 0.0, 1.0];
        // [self.x.x, self.y.x, self.z.x, 0.0,
        // self.x.y, self.y.y, self.z.y, 0.0,
        // self.x.z, self.y.z, self.z.z, 0.0,
        // self.x.w, self.y.w, self.z.w, 0.1]
    }
}

#[derive(Copy, Clone)]
pub struct MatrixProjection {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl MatrixProjection {
    pub fn to_buffer(&self) -> [f32; 16]{
        [self.x.x, self.x.y, self.x.z, self.x.w,
        self.y.x, self.y.y, self.y.z, self.y.w,
        self.z.x, self.z.y, self.z.z, self.z.w,
        self.w.x, self.w.y, self.w.z, self.w.w]
        // [self.x.x, self.y.x, self.z.x, self.w.x,
        // self.x.y, self.y.y, self.z.y, self.w.y,
        // self.x.z, self.y.z, self.z.z, self.w.z,
        // self.x.w, self.y.w, self.z.w, self.w.w]
    }
    pub fn transpose(&mut self) {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        let w = self.w;
        self.x = Vec4::new(
            x.x, y.x, z.x, w.x
        );
        self.y = Vec4::new(
            x.y, y.y, z.y, w.y
        );
        self.z = Vec4::new(
            x.z, y.z, z.z, w.z
        );
        self.w = Vec4::new(
            x.w, y.w, z.w, w.w
        );
    }

    pub fn ortho_projection(&mut self, l:f32, r:f32, t:f32, b:f32, n:f32, f:f32) {
        
        // use cgmath::prelude::*;

        // let mat = cgmath::ortho(l, r, b, t, n, f);
        
        // self.x = Vec4::new(
        //     mat.x.x, mat.y.x, mat.z.x, mat.w.x
        // );
        // self.y = Vec4::new(
        //     mat.x.y, mat.y.y, mat.z.y, mat.w.y
        // );
        // self.z = Vec4::new(
        //     mat.x.z, mat.y.z, mat.z.z, mat.w.z
        // );
        // self.w = Vec4::new(
        //     mat.x.w, mat.y.w, mat.z.w, mat.w.w
        // );
        
        let mid_x = (r + l)/ (r - l);
        let mid_y = (t + b)/ (t - b);
        let mid_z = (f + n) / (f - n);

        let scale_x = 2.0 / (r - l);
        let scale_y = 2.0 / (t - b);
        let scale_z = 2.0 / (f - n);
        
        self.x = Vec4::new(
            scale_x, 0.0, 0.0, -mid_x
        );
        self.y = Vec4::new(
            0.0, scale_y, 0.0, -mid_y
        );
        self.z = Vec4::new(
            0.0, 0.0, -scale_z, -mid_z
        );
        self.w = Vec4::new(
            0.0, 0.0, 0.0, 1.0
        );
    }

    pub fn perpective_projection(&mut self, ratio: f32, view_angle: f32, far: f32, near: f32) {
        fn frustrum(l:f32, r:f32, t:f32, b:f32, n:f32, f:f32) -> MatrixProjection {
            let mid_x = n * (r + l) / (r - l);
            let mid_y = n * (t + b) / (t - b);
            let mid_z = (f + n) / (f - n);
            let mut mat = MatrixProjection::identity();
            let scale_x = 2.0 * n / (r - l);
            let scale_y = 2.0 * n / (t - b);
            let scale_z = 2.0 * f * n / (n - f);

            mat.x = Vec4::new(
                -scale_x, 0.0, 0.0, -mid_x
            );
            mat.y = Vec4::new(
                0.0, scale_y, 0.0, -mid_y
            );
            mat.z = Vec4::new(
                0.0, 0.0, scale_z, -mid_z
            );
            mat.w = Vec4::new(
                0.0, 0.0, -1.0, 0.0
            );
            mat
        }
        let t = near * (view_angle.to_radians()/2.0).tan();
        let b = -t.clone();
        let r = t.clone() * ratio;
        let l = -r.clone();
        let mat = frustrum(l, r, t, b, near, far);
        self.x = mat.x;
        self.y = mat.y;
        self.z = mat.z;
        self.w = mat.w;

        // let mat = cgmath::perspective(cgmath::Deg(view_angle), ratio, near, far);
        // // println!("{:?}", mat2);
        // self.x = Vec4::new(
        //     mat.x.x, mat.y.x, mat.z.x, mat.w.x
        // );
        // self.y = Vec4::new(
        //     mat.x.y, mat.y.y, mat.z.y, mat.w.y
        // );
        // self.z = Vec4::new(
        //     mat.x.z, mat.y.z, mat.z.z, mat.w.z
        // );
        // self.w = Vec4::new(
        //     mat.x.w, mat.y.w, mat.z.w, mat.w.w
        // );

        // *self = *self * Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0).cross(Vec3::new(0.0, 0.0, 1.0)), Vec3::new(0.0, 1.0, 0.0).angle_to(Vec3::new(0.0, 0.0, 1.0))).to_mat33().to_mat34();
        
    }

    pub fn new() -> Self {
        Self { x: Vec4::zero(), y: Vec4::zero(), z: Vec4::zero(), w: Vec4::zero() }
    }

    pub fn identity() -> Self{
        Self { x: Vec4::new(1.0, 0.0, 0.0, 0.0), y: Vec4::new(0.0, 1.0, 0.0, 0.0), z: Vec4::new(0.0, 0.0, 1.0, 0.0), w: Vec4::new(0.0, 0.0, 0.0, 1.0) }
    }

    
}

impl ops::Mul<Matrix34> for MatrixProjection {

    type Output = Self;
    fn mul(self, rhs: Matrix34) -> Self::Output {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        let w = self.w;

        Self {
            x: Vec4::new(
                x.x * rhs.x.x + x.y * rhs.y.x + x.z * rhs.z.x + x.w * 0.0,
                y.x * rhs.x.x + y.y * rhs.y.x + y.z * rhs.z.x + y.w * 0.0,
                z.x * rhs.x.x + z.y * rhs.y.x + z.z * rhs.z.x + z.w * 0.0,
                w.x * rhs.x.x + w.y * rhs.y.x + w.z * rhs.z.x + w.w * 0.0
            ),
            y: Vec4::new(
                x.x * rhs.x.y + x.y * rhs.y.y + x.z * rhs.z.y + x.w * 0.0,
                y.x * rhs.x.y + y.y * rhs.y.y + y.z * rhs.z.y + y.w * 0.0,
                z.x * rhs.x.y + z.y * rhs.y.y + z.z * rhs.z.y + z.w * 0.0,
                w.x * rhs.x.y + w.y * rhs.y.y + w.z * rhs.z.y + w.w * 0.0
            ),
            z: Vec4::new(
                x.x * rhs.x.z + x.y * rhs.y.z + x.z * rhs.z.z + x.w * 0.0,
                y.x * rhs.x.z + y.y * rhs.y.z + y.z * rhs.z.z + y.w * 0.0,
                z.x * rhs.x.z + z.y * rhs.y.z + z.z * rhs.z.z + z.w * 0.0,
                w.x * rhs.x.z + w.y * rhs.y.z + w.z * rhs.z.z + w.w * 0.0
            ),
            w: Vec4::new(
                x.x * rhs.x.w + x.y * rhs.y.w + x.z * rhs.z.w + x.w * 1.0,
                y.x * rhs.x.w + y.y * rhs.y.w + y.z * rhs.z.w + y.w * 1.0,
                z.x * rhs.x.w + w.y * rhs.y.w + w.z * rhs.z.w + z.w * 1.0,
                w.x * rhs.x.w + w.y * rhs.y.w + w.z * rhs.z.w + w.w * 1.0
            )
        }
        
    }
}

impl ops::Mul<MatrixProjection> for Matrix34 {

    type Output = Self;
    fn mul(self, rhs: MatrixProjection) -> Self::Output {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        let w = Vec4::new(0.0, 0.0, 0.0, 1.0);

        Self {
            x: Vec4::new(
                x.x * rhs.x.x + x.y * rhs.y.x + x.z * rhs.z.x + x.w * rhs.w.x,
                y.x * rhs.x.x + y.y * rhs.y.x + y.z * rhs.z.x + y.w * rhs.w.x,
                z.x * rhs.x.x + z.y * rhs.y.x + z.z * rhs.z.x + z.w * rhs.w.x,
                w.x * rhs.x.x + w.y * rhs.y.x + w.z * rhs.z.x + w.w * rhs.w.x
            ),
            y: Vec4::new(
                x.x * rhs.x.y + x.y * rhs.y.y + x.z * rhs.z.y + x.w * rhs.w.y,
                y.x * rhs.x.y + y.y * rhs.y.y + y.z * rhs.z.y + y.w * rhs.w.y,
                z.x * rhs.x.y + z.y * rhs.y.y + z.z * rhs.z.y + z.w * rhs.w.y,
                w.x * rhs.x.y + w.y * rhs.y.y + w.z * rhs.z.y + w.w * rhs.w.y
            ),
            z: Vec4::new(
                x.x * rhs.x.z + x.y * rhs.y.z + x.z * rhs.z.z + x.w * rhs.w.z,
                y.x * rhs.x.z + y.y * rhs.y.z + y.z * rhs.z.z + y.w * rhs.w.z,
                z.x * rhs.x.z + z.y * rhs.y.z + z.z * rhs.z.z + z.w * rhs.w.z,
                w.x * rhs.x.z + w.y * rhs.y.z + w.z * rhs.z.z + w.w * rhs.w.z
            ),

        }
        
    }
}

impl std::ops::Mul<Vec4> for Matrix34 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4::new(
            self.x.x * rhs.x + self.x.y * rhs.y + self.x.z * rhs.z + self.x.w * rhs.w,
            self.y.x * rhs.x + self.y.y * rhs.y + self.y.z * rhs.z + self.y.w * rhs.w,
            self.z.x * rhs.x + self.z.y * rhs.y + self.z.z * rhs.z + self.z.w * rhs.w,
            0.0 * rhs.x + 0.0 * rhs.y + 0.0 * rhs.z + 1.0 * rhs.w
        )
    }
}

impl std::ops::Mul<Vec4> for MatrixProjection {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4::new(
            self.x.x * rhs.x + self.x.y * rhs.y + self.x.z * rhs.z + self.x.w * rhs.w,
            self.y.x * rhs.x + self.y.y * rhs.y + self.y.z * rhs.z + self.y.w * rhs.w,
            self.z.x * rhs.x + self.z.y * rhs.y + self.z.z * rhs.z + self.z.w * rhs.w,
            self.w.x * rhs.x + self.w.y * rhs.y + self.w.z * rhs.z + self.w.w * rhs.w
        )
    }
}

impl Matrix34 {

    pub fn identity() -> Self {
        Self { x: Vec4::new(1, 0, 0, 0), y: Vec4::new(0, 1, 0, 0), z: Vec4::new(0, 0, 1, 0,) }
    }
}

impl std::ops::Mul<Vec3> for Matrix33 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.x.x * rhs.x + self.x.y * rhs.y + self.x.z * rhs.z,
            self.y.x * rhs.x + self.y.y * rhs.y + self.y.z * rhs.z,
            self.z.x * rhs.x + self.z.y * rhs.y + self.z.z * rhs.z
        )
    }
}

impl std::ops::Mul<Vec3> for Matrix34 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.x.x * rhs.x + self.x.y * rhs.y + self.x.z * rhs.z + self.x.w,
            self.y.x * rhs.x + self.y.y * rhs.y + self.y.z * rhs.z + self.y.w,
            self.z.x * rhs.x + self.z.y * rhs.y + self.z.z * rhs.z + self.z.w
        )
    }
}

impl ops::Mul<f32> for Vec4{
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Vec4 {
        let mut this = self;
        this.x *= rhs;
        this.y *= rhs;
        this.z *= rhs;
        this.w *= rhs;
        this
    }
}

impl ops::Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Self) -> Self::Output {
        let mut this = self;
        this.x += rhs.x;
        this.y += rhs.y;
        this.z += rhs.x;
        this.w += rhs.w;
        this
    }
}

impl ops::Neg for Vec4 {
    type Output = Vec4;
    fn neg(self) -> Self::Output {
        let mut this = self;
        this.x = -this.x;
        this.y = -this.y;
        this.z = -this.z;
        this.w = -this.w;
        this
    }
}

impl ops::Sub for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut this = self;
        this.x -= rhs.x;
        this.y -= rhs.y;
        this.z -= rhs.z;
        this.w -= rhs.w;
        this
    }
}

impl ops::AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}