#![allow(unused)]
use std::{any::*, collections::HashMap};
use parking_lot::*;
use shaderc::ShaderKind;
use crate::black_ice::common::{filesystem::files::*, engine::gamesys::*, *};
use std::sync::Arc;

use super::engine::pipeline::RenderPipelineSystem;


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

#[derive(Clone)]
pub enum ShaderType {
    Compute,
    Fragment,
    Vertex,
}

pub struct ShaderData {
    pub data: Arc<Mutex<Vec<u8>>>,
    pub compiled_data: Option<Arc<Mutex<Vec<u8>>>>,
}

impl ShaderData {
    fn include_shaders() -> glsl_include::Context<'static> {
        let path: String = APP_DIR.to_owned() + "\\assets\\shaders\\";

        let mut context: glsl_include::Context = glsl_include::Context::new();
        return context;
    }

    pub fn compile(&mut self, shaderc_kind: ShaderKind, shader_lang:ShaderLang) {
        
    }
}

#[derive(Copy, Clone)]
pub enum ShaderLang {
    Glsl,
    Hlsl,
    Pgsl,
    GodotShader,
}

pub struct ShaderStage {
    pub stage_name: String,
    pub shader_data: ShaderData,
    pub shader_type: ShaderType,
    pub shader_lang: ShaderLang,
}

impl ShaderStage {
    fn new(shader_name: String, shader_type: ShaderType, shader_lang: ShaderLang, mut shader_data: ShaderData) -> ShaderStage {
        
        let shader_kind = match (shader_type) {
            ShaderType::Fragment => shaderc::ShaderKind::Fragment,
            ShaderType::Vertex => shaderc::ShaderKind::Vertex,
            ShaderType::Compute => shaderc::ShaderKind::Compute,
        };

        unsafe {
            shader_data.compile(shader_kind, shader_lang);
            ShaderStage {stage_name: shader_name, shader_data: shader_data, shader_type:shader_type, shader_lang:shader_lang}
        }
    }

    fn new_from_compiled(shader_name: String, shader_type: ShaderType, shader_lang: ShaderLang, mut shader_data: ShaderData) -> ShaderStage {
        let shader_kind = match (shader_type) {
            ShaderType::Fragment => shaderc::ShaderKind::Fragment,
            ShaderType::Vertex => shaderc::ShaderKind::Vertex,
            ShaderType::Compute => shaderc::ShaderKind::Compute,
        };

        unsafe {
            
            ShaderStage {stage_name: shader_name, shader_data: shader_data, shader_type:shader_type, shader_lang:shader_lang}
        }
    }

    fn read_uniforms(&mut self) -> HashMap<String, Box<&'static dyn Base>> {
        let hash: HashMap<String, Box<&'static dyn Base>> = HashMap::new();
        let file = &self.shader_data;
        
        return hash;
    }
}

pub type ShaderPtr = usize;

#[derive(Clone)]
pub struct Shader {
    pub shader_stages: Vec<ShaderPtr>,
    pub shader_name: String,
    pub file_path: String,
}
struct ShaderToken {
    pub shader_name: String,
    pub shader_type: ShaderType,
    pub shader_lang: ShaderLang,
    pub shader_inout_datas: Vec<(String, ShaderDataTypeClean, ShaderDataHint)>,
    pub shader_code: Vec<u8>,
    pub is_compiled: bool,

}

impl Shader {
    /// Opens .shad, .glsl or .comp shader files
    pub fn new(path: String) -> Self {
        // This opens either .shad, .glsl or .comp shader files!!
        // get the file extention
        let mut file = FileSys::new();
        file.open(path.as_str());
        let ext = file.get_file_ext();
        if(file.check_file_ext() != MFType::SHADER){
            panic!("File is not a shader type!!");
        }
        else{
            // parse data and organise
            let mut stages: Vec<ShaderStage> = vec![];
            unsafe {
                match ext.as_str() {
                    "shad" => {
                        // parse file and load shader stages
                        let mut datas = Self::parse_shad_file(file.read());
                        for token in datas {
                            let mut stage_ext = ".shad";
                            match token.shader_lang {
                                ShaderLang::Glsl => {
                                    match token.shader_type {
                                        ShaderType::Compute => stage_ext = ".comp",
                                        ShaderType::Fragment => stage_ext = ".frag",
                                        ShaderType::Vertex => stage_ext = ".vert",
                                    }
                                },
                                ShaderLang::Hlsl => stage_ext = ".hlsl",
                                ShaderLang::GodotShader => stage_ext = ".gdshad",
                                ShaderLang::Pgsl => stage_ext = ".pfx",
                            }
                            if !token.is_compiled {
                                let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(token.shader_code.clone())), compiled_data: None };
                                stages.push(ShaderStage::new(file.get_file_name() + stage_ext, token.shader_type, token.shader_lang.clone(), shader_data));
                            }
                            else {
                                let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(token.shader_code.clone())), compiled_data: Some(Arc::new(Mutex::new(token.shader_code.clone()))) };
                                stages.push(ShaderStage::new_from_compiled(file.get_file_name() + stage_ext, token.shader_type, token.shader_lang.clone(), shader_data));
                            }
                        }
                    },
                    "vert" => {
                        //parse as single stage
                        let mut vec = file.read();
                        let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.as_mut_vec().clone())), compiled_data: None };
                        stages.push(ShaderStage::new(file.get_file_name() + file.get_file_ext().as_str(), ShaderType::Vertex, ShaderLang::Glsl, shader_data));
                    },
                    "frag" => {
                        //parse as single stage
                        let mut vec = file.read();
                        let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.as_mut_vec().clone())), compiled_data: None };
                        stages.push(ShaderStage::new(file.get_file_name() + file.get_file_ext().as_str(), ShaderType::Fragment, ShaderLang::Glsl, shader_data));
                    },
                    "glsl" => {
                        //parse as single stage
                        let mut vec = file.read();
                        let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.as_mut_vec().clone())), compiled_data: None };
                        stages.push(ShaderStage::new(file.get_file_name() + file.get_file_ext().as_str(), ShaderType::Compute, ShaderLang::Glsl, shader_data));
                    },
                    "comp" => {
                        //parse as single stage
                        let mut vec = file.read();
                        let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.as_mut_vec().clone())), compiled_data: None };
                        stages.push(ShaderStage::new(file.get_file_name() + file.get_file_ext().as_str(), ShaderType::Compute, ShaderLang::Glsl, shader_data));
                    },
                    "hlsl" => {
                        //parse as single stage
                        let mut vec = file.read();
                        let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.as_mut_vec().clone())), compiled_data: None };
                        stages.push(ShaderStage::new(file.get_file_name() + file.get_file_ext().as_str(), ShaderType::Compute, ShaderLang::Hlsl, shader_data));
                    },
                    "fx" => {
                        //parse as single stage
                        let mut vec = file.read();
                        let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.as_mut_vec().clone())), compiled_data: None };
                        stages.push(ShaderStage::new(file.get_file_name() + file.get_file_ext().as_str(), ShaderType::Compute, ShaderLang::Hlsl, shader_data));
                    },
                    "pfx" => {
                        //parse as single stage
                        let mut vec = file.read();
                        let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.as_mut_vec().clone())), compiled_data: None };
                        stages.push(ShaderStage::new(file.get_file_name() + file.get_file_ext().as_str(), ShaderType::Compute, ShaderLang::Pgsl, shader_data));
                        panic!("Unimplemented!");
                    },
                    "gdsahd" => {
                        //parse as single stage
                        let mut vec = file.read();
                        let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.as_mut_vec().clone())), compiled_data: None };
                        stages.push(ShaderStage::new(file.get_file_name() + file.get_file_ext().as_str(), ShaderType::Compute, ShaderLang::GodotShader, shader_data));
                        panic!("Unimplemented!!");
                    }
                    _ => {
                        panic!("File is not a shader type!!");
                    }
                }
            }
            let mut stages_ptr = Vec::<ShaderPtr>::new();

            for stage in stages {
                unsafe {
                    stages_ptr.push(RenderPipelineSystem::register_shader_stage(stage));
                }
            }

            Self {shader_name: file.get_file_name(), shader_stages: stages_ptr, file_path: path}
        }

    }

    fn parse_shad_file(data: String) -> Vec<ShaderToken> {
        //tokenize the shader data
        #[derive(Clone)]
        pub enum Data {
            String(Vec<u8>),
            Vector(Vec<Vec<u8>>),
        }

        impl Data {
            pub fn as_str(&mut self) -> Vec<u8> {
                match self {
                    Data::String(s) => s.clone(),
                    _ => vec![]
                }
            }

            pub fn as_vec(&mut self) -> Vec<Vec<u8>> {
                match self {
                    Data::Vector(v) => v.clone(),
                    _ => vec![]
                }
            }
        }

        let mut tokens: Vec<ShaderToken> = vec![];

        let mut is_in_token: bool = false;
        let mut braket_count: u32 = 0;
        let mut is_quote: bool = false;
        let mut data_label: String = "".to_string();
        let mut d: Vec<u8> = vec![];
        let mut is_back: bool = false;
        let mut is_obtaining_data: bool = false;
        let mut data_list: HashMap<String,Data> = HashMap::new();
        let mut is_vector_data: bool = false;
        let mut vector_data: Vec<Vec<u8>> = vec![];
        let mut is_data: bool = false;
        let mut saved_data: Data = Data::String(vec![]);
        for c in data.as_bytes() {
            match c {
                b'{' => {
                    if is_quote || is_vector_data{
                        d.push(c.clone());
                        continue;
                    }
                    if is_in_token {

                    }
                    else {
                        is_in_token = true;
                    }
                    braket_count += 1;
                },
                b'}' => {
                    if is_quote || is_vector_data{
                        d.push(c.clone());
                        continue;
                    }
                    if is_in_token && braket_count > 1{
                        
                    }
                    else if is_in_token && braket_count == 1 {
                        is_in_token = false;
                        // theoretically have reached the end of all!!
                        let mut name_default = Data::String(vec![b's',b'h',b'a',b'd',b'e',b'r']);
                        let mut name_temp = data_list.get("shader_name").unwrap_or(&name_default).clone().as_str();
                        let mut _shader_name = String::from_utf8(name_temp).expect("Failed to parse shader name!!");
                        let type_default = Data::String(b"compute".to_vec());
                        let _shader_type = match data_list.get("shader_type").unwrap_or(&type_default).clone().as_str().as_slice() {
                            b"fragment" => ShaderType::Fragment,
                            b"vertex" => ShaderType::Vertex,
                            _ => ShaderType::Compute,
                        };
                        let lang_default = Data::String(b"glsl".to_vec());
                        let mut _shader_lang = match data_list.get("shader_lang").unwrap_or(&lang_default).clone().as_str().as_slice() {
                            b"hlsl" => ShaderLang::Hlsl,
                            b"pgsl" => ShaderLang::Pgsl,
                            b"godot" => ShaderLang::GodotShader,
                            _ => ShaderLang::Glsl,
                        };
                        let com_default = Data::String(b"false".to_vec());
                        let mut _is_compiled = match data_list.get("is_compiled").unwrap_or(&com_default).clone().as_str().as_slice() {
                            b"true" => true,
                            _ => false,
                        };
                        let mut inout_datas: Vec<(String, ShaderDataTypeClean, ShaderDataHint)> = vec![];
                        if data_list.contains_key("shader_inout_datas") {
                            let temp = data_list.get("shader_inout_datas").expect("No such value shader_inout_datas!!").clone().as_vec();
                            for val in temp 
                            {
                                unsafe {
                                    let string = String::from_utf8(val).expect("Failed to parse string!");
                                
                                    let temp_list: Vec<&str> = string.split(';').collect();
                                    
                                    let name = temp_list[0].to_string();
                                    let data_type = match temp_list[1] {
                                        "Integer" => ShaderDataTypeClean::Integer,
                                        "Boolean" => ShaderDataTypeClean::Boolean,
                                        "UnssignedInteger" => ShaderDataTypeClean::UnsignedInteger,
                                        "Float" => ShaderDataTypeClean::Float,
                                        "Double" => ShaderDataTypeClean::Double,
                                        "Vec3" => ShaderDataTypeClean::Vec3,
                                        "Vec4" => ShaderDataTypeClean::Vec4,
                                        "Vec2" => ShaderDataTypeClean::Vec2,
                                        "IVec3" => ShaderDataTypeClean::IVec3,
                                        "IVec4" => ShaderDataTypeClean::IVec4,
                                        "IVec2" => ShaderDataTypeClean::IVec2,
                                        "UVec3" => ShaderDataTypeClean::UVec3,
                                        "UVec4" => ShaderDataTypeClean::UVec4,
                                        "UVec2" => ShaderDataTypeClean::UVec2,
                                        "DVec3" => ShaderDataTypeClean::Vec3,
                                        "DVec4" => ShaderDataTypeClean::Vec4,
                                        "DVec2" => ShaderDataTypeClean::Vec2,
                                        "Sampler2D" => ShaderDataTypeClean::Sampler2D,
                                        _ => ShaderDataTypeClean::Float,
                                    };
                                    let data_hint = match temp_list[2] {
                                        "Uniform" => ShaderDataHint::Uniform,
                                        "In" => ShaderDataHint::In,
                                        "Out" => ShaderDataHint::Out,
                                        "InOut" => ShaderDataHint::InOut,
                                        "Buffer" => ShaderDataHint::Buffer,
                                        _ => ShaderDataHint::Uniform,
                                    };

                                    inout_datas.push((name, data_type, data_hint));
                                    
                                }
                            }
                        }
                        let default_code = Data::String(b"".to_vec());

                        tokens.push(ShaderToken { 
                            shader_name: _shader_name, 
                            shader_type: _shader_type, 
                            shader_inout_datas: inout_datas, 
                            shader_code: data_list.get("shader_code").unwrap_or_else(|| {&default_code}).clone().as_str().clone(),
                            shader_lang: _shader_lang,
                            is_compiled: _is_compiled
                        });
                                    
                                
                            
                        
                        data_list.clear();
                    }

                    if braket_count >= 1 {
                        braket_count -= 1;
                    }
                },
                b'\"' => {
                    if is_back {
                        d.push(c.clone());
                        is_back = false;
                        continue;
                    }
                    if is_quote {
                        is_quote = false;
                        //check if the string is name for property
                        if is_data && !is_vector_data {
                            // this must be data
                            saved_data = Data::String(d.clone());
                            d.clear();
                        }
                        
                    }
                    else{
                        is_quote = true;
                    }
                },
                b'\\' => {
                    if is_back {
                        d.push(c.clone());
                        is_back = false;
                        continue;
                    }
                    else{
                        is_back = true;
                    }
                },
                b':' => {
                    if is_quote{
                        d.push(c.clone());
                        continue;
                    }
                    if !is_data {
                        is_data = true;
                        unsafe {
                            data_label = String::from_utf8(d.clone()).expect("Failed to parse String!!");
                            d.clear();
                        }
                    }
                },
                b',' => {
                    if is_quote{
                        d.push(c.clone());
                        continue;
                    }
                    if is_vector_data {
                        vector_data.push(d.clone());
                        d.clear();
                        continue;
                    }
                    if is_data{
                        is_data = false;
                        data_list.insert(data_label.clone(), saved_data.clone());
                        data_label.clear();
                        saved_data = Data::String(b"".to_vec());
                        continue;
                    }
                },
                b'[' => {
                    if is_quote {
                        d.push(c.clone());
                        continue;
                    }
                    if !is_vector_data && is_data{
                        is_vector_data = true;
                    }
                },
                b']' => {
                    if is_quote {
                        d.push(c.clone());
                        continue;
                    }
                    if is_vector_data && is_data{
                        is_vector_data = false;
                        // Now we submit the data in vec_datato data list
                        saved_data = Data::Vector(vector_data.clone());
                        vector_data.clear();
                    }
                },
                _ => {
                    if is_quote {
                        d.push(c.clone());
                    }
                }

            }
        }

        return tokens;
    }
}

pub trait ShaderDescriptor{
    fn get_num_values(&self) -> isize;
    fn get_value_type(&self, offset: isize) -> TypeId;
    fn get_value(&self, offset: isize) -> Ptr<Box<&'static dyn Base>>;
    fn get_value_name(&self, offset: isize) -> String;
}

#[derive(Clone)]
pub enum ShaderDataType {
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

enum ShaderDataTypeClean {
    Integer,
    Boolean,
    UnsignedInteger,
    Float,
    Double,
    Vec3,
    Vec4,
    Vec2,
    IVec3,
    IVec4,
    IVec2,
    UVec3,
    UVec4,
    UVec2,
    DVec3,
    DVec4,
    DVec2,
    Sampler2D,

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
    pub shader_descriptor: HashMap<String, (Box<ShaderDataType>, ShaderDataHint)>,

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