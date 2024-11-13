
use std::fs::{*, self};
use std::io::{prelude::*, BufReader};
use std::sync::Arc;
use shaderc::CompileOptions;
use std::any::TypeId;

use crate::black_ice::common::{APP_DIR, materials};
use crate::black_ice::common::engine::gamesys::*;

#[cfg(target_os = "windows")] const ASSET_PATH: &str =  "F:\\Rust\\Program 1\\assets";
#[cfg(target_os = "linux")] const ASSET_PATH: &str = "/home/detrix/rust/black-ice/assets";
#[cfg(not(debug_assertions))] const IS_DEBUG: bool = false;
#[cfg(debug_assertions)] const IS_DEBUG: bool = true;

pub struct FileSys{

    f: Option<File>,
    b: Option<BufReader<File>>,
    total_len: usize,
    i: usize,
    pub path: String

}

impl Base for FileSys {}
impl Base for Option<File> {}
impl Reflection for FileSys {
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));
        
        register.addProp(Property{
            name: Box::new("path"), 
            desc: Box::new("Path of file."), 
            reference: Box::new(&self.path), 
            ref_type: TypeId::of::<String>()
        });
        return Ptr {b: register};
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MFType{

    NOEXT,
    FBX,
    OBJ,
    MTL,
    PNG,
    SHADER,
    UNKNOWN


}

trait Handlers {
    fn fbx_handler(&mut self) -> String;
    fn obj_handler(&mut self) -> String;
    fn mtl_handler(&mut self) -> String;
    fn png_handler(&mut self) -> String;
}

pub trait Reader<U> {
    fn read(&mut self) -> U;
    fn read_file(&mut self) -> U;
}

impl FileSys{

    pub fn new() -> FileSys{
        return FileSys { f: None, b: None, i: 0, total_len: 0, path: String::from("") };
    }

    pub fn open(&mut self, _path: &str){
        // do more to getting it for only local systems e.g. get resources from pak files when in release and from local assets when in debug


        if IS_DEBUG
        {
            let mut full_path = format!("{}\\{}", ASSET_PATH, _path[7..].to_owned());
            full_path = String::from(full_path).replace("\\", "/");
            let dir = fs::metadata(full_path.clone()).unwrap();

            if dir.is_file() {
                self.f = Option::Some(File::open(full_path.as_str()).expect(format!("File {} not found!", _path[7..].to_owned()).as_str()));// format "ASSET:\\path\\to\\file" => "DRIVE:\\path\\to\\assets\\path\\to\\file"
                self.path = String::from(_path);
                self.b = Option::Some(BufReader::new((*self.f.as_ref().unwrap()).try_clone().expect("Couldn't clone file for BufReader!!")));
            }
        }
        else 
        {
            // now get as asset pack!!
        }
    }

    pub fn get_file_ext(&self) -> String {
        let mut last_i = 0;

        for i in 0..(self.path.len()){

            if self.path.chars().nth(i).unwrap() == '.' {

                last_i = i;

            }
            
        }

        if last_i == 0 {
            return "".to_string();
        }

        self.path.get(last_i+1..).unwrap().to_string()
    }

    pub fn check_file_ext(&self) -> MFType{
        
        let mut last_i = 0;

        for i in 0..(self.path.len()){

            if self.path.chars().nth(i).unwrap() == '.' {

                last_i = i;

            }
            
        }

        if last_i == 0 {
            return MFType::UNKNOWN;
        }

        let ext = self.path.get(last_i+1..).unwrap();
        let mut mf = MFType::NOEXT;
        
        if ext.eq("fbx") || ext.eq(".fbx") {
            mf = MFType::FBX;
        }
        if ext.eq("png") || ext.eq(".png") {
            mf = MFType::PNG;
        }
        if ext.eq(".obj") || ext.eq("obj") {
            mf = MFType::OBJ;
        }
        if ext.eq("mtl") || ext.eq(".mtl") {
            mf = MFType::MTL;
        }
        if ext.eq(".shad") || ext.eq("shad") || ext.eq(".glsl") || ext.eq("glsl") || ext.eq(".comp") || ext.eq("comp") || ext.eq(".frag") || ext.eq("frag") || ext.eq(".vert") || ext.eq("vert") {
            mf = MFType::SHADER;
        }
        return mf;

    }

    pub fn get_file_name(&self) -> String {
        let mut last_i = 0;
        let mut last_slash = 0;

        for i in 0..(self.path.len()){

            if self.path.chars().nth(i).unwrap() == '.' {

                last_i = i;

            }
            if self.path.chars().nth(i).unwrap() == '/' {

                last_slash = i;

            }
            if self.path.chars().nth(i).unwrap() == '\\' {

                last_slash = i;

            }
            
        }

        if last_i == 0 {
            return "".to_string();
        }
        if last_slash == 0 {
            return "".to_string();
        }

        self.path.get(last_slash..last_i).unwrap().to_string()
    }

}

#[derive(Clone)]
pub struct ShaderFile {
    pub code: Vec<u32>,
    pub file: Vec<u8>,
    pub shader_kind: shaderc::ShaderKind,
    pub shader_path: String,
}

impl ShaderFile {
    
    pub fn compile(&mut self) -> bool {
        let result = std::panic::catch_unwind(||{


            let include_context = FileSys::include_shaders();
            let glsl_buff = include_context.expand(std::str::from_utf8(self.file.as_slice()).unwrap()).expect("Failed to load Shader File!!");
            let compiler = shaderc::Compiler::new().unwrap();
            let compiled_code = compiler.compile_into_spirv(&glsl_buff.as_str(), self.shader_kind, &self.shader_path.as_str(), "main", None)
                .expect("Failed to compile Shader");
            
            compiled_code
        
        
        });

        match result {
            Ok(v) => {
                self.code = v.as_binary().to_vec();
                true
            },
            Err(_) => false
        }
    }

}

pub trait AsShaderFile {
    fn as_shader_file(&mut self, kind: shaderc::ShaderKind) -> ShaderFile;
    fn include_shaders() -> glsl_include::Context<'static> ;
}

impl AsShaderFile for FileSys {
    fn as_shader_file(&mut self, kind: shaderc::ShaderKind) -> ShaderFile {
        let reader = self.b.as_mut().unwrap();
        let mut file_buff:String = String::new();
        reader.read_to_string(&mut file_buff).expect("Failed to read shader file!!");
        let include_context = Self::include_shaders();
        let glsl_buff = include_context.expand(file_buff.clone()).expect("Failed to load Shader File!!");
        let compiler = shaderc::Compiler::new().unwrap();
        let mut compile_options = CompileOptions::new().unwrap();
        compile_options.set_auto_bind_uniforms(true);
        compile_options.set_auto_map_locations(true); 
        #[cfg(feature = "opengl")]compile_options.set_target_env(shaderc::TargetEnv::OpenGL, shaderc::EnvVersion::OpenGL4_5 as u32);
        #[cfg(feature = "vulkan")]compile_options.set_taget_env(shaderc::TargetEnv::Vulkan, shaderc::EnvVersion::Vulkan1_2);  
        let compiled_code = compiler.compile_into_spirv(&glsl_buff.as_str(), kind, self.path.as_str(), "main", Some(&compile_options)).expect("Failed to compile Shader");
        

        ShaderFile { code: compiled_code.as_binary().to_vec(), file: file_buff.as_bytes().to_vec() , shader_kind: kind, shader_path: self.path.clone()}
    }

    fn include_shaders() -> glsl_include::Context<'static> {
        let path: String = APP_DIR.to_owned() + "\\assets\\shaders\\";
        let mut directory = fs::read_dir(path).unwrap();
        let mut context: glsl_include::Context = glsl_include::Context::new();
        let mut path_stack = Vec::<ReadDir>::new();
        let mut current_path = directory.next();
        'run: loop {
            if current_path.is_none() {
                if path_stack.is_empty() {
                    break 'run;
                }
                directory = path_stack.pop().unwrap();
                current_path = directory.next();
                continue;
            }
            let path_unwraped = current_path.unwrap().unwrap();
            let path_path = path_unwraped.path();
            if path_path.is_file() && materials::compare(path_unwraped.file_name().to_str().unwrap()) {
                let path_file_name = path_unwraped.file_name();
                let path_file_str = path_file_name.as_os_str().to_str().unwrap();
                
                let mut path_file = File::open(path_path.display().to_string()).expect("Error opening shader!!");
                let mut path_data = String::new();
                path_file.read_to_string(&mut path_data).expect("Failed to open shader file!!");
                let path_data_string = path_data.to_string();
                let path_data_str = path_data_string.as_str();
                context.include(path_file_str, path_data_str);
            }
            else if path_path.is_dir() && !path_path.ends_with(".shad"){
                path_stack.push(directory);
                directory = path_path.read_dir().unwrap();
            }
            current_path = directory.next();
        }
        return context;
    }
}

impl Reader<String> for FileSys {
    fn read(&mut self) -> String {

        let mut result:String = String::from("");

        match self.check_file_ext() {
            MFType::FBX=>{
                result = self.fbx_handler();
            },
            MFType::OBJ=>{
                result = self.obj_handler();
            },
            MFType::MTL=>{
                result = self.mtl_handler();
            },
            MFType::PNG=>{
                result = self.png_handler();
            },
            _=>{
                unsafe {
                    let _data = self.b.as_mut().expect("Failed to read file!!").read_to_end(result.as_mut_vec());
                }
            }
        }

        return result;
        
    }

    fn read_file(&mut self) -> String {

        let mut result:String = String::from("");

        match self.check_file_ext() {
            MFType::FBX=>{
                result = self.fbx_handler();
            },
            MFType::OBJ=>{
                result = self.obj_handler();
            },
            MFType::MTL=>{
                result = self.mtl_handler();
            },
            MFType::PNG=>{
                result = self.png_handler();
            },
            _=>{

            }
        }

        return result;
    }

}

impl Handlers for FileSys{
    fn fbx_handler(&mut self) -> String {

        unimplemented!();
    }
    fn mtl_handler(&mut self) -> String {
        unimplemented!()
    }
    fn obj_handler(&mut self) -> String {
        let buff = self.b.as_mut().unwrap();
        let mut result = String::from("");

        (*buff).read_to_string(&mut result).expect("Couldn't read anything!!");
        return result;
    }
    fn png_handler(&mut self) -> String {
        let buff = self.b.as_mut().unwrap();
        let mut temp: Vec<u8> = vec![];
        let result = String::from("");

        (*buff).read_to_end(&mut temp).expect("Couldn't read anything!!");
        return result;
    }

}
use std::fmt::Display;
#[allow(unused_assignments)]
impl Display for MFType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut name: String = String::from("");

        match self {
            &MFType::FBX=>{
                name = String::from("FBX");
            },
            &MFType::OBJ=>{
                name = String::from("OBJ");
            },
            &MFType::MTL=>{
                name = String::from("MTL");
            },
            &MFType::PNG=>{
                name = String::from("PNG");
            },
            &MFType::NOEXT=>{
                name = String::from("NO_EXT");
            },
            &MFType::UNKNOWN=>{
                name = String::from("UNKNOWN");
            },
            _=>{
                name = String::from("user defined");
            }
        }

        return write!(f, "{}", format!("{}", name));
    }
}


