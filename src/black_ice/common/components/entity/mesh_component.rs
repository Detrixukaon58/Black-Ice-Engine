#![allow(unused)]
#![allow(non_snake_case)]

use std::{any::Any, collections::HashMap};
use std::sync::Arc;
use parking_lot::*;

use crate::black_ice::common::engine::asset_types::materials::Material;
use crate::black_ice::common::New;
use crate::black_ice::common::{angles::*, components::{component_system::*, entity::entity_system::{entity_event::*, *}}, engine::{asset_types::{materials, shader_asset::Shader}, gamesys::*}, matrices::*, mesh::*, transform::Transform, vertex::*};
use crate::black_ice::common::engine::pipeline::*;
use colored::*;
// This is a type of pointer that is assigned by the game engine. This means that it must be of trait Reflection

pub struct MeshComponent {
    mesh: Arc<Mutex<Mesh>>,
    materials: HashMap<u32, Arc<Mutex<Material>>>,
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

        // load the materials
        let material_list = definition.get("materials").expect("Failed to get materials list").as_vector();
        let mut materials: HashMap<u32, Arc<Mutex<Material>>> = HashMap::new();
        
        Some(ComponentRef_new(MeshComponent {
            mesh: mesh.clone(), 
            layer: layer,
            materials: materials,
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
                Value::Component("materials".to_string(), std::sync::Arc::new(Value::Array(Vec::<Value>::new()))),
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
            // load the shader
            for p_material in &self.materials {
                let material = p_material.1.lock();
                let shader = material.shader.clone();
                RenderPipelineSystem::register_shader(self.layer, shader);
            }
            //println!("{}", "Added Mesh".blue());
        }
    }
    pub fn update_mesh(&mut self) {        
        // update mesh for the render pipelines
        unsafe{
            // now we gotta send the shader data to render
            let p_mesh = self.mesh.clone();
            let mut mesh = p_mesh.lock();
            mesh.transform = self.p_entity.get_world_tm() * self.transform.get_world_tm();
            for p_surface in &mesh.surfaces {
                let mut data = vec![Data::Surface(p_surface.clone()), Data::MeshMatrix("_m".to_string(), mesh.transform.clone()), Data::Vector("lol".to_string(), Vec3::new(1.0, 1.0, 1.0)), Data::Matrix("_norm".to_string(), self.transform.get_world_tm())];
                let surface = p_surface.lock();
                let id = surface.id.clone();
                drop(surface);
                let (di,p_material) = (&self.materials).into_iter().find(|x| {*x.0 == id}).unwrap();
                let material = p_material.lock();
                for (name, (p_value, value_type)) in &material.shader_descriptor {
                    match value_type {
                        crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataHint::Uniform => {
                            let value = p_value.lock();
                            match value.clone(){
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::Integer(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::Boolean(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::UnsignedInteger(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::Float(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::Double(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::Vec3(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::Vec4(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::Vec2(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::IVec3(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::IVec4(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::IVec2(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::UVec3(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::UVec4(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::UVec2(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::DVec3(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::DVec4(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::DVec2(i) => todo!(),
                                crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataType::Sampler2D(vec, _, _) => todo!(),
                            }
                        },
                        crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataHint::In => todo!(),
                        crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataHint::Out => todo!(),
                        crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataHint::InOut => todo!(),
                        crate::black_ice::common::engine::asset_types::shader_asset::ShaderDataHint::Buffer(_) => todo!(),
                    }
                }
                RenderPipelineSystem::register_shader(self.layer, material.shader.clone());
                RenderPipelineSystem::render_shader(self.layer, material.shader.clone(), data);
            }
        }
            
        
    }

    pub fn triangle(&mut self) {
        let mut m = self.mesh.lock();
        let id = m.triangles();
        self.materials.insert(id, Arc::new(Mutex::new(Material::new())));
    }

    pub fn square(&mut self) {
        let mut m = self.mesh.lock();
        let id = m.square();
        self.materials.insert(id, Arc::new(Mutex::new(Material::new())));
    }

    pub fn rotate(&mut self, rotation: Quat)
    {
        self.transform.rotate(rotation);
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.transform.translate(translation);
    }
}
