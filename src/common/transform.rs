use crate::common::{vertex::*, angles::*, matrices::*};

#[derive(Clone)]
pub struct Transform {

    pub transformID: i128,
    pub localPosition: Vec3,
    pub localRotation: Quat,
    pub localEulerRotation: Ang3,
    pub localScale: Vec3,

    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    pub worldMatrix: Matrix34,

    pub parent: i128,

    pub children: Vec<i128>
    
    

}