#![feature(mutex_unlock)]
#![allow(unused)]
use std::any::Any;
use std::io::{Error, ErrorKind};
use std::os::raw::c_void;
use std::process::id;
use gl46::GL_LINK_STATUS;
use raw_window_handle::HasRawWindowHandle;
use sys::id_t;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, atomic::*};
use parking_lot::*;
use std::sync::atomic::AtomicI32;
use std::{future::*, u32};
use sdl2::*;
use colored::*;


use crate::black_ice::common::engine::asset_types::{materials::*, shader_asset::*};
use crate::black_ice::common::matrices::*;
use crate::black_ice::common::mesh::*;
use crate::black_ice::common::engine::*;
use crate::black_ice::common::vertex::*;

use super::gamesys::*;
use super::threading::*;

#[cfg(feature = "vulkan")] use super::vulkan::*;
#[cfg(feature = "opengl")] use super::opengl::*;
#[cfg(feature = "gles")] use super::gles::*;

// generic data enum to let us handle acceptable types of data
#[derive(Clone)]
pub enum Data {
    FloatSequence(Arc<Mutex<Vec<f32>>>),
    IntegerSequence(Arc<Mutex<Vec<i32>>>),
    I16Sequence(Arc<Mutex<Vec<i16>>>),
    DoubleSequence(Arc<Mutex<Vec<f64>>>),
    ImageSequence(Arc<Mutex<Image>>),
    Surface(Arc<Mutex<Surface>>),
    Float(f32),
    Integer(i32),
    I16(i16),
    Double(f64),
    VectorBuffer(Vec<Vec3>),
    Vector(Vec3),
    IVectorBuffer(Vec<Vec3>),
    IVector(Vec3),
    DVectorBuffer(Vec<Vec3>),
    DVector(Vec3),
    MeshMatrix(Matrix34),
    Matrix(Matrix34),

}

pub struct Pipeline {
    pub id: i32,
    pub name: String,
    pub cameras: Vec<CameraDriver>,
    pub layer: u32,
    pub driver: Arc<Mutex<Option<DriverValues>>>,
    pub shaders: HashMap<String, (u32, Vec<u32>,Option<ShaderStageDescriptor>)>,
    pub is_init: bool,
    counter: AtomicI32,
}

#[derive(Clone)]
pub struct PipelineParams {
    pub name: String,
    pub layer: u32,
}

unsafe impl Send for Pipeline {}

// Need to change this for generic gpu rendering and compute shaders!!
impl Pipeline {

    pub fn register_camera(&mut self, p_camera: Arc<Mutex<Camera>>){
        self.cameras.push(CameraDriver::new(p_camera.clone()));
    }

    pub fn register_shader_program(&mut self, shader: Shader) {
        #[cfg(feature="opengl")] self.register_shader_program_gl(shader);
        #[cfg(feature="vulkan")] self.register_shader_program_vk(shader);
    }
    
    fn register_shader_program_gl(&mut self, shader: Shader) -> (u32, Vec<u32>, Option<ShaderStageDescriptor>) {
        let mut shader_program: Option<(u32, Vec<u32>, Option<ShaderStageDescriptor>)> = self.shaders.get(&shader.asset_path).cloned();
        
        
        // check if the shader program exists
        if shader_program.is_none() {
            let mut p_driver = self.driver.lock();
            let mut driver = p_driver.as_mut().unwrap();
            let mut converted_stages: Vec<u32> = vec![];
            let mut uniforms = ShaderStageDescriptor::new();
            for stage in shader.shader_stages.clone() {
                // we need to check if there has already been a shader registered
                // we don't want to keep running this
                unsafe{                    
                    let mut shader_stage = RenderPipelineSystem::get_shader_stage(stage);
                    
                    //uniforms.append(&mut stage_uniforms);
                    match shader_stage.shader_lang {
                        ShaderLang::Glsl => {
                            match shader_stage.shader_type {
                                ShaderType::Compute => {
                                    let shader_id = driver.gl.as_ref().unwrap().CreateShader(gl46::GL_COMPUTE_SHADER);

                                    // Using SPIR-V 
                                    // Always enter from "main"
                                    if shader_stage.shader_data.compiled_data.is_none() {
                                        panic!("Shader was not compiled!!!");
                                    }
                                    let p_binary = shader_stage.shader_data.compiled_data.expect("Failed to compile Shader!!!").clone();
                                    let c_binary = p_binary.lock();
                                    let binary = c_binary.clone();
                                    drop(c_binary);
                                    let length = binary.len().clone() as i32;
                                    driver.gl.as_ref().unwrap().ShaderBinary(1, &shader_id, gl46::GL_SHADER_BINARY_FORMAT_SPIR_V, binary.as_ptr() as *const c_void, length);
                                    let entry = std::ffi::CString::new("main").unwrap();
                                    driver.gl.as_ref().unwrap().SpecializeShader(shader_id, entry.as_ptr() as *const u8, 0, std::ptr::null(), std::ptr::null());
                                    //driver.gl.as_ref().unwrap().CompileShader(shader_id);
                                    let mut compiled = 0;
                                    driver.gl.as_ref().unwrap().GetShaderiv(shader_id, gl46::GL_COMPILE_STATUS, &mut compiled);

                                    if compiled == 0 {
                                        panic!("Failed to compile the shader for rendering!!!");
                                    }
                                    
                                    converted_stages.push(shader_id);
                                },
                                ShaderType::Fragment => {
                                    
                                    let shader_id = driver.gl.as_ref().unwrap().CreateShader(gl46::GL_FRAGMENT_SHADER);
                                    println!("{:#?}", driver.gl.as_ref().unwrap().GetError());
                                    // Using SPIR-V 
                                    // Always enter from "main"
                                    if shader_stage.shader_data.compiled_data.is_none() {
                                        panic!("Shader was not compiled!!!");
                                    }
                                    let p_binary = shader_stage.shader_data.compiled_data.expect("Failed to compile Shader!!!").clone();
                                    let c_binary = p_binary.lock();
                                    let binary = c_binary.clone();
                                    drop(c_binary);
                                    let length = binary.len().clone() as i32;
                                    driver.gl.as_ref().unwrap().ShaderBinary(1, &shader_id, gl46::GL_SHADER_BINARY_FORMAT_SPIR_V, binary.as_ptr() as *const c_void, length);
                                    println!("{:#?}", driver.gl.as_ref().unwrap().GetError());
                                    let entry = std::ffi::CString::new("main").unwrap();
                                    driver.gl.as_ref().unwrap().SpecializeShader(shader_id, entry.as_ptr() as *const u8, 0, 0 as *const _, 0 as *const _);
                                    println!("{:#?}", driver.gl.as_ref().unwrap().GetError());
                                    //driver.gl.as_ref().unwrap().CompileShader(shader_id);
                                    let mut compiled = 0;
                                    driver.gl.as_ref().unwrap().GetShaderiv(shader_id, gl46::GL_COMPILE_STATUS, &mut compiled);
                                    // let mut len = 0;
                                    // let mut log: [u8; 512] = [0;512];
                                    // driver.gl.as_ref().unwrap().GetShaderInfoLog(shader_id, 512, &mut len, log.as_mut_ptr());
                                    println!("{:#?}", driver.gl.as_ref().unwrap().GetError());

                                    if compiled == 0 {
                                        panic!("Failed to compile the shader for rendering!!!");
                                    }
                                    
                                    converted_stages.push(shader_id);
                                    
                                },
                                ShaderType::Vertex => {
                                    let mut shader_id = driver.gl.as_ref().unwrap().CreateShader(gl46::GL_VERTEX_SHADER);
                                    // Using SPIR-V 
                                    // Always enter from "main"
                                    if shader_stage.shader_data.compiled_data.is_none() {
                                        panic!("Shader was not compiled!!!");
                                    }
                                    let p_binary = shader_stage.shader_data.compiled_data.expect("Failed to compile Shader!!!").clone();
                                    let c_binary = p_binary.lock();
                                    let binary = c_binary.clone();
                                    drop(c_binary);
                                    let length = binary.len().clone() as i32;
                                    driver.gl.as_ref().unwrap().ShaderBinary(1, &shader_id, gl46::GL_SHADER_BINARY_FORMAT_SPIR_V, binary.as_ptr() as *const c_void, length);
                                    let entry = std::ffi::CString::new("main").unwrap();
                                    driver.gl.as_ref().unwrap().SpecializeShader(shader_id, entry.as_ptr() as *const u8, 0, std::ptr::null(), std::ptr::null());
                                    
                                    let mut compiled = 0;
                                    driver.gl.as_ref().unwrap().GetShaderiv(shader_id, gl46::GL_COMPILE_STATUS, &mut compiled);

                                    if compiled == 0 {
                                       panic!("Failed to compile the shader for rendering!!!");
                                    }
                                    
                                    converted_stages.push(shader_id);

                                },
                                ShaderType::Infer => {
                                    let shader_type_infered = shader_stage.shader_data.infer_shader_type();
                                    let shader_type = match shader_type_infered {
                                        ShaderType::Compute => (gl46::GL_COMPUTE_SHADER),
                                        ShaderType::Fragment => (gl46::GL_FRAGMENT_SHADER),
                                        ShaderType::Vertex => (gl46::GL_VERTEX_SHADER),
                                        _ => panic!("No shader type defined in file. Please add #pragma shader_type(shader type) to your file!!")
                                    };
                                    let mut shader_id = driver.gl.as_ref().unwrap().CreateShader(shader_type);
                                    // Using SPIR-V 
                                    // Always enter from "main"
                                    if shader_stage.shader_data.compiled_data.is_none() {
                                        panic!("Shader was not compiled!!!");
                                    }
                                    let p_binary = shader_stage.shader_data.compiled_data.expect("Failed to compile Shader!!!").clone();
                                    let c_binary = p_binary.lock();
                                    let binary = c_binary.clone();
                                    drop(c_binary);
                                    let length = binary.len().clone() as i32;
                                    driver.gl.as_ref().unwrap().ShaderBinary(1, &shader_id, gl46::GL_SHADER_BINARY_FORMAT_SPIR_V, binary.as_ptr() as *const c_void, length);
                                    let entry = std::ffi::CString::new("main").unwrap();
                                    driver.gl.as_ref().unwrap().SpecializeShader(shader_id, entry.as_ptr() as *const u8, 0, std::ptr::null(), std::ptr::null());
                                    
                                    let mut compiled = 0;
                                    driver.gl.as_ref().unwrap().GetShaderiv(shader_id, gl46::GL_COMPILE_STATUS, &mut compiled);

                                    if compiled == 0 {
                                        panic!("Failed to compile the shader for rendering!!!");
                                    }
                                    
                                    converted_stages.push(shader_id);
                                }
                            }
                        },
                        ShaderLang::Hlsl => {
                            // get the shaders in the file and the shader function names
                            let shader_entries = shader_stage.shader_data.get_hlsl_shaders();

                            let mut shader_ids: Vec<u32> = vec![];
                            
                            for (shader_entry, shader_type) in &shader_entries {
                                let shader_id = driver.gl.as_ref().unwrap().CreateShader(match shader_type {
                                    ShaderType::Compute => gl46::GL_COMPUTE_SHADER,
                                    ShaderType::Fragment => gl46::GL_FRAGMENT_SHADER,
                                    ShaderType::Vertex => gl46::GL_VERTEX_SHADER,
                                    ShaderType::Infer => panic!("No shader type has been defined!!!!"),
                                });
                                
                                shader_ids.push(shader_id);
                            }
                            if shader_stage.shader_data.compiled_data.is_none() {
                                panic!("Shader was not compiled!!!");
                            }
                            let p_binary = shader_stage.shader_data.compiled_data.expect("Failed to compile Shader!!!").clone();
                            let c_binary = p_binary.lock();
                            let binary = c_binary.clone();
                            drop(c_binary);
                            let length = binary.len().clone() as i32;
                            driver.gl.as_ref().unwrap().ShaderBinary(shader_ids.len() as i32, shader_ids.as_ptr(), gl46::GL_SHADER_BINARY_FORMAT_SPIR_V, binary.as_ptr() as *const std::os::raw::c_void, length);

                            for i in 0..shader_ids.len() {
                                let shader_id = shader_ids[i];
                                let shader_entry = &shader_entries[i].0;
                                let entry = std::ffi::CString::new(shader_entry.clone()).unwrap();
                                driver.gl.as_ref().unwrap().SpecializeShader(shader_id, entry.as_ptr() as *const u8, 0, std::ptr::null(), std::ptr::null());
                                
                                let mut compiled = 0;

                                driver.gl.as_ref().unwrap().GetShaderiv(shader_id, gl46::GL_COMPILE_STATUS, &mut compiled);

                                if compiled == 0 {
                                    panic!("Failed to compile the shader!!!");
                                }
                            }
                            
                            converted_stages.append(&mut shader_ids);
                        },
                        ShaderLang::Pssl => {
                            unimplemented!("Todo!");
                        },
                        ShaderLang::GodotShader => {
                            unimplemented!("Todo!");
                        },
                    }
                    let mut stage_uniforms = shader_stage.shader_data.descriptor.clone();
                    uniforms.append(&mut stage_uniforms);
                }
            }

            let gl = driver.gl.as_ref().unwrap();
            let program_id = gl.CreateProgram();
            for stage in &converted_stages {
                gl.AttachShader(program_id, *stage);
            }
            gl.LinkProgram(program_id);

            let mut status = 0;
            unsafe {gl.GetProgramiv(program_id, GL_LINK_STATUS, &mut status);}
            if status == 0 {
                panic!("Shader Program did not link correctly!!");
            }
            // lets add this to the system
            unsafe{
                let p_rend = Env::get_render_sys();
                let mut rend = p_rend.write();
                shader_program = Some((program_id.clone(), converted_stages.clone(), Some(uniforms.clone())));
                self.shaders.insert(shader.asset_path.clone(), (program_id, converted_stages.clone(), Some(uniforms)));
                drop(rend);
            }
        }
        shader_program.unwrap()
    }

    fn register_shader_program_vk(&mut self, shader: Shader) {

    }
}

#[derive(Clone)]
pub struct Camera {
    pub cam_id: i32,
    pub projection: MatrixProjection,
    pub transform: Matrix34,
    pub render_texture: Option<RenderTexture>,
    pub up: Vec3,
    pub forward: Vec3,
    is_active: bool,
}

impl Camera {
    
    pub fn new(id: i32) -> Self {
        Self { 
            cam_id: id, 
            projection: MatrixProjection::new(), 
            transform: Matrix34::identity(), 
            render_texture: None, 
            is_active: false, 
            up: Vec3::new(0.0, 0.0, 1.0), 
            forward:Vec3::new(1.0, 0.0, 0.0) 
        }
    }

}

pub struct Image {
    data: Vec<[u32;4]>,
    width: u32,
    height: u32,
    pub is_atlas: bool,
    pub is_full: bool,
    max_width: u32,
    max_height: u32,
    uvs: HashMap<String, (u32, u32)>,
}

impl Image {
    pub fn new(im_data: Vec<[u32;4]>, width: u32, height: u32, is_atlas: bool) -> Self {
        Self { 
            data: im_data, 
            width: width, 
            height: height, 
            is_atlas: is_atlas, 
            is_full: false, 
            max_width: u32::MAX, 
            max_height: u32::MAX,
            uvs: HashMap::new(),
        }
    }
}

pub struct Did {
    pub id: u32,
    is_free: Arc<Mutex<bool>>,
    queued_free: Arc<Mutex<bool>>,
}

impl Did {

    fn is_freed(&self) -> bool {
        *self.is_free.lock() || *self.queued_free.lock()
    }

    fn queue_free(&mut self) {
        let mut v = self.queued_free.lock();
        *v = true;
    }
}

trait DidCreation {
    fn new(_id: u32) -> Self where Self: Sized;
}

impl DidCreation for Did {
    fn new(_id: u32) -> Self where Self: Sized {
        Did {id: _id, is_free: Arc::new(Mutex::new(false)), queued_free: Arc::new(Mutex::new(false))}
    }
}

// TODO: Add mesh reference vector so that we don't reference meshes only in the pipeline
// (Saves on memory and don't want to store multiple instances of meshes)
pub struct RenderPipelineSystem {
    pub pipelines: Vec<Arc<Mutex<Pipeline>>>,
    counter: AtomicI32,
    thread_count: usize,
    threads: Dict<usize, Arc<Mutex<Threader>>>,
    thread_reciever: Arc<Mutex<Vec<ThreadData>>>,
    driver_vals: Arc<Mutex<Option<DriverValues>>>,
    cameras: Vec<Arc<Mutex<Camera>>>,
    active_camera: i32,
    ready: bool,
    pub video: Arc<Mutex<sdl2::VideoSubsystem>>, 
    pub window: Arc<Mutex<sdl2::video::Window>>,
    pub sdl: Arc<Mutex<sdl2::Sdl>>,
    shader_stages_data: Vec<ShaderStage>,
    registered_images: HashMap<String, Arc<Mutex<Image>>>,
    pub registered_shaders: HashMap<String, (String, Vec<u8>)>,
    pub shader_programs:HashMap<String, (u32, Vec<u32>)>,
    input_data: HashMap<u32, Data>

}

unsafe impl Send for RenderPipelineSystem {}
unsafe impl Sync for RenderPipelineSystem {}

impl RenderPipelineSystem{
    // TODO: Fix these so that it doesn't borrow self!!
    pub unsafe fn resgister_pipeline(params: PipelineParams) -> i32{
        let p_this = Env::get_render_sys().clone();
        let mut this = p_this.write();
        let id = this.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let p = Arc::new(Mutex::new(Pipeline {
            id: id.clone(),
            name: params.name.clone(),
            layer: params.layer,
            driver: this.driver_vals.clone(),
            cameras: Vec::new(),
            is_init: false,
            counter: AtomicI32::new(0),
            shaders: HashMap::new(),
        }));
        
        this.pipelines.push(p);
        id
    }

    pub unsafe fn register_shader(layer: u32, shader: Shader){
        // we are going to create a shader program out of the shader stages
        let p_this = Env::get_render_sys();
        let this = p_this.write();
        let pipelines = this.pipelines.clone();
        drop(this);
    }

    pub unsafe fn render_shader(layer: u32, shader:Shader) {
        let p_this = Env::get_render_sys();
        let this = p_this.write();
        let pipelines = this.pipelines.clone();
        drop(this);
    }

    pub unsafe fn register_camera(&mut self, layer: u32) -> i32 {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let p_cam = Arc::new(Mutex::new(Camera::new(id.clone())));

        let pos = self.cameras.len();

        self.cameras.push(p_cam.clone());

        for p in &self.pipelines {
            let mut pipeline = p.lock();
            if pipeline.layer == layer {
                pipeline.register_camera(p_cam.clone());
            }
        }

        id

    }

    pub unsafe fn register_shader_stage(shader_stage: ShaderStage) -> usize{
        let mut p_rend = Env::get_render_sys();
        let mut rend = p_rend.write();
        let mut i = 0;
        for stages in &rend.shader_stages_data {
            if stages.stage_name.eq(&shader_stage.stage_name) {
                return i;
            }
            i += 1;
        }
        i = rend.shader_stages_data.len();
        rend.shader_stages_data.push(shader_stage);
        return i;
    }

    // pub unsafe fn update_shader(shader_data: ShaderData, shader_ptr: usize) {
    //     let mut p_rend = Env::get_render_sys();
    //     let mut rend = p_rend.write();
    //     let mut i = 0;
    //     let mut stage = rend.shader_stages_data.get_mut(shader_ptr).expect("Shader pointer out of range!!");
    //     // Looks like this may cause a memory leak - pls check!!
    //     stage.shader_data = shader_data;
    // }

    pub unsafe fn get_shader_stage(stage: usize) -> ShaderStage {
        let mut p_rend = Env::get_render_sys();
        let mut rend = p_rend.read();
        rend.shader_stages_data.get(stage).expect("No such shader registered!! Something must have gone wrong with shader registration!!").clone()
    } 

    pub fn update_camera(&mut self, id: i32, projection: &MatrixProjection, transform: &Matrix34, up: Vec3, forward: Vec3) {
        let p_cam = self.cameras.iter().find(|v| {let vv = v.lock(); vv.cam_id == id}).clone().expect("No such registered camera!!");
        let mut cam = p_cam.lock();
        cam.projection = projection.clone();
        cam.transform = transform.clone();
        cam.up = up;
        cam.forward = forward;
    }

    pub fn camera_set_active(&mut self, id: i32) {
        if id == self.active_camera {
            return;
        }
        
        let p_active_cam = self.cameras.iter().find(|v| {let vv = v.lock(); vv.cam_id == self.active_camera}).clone();
        if let Some(pp) = p_active_cam {
            let mut active_cam = pp.lock();
            active_cam.is_active = false;
        }
        let p_cam = self.cameras.iter().find(|v| {let vv = v.lock(); vv.cam_id == id}).clone().expect("No such registered camera!!");
        let mut cam = p_cam.lock();
        cam.is_active = true;
        self.active_camera = id;
        
    }

    pub fn camera_set_render_texture(&mut self, id: i32, texture_type: TextureType, width: i32, height: i32) {
        unsafe{
            let p_cam = self.cameras.iter().find(|v| {let vv = v.lock(); vv.cam_id == id}).clone().expect("No such registered camera!!");
            let mut cam = p_cam.lock();
            let mut p_driver = self.driver_vals.lock();
            let mut driver = p_driver.as_mut().unwrap();
            cam.render_texture = Some(DriverValues::create_render_texture(driver, width, height, texture_type));
            
        }
    }

    pub fn register_image(&mut self, image_data: &Vec<[u32; 4]> , width: u32, height: u32, depth: u32, image_name:String) -> Arc<Mutex<Image>> {
        let result = self.registered_images.get(&image_name);
        if let Some(value) = result {
            return value.clone();
        }

        // we didn't find anything, so we must register it!!
        let image = Arc::new(Mutex::new(Image::new(image_data.clone(), width, height, false)));
        self.registered_images.insert(image_name, image.clone());
        image
    }

    pub fn find_image(&self, image_name: String) -> Result<Arc<Mutex<Image>>, std::io::ErrorKind> {
        self.registered_images.get(&image_name).cloned().ok_or(std::io::ErrorKind::NotFound)
    }

    pub fn register_shader_data(shader_name: String, asset_path: String, data: Vec<u8>) {
        unsafe {
            let p_render_sys = Env::get_render_sys();
            let mut render_sys = p_render_sys.write();
            render_sys.registered_shaders.insert(shader_name, (asset_path, data));
        }
    }

    pub fn new(sdl: Arc<Mutex<Sdl>>, video: Arc<Mutex<sdl2::VideoSubsystem>>, window: Arc<Mutex<sdl2::video::Window>>) -> RenderPipelineSystem {

        let pip_sys = RenderPipelineSystem {
            pipelines: Vec::new(),
            counter: AtomicI32::new(1),
            thread_count: 0,
            threads: Dict::<usize, Arc<Mutex<Threader>>>::new(),
            thread_reciever: Arc::new(Mutex::new(Vec::new())),
            driver_vals: crate::black_ice::common::components::component_system::ComponentRef_new(Some(DriverValues::default())),
            cameras: Vec::new(),
            active_camera: 0,
            ready: false,
            window: window,
            video: video,
            sdl: sdl,
            shader_stages_data: vec![],
            registered_images: HashMap::<String, Arc<Mutex<Image>>>::new(),
            registered_shaders: HashMap::new(),
            shader_programs: HashMap::new(),
            input_data: HashMap::new(),
        };
        return pip_sys;
    }

    pub fn processing<'a>(p_this: Arc<RwLock<Self>>) -> i32{
        unsafe{
           
            let this = p_this.read();
            let p_recv = this.thread_reciever.clone();
            let p_pipelines = this.pipelines.clone();
            let p_window = this.window.clone();
            let p_video = this.video.clone();
            drop(this);
            for p in p_pipelines{
                let pipe = p.lock();
                let is_init = pipe.is_init.clone();
                drop(pipe);
                if !is_init {
                    #[cfg(feature = "vulkan")]VulkanRender::init(p.clone());
                    #[cfg(feature = "opengl")]OGlRender::init(p.clone());
                    #[cfg(feature = "gles")]GLESRender::init(p.clone());
                }
                
                #[cfg(feature = "vulkan")]VulkanRender::render(p.clone(), p_window.clone(), p_video.clone());
                #[cfg(feature = "opengl")]OGlRender::render(p.clone(), p_window.clone(), p_video.clone());
                #[cfg(feature = "gles")]GLESRender::render(p.clone(), p_window.clone(), p_video.clone());
            }
                
                // std::thread::sleep(std::time::Duration::from_millis(5));
                
        }
        
        0
    }

    pub fn cleanup(p_this: Arc<RwLock<Self>>){
        let this = p_this.read();
        let p_pipelines = this.pipelines.clone();
        drop(this);
        for p in p_pipelines{
                
            #[cfg(feature = "vulkan")]VulkanRender::cleanup(p.clone());
            #[cfg(feature = "opengl")]OGlRender::cleanup(p.clone());
            #[cfg(feature = "gles")]GLESRender::cleanup(p.clone());
        }
        //println!("Closing thread!!");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    pub fn init(p_this: Arc<RwLock<Self>>){
        unsafe{
            let this = p_this.write();
            // Initialise pipeline stuff
            #[cfg(feature = "vulkan")]DriverValues::init_vulkan(this.driver_vals.lock().as_mut().unwrap(), &this.window.lock(), &this.video.lock());
            #[cfg(feature = "opengl")]DriverValues::init_ogl(this.driver_vals.lock().as_mut().unwrap(), &this.window.lock(), &this.video.lock());
            #[cfg(feature = "gles")]DriverValues::init_gles(this.driver_vals.lock().as_mut().unwrap(), &this.window.lock(), &this.video.lock());


            Env::set_status(gamesys::StatusCode::RENDER_INIT);
        }
    }

    pub unsafe fn set_mouse_position(x: i32, y: i32)
    {
        let p_rend_sys = Env::get_render_sys();
        let mut rend_sys = p_rend_sys.read();
        let window = rend_sys.window.lock();
    }

    pub fn get_sdl() -> Arc<Mutex<sdl2::Sdl>> {
        unsafe{
            let p_rend = Env::get_render_sys();
            let rend = p_rend.read();
            rend.sdl.clone()
        }
    }

    //region Vulkan Render 


    //endregion

    //region OpenGL


    //endregion

    //region GLES


    //endregion


    // pub fn quick_redner<'a>(&mut self, vertices: Vec<Vertex>, faces: Vec<(i32, i32, i32)>, tex_coord: Vec<[f32; 2]>, image: std::iter::Enumerate<imagine::png::PngRawChunkIter<'a>>){
        
    //     let mut pallete = Vec::<Vec4>::new();
    //     let mut data = Vec::<Vec4>::new();
        
    //     for (n, raw_chunk) in image {
    //         let chunk_res = imagine::png::PngChunk::try_from(raw_chunk).unwrap();
    //         match chunk_res {
    //             imagine::png::PngChunk::sRGB(srgb) => {
                    
    //             },
    //             imagine::png::PngChunk::PLTE(plte) => {
    //                 for d in plte.entries() {
    //                     pallete.push(Vec4::new(d[0] as f32, d[1] as f32, d[2] as f32, 0.0));
    //                 }
    //             },
    //             imagine::png::PngChunk::tRNS(trns) => {
    //                 for (a, c) in trns.to_alphas().iter().zip(pallete.iter_mut()) {
    //                     c.w = *a as f32;
    //                 }
    //             },
    //             imagine::png::PngChunk::IDAT(idat) => {
    //                 let data_string = format!("{:?}", idat);
    //                 for b in data_string.as_bytes() {
    //                     data.push(pallete[*b as usize]);
    //                 }
    //             },
    //             _ => continue
    //         }
    //     }
    //     'test: loop{
    //         let p_recv = self.thread_reciever.clone();
    //         let mut recv = match p_recv.try_lock() {
    //             Some(re) => re,
    //             None => continue 'test
    //         };
    //         recv.push(ThreadData::QuickDraw(vertices.clone(), faces.clone(), tex_coord.clone(), data.clone()));
    //         drop(recv);
    //         break;
    //     }
    // }

}


// #[cfg(feature = "opengl")]
// pub trait OGLRender {
//     fn init(&self) -> i32;
//     fn render(th: Arc<Mutex<Self>>) -> i32;
// }

// #[cfg(feature = "opengl")]
// #[derive(Default, Clone)]
// pub struct DriverValues {}

// #[cfg(feature = "gles")]
// pub trait GLESRender {
//     fn init(&self) -> i32;
//     fn render(th: Arc<Mutex<Self>>) -> i32;
// }

// #[cfg(feature = "gles")]
// #[derive(Default, Clone, Clone)]
// pub struct DriverValues {}


