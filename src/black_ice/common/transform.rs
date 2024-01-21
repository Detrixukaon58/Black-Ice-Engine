use crate::black_ice::common::{vertex::*, angles::*, matrices::*};

use super::{components::entity::entity_system::*, engine::gamesys::Game};
use std::sync::*;

#[derive(Clone)]
pub struct Transform {

    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    world_matrix: Matrix34,

    parent_transform: Option<Arc<Transform>>,
    pp_entity: Option<EntityID>,

}

impl Default for Transform {
    fn default() -> Self {
        Self{
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::euler(Ang3::new(0.0, 0.0, 0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
            world_matrix: Matrix34::identity(),
            parent_transform: None,
            pp_entity: None,
        }
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let mut mat = Matrix34::identity();
        mat.translate(position);
        mat.rotate(rotation);
        // mat.scale(scale);
        Self { position: position, rotation: rotation, scale: scale, world_matrix: mat , parent_transform: None, pp_entity: None}
    }

    fn update(&mut self) {
        
        self.world_matrix = Matrix34::identity();
        self.world_matrix.translate(self.position);
        self.world_matrix.rotate(self.rotation);
        self.world_matrix.scale(self.scale);
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation *= rotation;
        self.update();
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
        self.update();
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.update();
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.update();
    }
    
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.update();
    }

    pub fn get_entity(&self) -> Option<EntityPtr> {
        if let Some(p_entity) = self.pp_entity.as_ref(){
            unsafe {
                let p_entsy = Game::get_entity_sys();
                let mut ent_sys = p_entsy.lock();
                return ent_sys.get_entity(*p_entity);
            }
        }
        None
    }

    pub fn get_forward(&self) -> Vec3 {
        let v = self.world_matrix * Vec4::new(1.0, 0.0, 0.0, 1.0);
        Vec3::new(v.x, v.y, v.z)
    }

    pub fn get_world_tm(&self) -> Matrix34 {
        let mut result = self.world_matrix.clone();
        let mut p_parent = self.parent_transform.clone();
        while let Some(ref parent) = p_parent {
            result = result * parent.get_tm();
            p_parent = parent.parent_transform.clone();
        }
        result
    }

    pub fn get_tm(&self) -> Matrix34 {
        self.world_matrix.clone()
    }

    pub fn set_tm(&mut self, tm: Matrix34) {
        let rotation = tm.get_rotation();
        let scale = tm.get_scale();
        let position = tm.get_translation();
        self.set_rotation(rotation);
        self.set_position(position);
        self.set_scale(scale);
    } 

    pub fn get_global_position(&self) -> Vec3 {
        if let Some(t) = self.parent_transform.as_ref() {
            let v = t.get_world_tm() * Vec4::new_from_vec3(self.position, 1.0);
            Vec3::new(v.x, v.y, v.z)
        }
        else
        {
            self.position.clone()
        }
    }
}

pub trait TransformSetEntity {
    fn set_entity(&mut self, p_entity: EntityID);
}

impl TransformSetEntity for Transform {
    fn set_entity(&mut self, p_entity: EntityID) {
        self.pp_entity = Some(p_entity);
    }
}