use crate::common::vertex::*;
use serde::*;
use super::engine::gamesys::Base;

#[derive(Copy, Clone)]
pub struct Ang3 {
    y : f32,
    p : f32,
    r : f32
}

#[derive(Copy, Clone, Serialize, Deserialize)]
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