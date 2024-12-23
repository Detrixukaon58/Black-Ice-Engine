#![allow(unused)]
#![allow(non_snake_case)]

use std::{any::Any, collections::HashMap};
use std::sync::Arc;
use parking_lot::*;

use crate::black_ice::common::{angles::*, components::{component_system::*, entity::entity_system::{entity_event::*, *}}, engine::{asset_types::{materials, shader_asset::Shader}, gamesys::*}, matrices::*, mesh::*, transform::Transform, vertex::*};
use crate::black_ice::common::engine::pipeline::*;
use colored::*;
// This is a type of pointer that is assigned by the game engine. This means that it must be of trait Reflection

pub struct MeshComponent {
    mesh: Arc<Mutex<Mesh>>,
    surface_ids: HashMap<u32, i32>,
    layer: u32,
    transform: Transform,
    pub p_entity: EntityPtr,

}

impl Base for MeshComponent{}

impl BaseComponent for MeshComponent {
    fn get_entity(&self) -> EntityPtr {
        self.p_entity.clone()
    }

    fn get_event_mask(&self) -> EventFlag {
        EventFlag::INIT | EventFlag::UPDATE | EventFlag::RESPAWN
    }

    fn process_event(&mut self, event: &Event) {
        let frame_time = event.event_data.get("frame_time".to_string()).unwrap().as_f32().unwrap();
        match event.event_flag {
            EventFlag::INIT => {
                self.init_mesh();
                self.transform.set_rotation(Quat::euler(Ang3::new(0.0, 0.0, 0.0)));
            }
            EventFlag::UPDATE => {
                self.update_mesh();
            }
            EventFlag::RESPAWN => {

            }
            _ => {}
        }
    }
}

impl Reflection for MeshComponent{
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));
        //register.addProp(Property { name: "mesh", desc: "The Mesh", reference: Box::new(&self.mesh), refType: self.mesh.type_id() });
        
        //register.addPointer(Pointer{name: Box::new("mesh"), desc: Box::new("The Mesh Pointer"), reference: self.mesh.register_reflect(), ref_type: self.mesh.type_id()});

        return Ptr {b: register};
    }
}

impl Constructor<MeshComponent> for MeshComponent {
    unsafe fn construct(entity: EntityPtr, definition: &ConstructorDefinition) -> Option<ComponentRef<MeshComponent>> {
        let mut mesh: Arc<Mutex<Mesh>>;
        if definition.get("file").is_none() || definition["file"].get("mesh_file_path").is_none() || definition["file"]["mesh_file_path"].as_str().unwrap().is_empty() {
            mesh = Arc::new(Mutex::new(Mesh::new()));
        }
        else{
            let mesh_file = MeshFile::construct(definition.get("file").expect("Corrupted Mesh File Definition!!").clone());
            mesh = Arc::new(Mutex::new(mesh_file.as_mesh()));
        }
        let layer = definition.get("layer").expect("Failed to get layer!!").as_u32().unwrap();
        Some(ComponentRef_new(MeshComponent {
            mesh: mesh.clone(), 
            surface_ids: HashMap::new(),
            layer: layer,
            transform: Transform::new(definition["position"].as_vec3()?, definition["rotation"].as_quat()?, definition["scale"].as_vec3()?),
            p_entity: entity.clone(),
        }))
    }
    fn default_constuctor_definition() -> ConstructorDefinition {
        std::sync::Arc::new(
            Value::Array( vec![
                Value::Component("file".to_string(), std::sync::Arc::new(
                    Value::Component("mesh_file_path".to_string(), std::sync::Arc::new(
                        Value::String(String::new())
                    ))
                )),
                Value::Component("layer".to_string(), std::sync::Arc::new(Value::I32(0))),
                Value::Component("position".to_string(), std::sync::Arc::new(Value::Vec3(Vec3::new(0.0, 0.0, 0.0)))),
                Value::Component("rotation".to_string(), std::sync::Arc::new(Value::Quat(Quat::euler(Ang3::new(0.0, 0.0, 0.0))))),
                Value::Component("scale".to_string(), std::sync::Arc::new(Value::Vec3(Vec3::new(1.0, 1.0, 1.0))))
                ]
            )
        )
    }
}

impl MeshComponent {

    pub fn init_mesh(&mut self) {
        unsafe{
            //println!("{}", "Adding Mesh".red());
            // lets add all the mesh and shader data
            let p_mesh = self.mesh.clone();
            let mesh = p_mesh.lock();

            for (id, p_material) in &mesh.materials {
                let material = p_material.lock();
                let shader = material.shader.clone();
                
                RenderPipelineSystem::register_shader(self.layer, shader);
            }
            
            //println!("{}", "Added Mesh".blue());
        }
    }
    pub fn update_mesh(&mut self) {
        let mut mesh = self.mesh.lock();
        mesh.transform = self.p_entity.get_world_tm() * self.transform.get_world_tm();
        // update mesh for the render pipelines
        unsafe{

            for (id, p_material) in &mesh.materials {
                let material = p_material.lock();
                let shader = material.shader.clone();
                let surface = mesh.surfaces.iter().find(|p_x| { let x = p_x.lock(); x.id==*id}).unwrap().clone();
                let data = vec![Data::Surface(surface)];

                RenderPipelineSystem::register_shader(self.layer, shader.clone());
                RenderPipelineSystem::render_shader(self.layer, shader, data);
            }
            
        }
    }

    pub fn triangle(&mut self) {
        let mut m = self.mesh.lock();
        m.triangles();
        
    }

    pub fn square(&mut self) {
        let mut m = self.mesh.lock();
        m.square();
    }

    pub fn rotate(&mut self, rotation: Quat)
    {
        self.transform.rotate(rotation);
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.transform.translate(translation);
    }
}
