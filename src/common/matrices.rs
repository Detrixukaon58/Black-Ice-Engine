use crate::common::{ vertex::*, New, angles::*};
use std::*;



/// Made for rotations only!!
#[derive(Copy, Clone)]
pub struct Matrix33 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3
}

#[derive(Copy, Clone)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

/// This is our translations, rotations and scalings
#[derive(Copy, Clone)]
pub struct Matrix34 {

    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4

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
    fn getX(&self) -> Vec4;
    fn getY(&self) -> Vec4;
    fn getZ(&self) -> Vec4;
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

impl Vec4Constructor<Option<i16>> for Vec4 {
    fn new(x:Option<i16>, y:Option<i16>, z:Option<i16>, w:Option<i16>) -> Vec4 {
        return Vec4::new(x.unwrap(), y.unwrap(), z.unwrap(), w.unwrap());
    }
}

impl Vec4Constructor<i16> for Vec4 {
    fn new(x:i16, y:i16, z:i16, w:i16) -> Vec4 {
        return Vec4::new(i16::try_from(x).ok(), i16::try_from(y).ok(), i16::try_from(z).ok(), i16::try_from(w).ok());
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
}

pub trait QuatToMat33 {
    fn to_mat33(&self) -> Matrix33;
}

impl QuatToMat33 for Quat {
    fn to_mat33(&self) -> Matrix33 {
        let x = Vec3::new(2.0 * (self.x.powi(2) + self.y.powi(2)) - 1.0, 2.0 * (self.y * self.z - self.x * self.w), 2.0 * (self.y * self.w + self.x * self.z));
        let y = Vec3::new(2.0 * (self.y * self.z + self.x * self.w), 2.0 * (self.x.powi(2) + self.z.powi(2)) - 1.0, 2.0 * (self.z * self.w - self.x * self.y));
        let z = Vec3::new(2.0 * (self.y * self.w - self.x * self.z), 2.0 * (self.z * self.w + self.x * self.y), 2.0 * (self.x.powi(2) + self.w.powi(2)) -1.0);
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

        self.mult_33(mat);

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
        mat.x = Vec4::new(rhs.x, 0.0, 0.0, 1.0);
        mat.y = Vec4::new(0.0, rhs.y, 0.0, 1.0);
        mat.z = Vec4::new(0.0, 0.0, rhs.z, 1.0);
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
}

impl M34Buffer for Matrix34 {
    fn to_buffer(&self) -> [f32; 12] {
        return [self.x.x, self.x.y, self.x.z, self.x.w,
        self.y.x, self.y.y, self.y.z, self.y.w,
        self.z.x, self.z.y, self.z.z, self.z.w];
    }
}