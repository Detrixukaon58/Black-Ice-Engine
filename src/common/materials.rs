#![allow(unused)]
use std::{any::*, collections::HashMap};

use crate::common::{filesystem::files::*, engine::gamesys::*, *};



// pub struct ParamDescriptor {
//     pub name: String,
//     pub value: Box<MaybeUninit<&'static dyn Any>>,
//     pub data_type: TypeId,
//     pub size: isize,
// }

// impl ParamDescriptor {
//     fn new<T: Any>(name: String, value: &'static T) -> ParamDescriptor{
//         return ParamDescriptor {name: name, value: Box::new(MaybeUninit::new(value.as_any().clone())), data_type: value.type_id(), size: size_of_val(&value) as isize};
//     }
    
//     fn new_uninit(name: String) -> ParamDescriptor {
//         return ParamDescriptor { name: name, value: Box::new(MaybeUninit::uninit()), data_type: TypeId::of::<&dyn Any>(), size: 0 };
//     }

//     fn set_value<T: Any>(&mut self, value: &'static T){
//         self.value = Box::new(MaybeUninit::new(value.as_any().clone()));
//         self.data_type = value.type_id();
//         self.size = size_of_val(&value) as isize;
//     }
// }

pub fn compare(input: &str) -> bool {
    return input.ends_with(".glsl") || input.ends_with(".vert") || input.ends_with(".frag") || input.ends_with(".comp");
}

pub struct Shader{
    pub shader_name: String,
    pub fragment_file: ShaderFile,
    pub vertex_file: ShaderFile,
    pub is_compiled: bool,
}

impl Shader {
    fn new(path_to_shaders: String) -> Shader {

        let mut sub = path_to_shaders.chars().count() - path_to_shaders.chars().rev().position(|v| (v == '\\' || v == '/')).unwrap();
        
        let file_name_init = &path_to_shaders[sub..(&path_to_shaders.chars().count() - 5)];
        let frag_path = format!("{}/{}.frag", path_to_shaders, file_name_init);
        let frag_file = AssetPath::new(frag_path).open_as_file().as_shader_file(shaderc::ShaderKind::Fragment);
        let vert_path = format!("{}/{}.vert", path_to_shaders, file_name_init);
        let vert_file = AssetPath::new(vert_path).open_as_file().as_shader_file(shaderc::ShaderKind::Vertex);
        Shader {shader_name: String::from(file_name_init), fragment_file:  frag_file, vertex_file: vert_file, is_compiled: false}
    }

    fn compile_shader(&mut self) -> bool {
        self.is_compiled = self.fragment_file.compile() && self.vertex_file.compile();

        self.is_compiled
    }

    fn read_uniforms(&mut self) -> HashMap<String, Box<&'static dyn Base>> {
        let hash: HashMap<String, Box<&'static dyn Base>> = HashMap::new();
        let frag_file = &self.fragment_file;
        


        return hash;
    }
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        let frag_file = self.fragment_file.clone();

        let vert_file = self.vertex_file.clone();
        let new_shader = Shader {shader_name: self.shader_name.clone(), fragment_file:  frag_file, vertex_file: vert_file, is_compiled: self.is_compiled.clone()};
        return new_shader;
    }
}

pub trait ShaderDescriptor{
    fn get_num_values(&self) -> isize;
    fn get_value_type(&self, offset: isize) -> TypeId;
    fn get_value(&self, offset: isize) -> Ptr<Box<&'static dyn Base>>;
    fn get_value_name(&self, offset: isize) -> String;
}

#[derive(Clone)]
pub enum ShaderType {
    Integer(i32),
    Boolean(bool),
    UnsignedInteger(u32),
    Float(f32),
    Double(f64),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Vec2([f32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
    IVec2([i32; 2]),
    UVec3([u32; 3]),
    UVec4([u32; 4]),
    UVec2([u32; 2]),
    DVec3([f64; 3]),
    DVec4([f64; 4]),
    DVec2([f64; 2]),
    Sampler2D(Vec<u32>, u32, u32),

}

#[derive(Clone)]
pub enum ShaderDataHint {
    Uniform,
    In,
    Out,
    InOut,
    Buffer,

}


pub struct Material {
    
    pub shader: Shader,
    pub shader_descriptor: HashMap<String, (Box<ShaderType>, ShaderDataHint)>,

}


impl Clone for Material {
    fn clone(&self) -> Self {
        let mut mat = Material::new();
        mat.shader = self.shader.clone();
        mat.shader_descriptor = HashMap::new();
        for param in self.shader_descriptor.keys() {
            let value = self.shader_descriptor.get(param).unwrap().clone();
            
            mat.shader_descriptor.insert(param.to_string(), (Box::new((*value.0).clone()), value.1.clone()));
        }
        return mat;
    }
}

impl Base for Material{}

impl New<Material> for Material {
    fn new() -> Material {
        return Material {shader: Shader::new("ASSET:/shaders/slim-shadey.shad".to_string()),shader_descriptor: HashMap::new() };
    }
}

impl Reflection for Material{
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let _register = Box::new(Register::new(Box::new(self)));
        
        

        return Ptr {b: _register};
    } 
}