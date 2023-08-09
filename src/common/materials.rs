use std::{any::{TypeId, Any}, collections::HashMap, mem::{MaybeUninit, size_of_val}, path::{Path, PathBuf}, fs, sync::{Arc, Mutex}, str::FromStr, ffi::CString};

#[cfg(feature = "opengl")] use ogl33::c_void;
#[cfg(feature = "vulkan")]use ash::*;
use serde::ser::*;

use crate::{common::{filesystem::files::*, engine::gamesys::*, *}};

use super::{resources::Resource, matrices::{Matrix33, Matrix34}};

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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Shader{
    pub fragmentFile: AssetPath,
    pub vertexFile: AssetPath,
}

impl Shader {
    fn new(pathToShaders: String) -> Shader {
        let mut sub = 0;
        for i in ((&pathToShaders).len())..0 {
            if(pathToShaders.chars().nth(i).unwrap() == '\\'){
                sub = i;
                break;
            }
        }
        let fileName = pathToShaders.split_at(sub).1;
        let fileNameInit = fileName.split(".").collect::<Vec<&str>>()[0];
        let fragPath = format!("{}\\{}.frag", pathToShaders, fileNameInit);
        let mut fragFile = AssetPath::new(fragPath);
        let vertPath = format!("{}\\{}.frag", pathToShaders, fileNameInit);
        let mut vertFile = AssetPath::new(vertPath);
        Shader { fragmentFile:  fragFile, vertexFile: vertFile}
    }


    fn read_uniforms(&mut self) -> HashMap<String, Box<&'static dyn Base>> {
        let mut hash: HashMap<String, Box<&'static dyn Base>> = HashMap::new();
        let mut frag_file = self.fragmentFile.open_as_file();
        let mut line = frag_file.read();


        return hash;
    }
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        let mut fragFile = self.fragmentFile.clone();

        let mut vertFile = self.vertexFile.clone();
        let mut new_shader = Shader { fragmentFile:  fragFile, vertexFile: vertFile};
        return new_shader;
    }
}

pub trait ShaderDescriptor{
    fn get_num_values(&self) -> isize;
    fn get_value_type(&self, offset: isize) -> TypeId;
    fn get_value(&self, offset: isize) -> Ptr<Box<&'static dyn Base>>;
    fn get_value_name(&self, offset: isize) -> String;
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum ShaderType {
    Integer(i32),
    Boolean(bool),
    Unsigned_Integer(u32),
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

}

#[derive(serde::Deserialize)]
pub struct Material {
    
    pub shader: Shader,
    pub shaderDescriptor: HashMap<String, Box<ShaderType>>,

}

impl serde::Serialize for Material {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(self.shaderDescriptor.len())).unwrap();
        for (k,v) in &self.shaderDescriptor {
            map.serialize_entry(&k.to_string(), &*v).unwrap();
        }
        map.end()
    }
}

impl Clone for Material {
    fn clone(&self) -> Self {
        let mut mat = Material::new();
        mat.shader = self.shader.clone();
        mat.shaderDescriptor = HashMap::new();
        for param in self.shaderDescriptor.keys() {
            let value = self.shaderDescriptor.get(param).unwrap().clone();
            
            mat.shaderDescriptor.insert(param.to_string(), Box::new((*value).clone()));
        }
        return mat;
    }
}

impl Base for Material{}

impl New<Material> for Material {
    fn new() -> Material {
        return Material {shader: Shader::new("ASSET:shaders/slim-shader.shad".to_string()),shaderDescriptor: HashMap::new() };
    }
}

impl Reflection for Material{
    fn registerReflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));
        
        

        return Ptr {b: register};
    } 
}

#[cfg(feature = "vulkan")]
pub trait BakeVulkan {
    fn bake(&mut self, device: Option<vk::PhysicalDevice>);
}

#[cfg(feature = "vulkan")]
impl BakeVulkan for Material {
    fn bake(&mut self, device: Option<vk::PhysicalDevice>){
        unsafe{
            let device = device.unwrap();

        }
    }
}

fn include_shaders() -> glsl_include::Context<'static> {
    let path: String = APP_DIR.clone().to_owned() + "\\assets\\shaders\\";
    let directory = fs::read_dir(path).unwrap();
    let mut context: glsl_include::Context = glsl_include::Context::new();
    for mut path in directory {
        let mut path_unwraped = path.unwrap();
        let path_path = path_unwraped.path();
        if(path_path.is_file() && compare(path_unwraped.file_name().to_str().unwrap())){
            let mut path_file = path_unwraped.file_name();
            let mut path_file_str = path_file.as_os_str().to_str().unwrap();
            
            let mut path_data = path_path.display();
            let mut path_data_string = path_data.to_string();
            let mut path_data_str = path_data_string.as_str();
            context.include(path_file_str, path_data_str);
        }
    
    }
    return context;
}