use crate::common::{vertex::*, angles::*, matrices::*};

#[derive(Clone)]
pub struct Transform {

    pub transform_id: i128,
    pub local_position: Vec3,
    pub local_rotation: Quat,
    pub local_euler_rotation: Ang3,
    pub local_scale: Vec3,

    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    pub world_matrix: Matrix34,

    pub parent: i128,

    pub children: Vec<i128>
    
    

}