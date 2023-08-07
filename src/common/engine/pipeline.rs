#![feature(mutex_unlock)]
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::*;
use std::sync::atomic::AtomicI32;
use sdl2::*;

use crate::common::materials::BakeVulkan;
use crate::common::mesh::*;
use crate::common::engine::*;

use super::gamesys::*;
use super::threading::*;

#[cfg(feature = "vulkan")] use ash::*;
#[cfg(feature = "opengl")] use ogl33::*;
#[cfg(feature = "gles")] use opengles::*;

pub struct Pipeline {
    id: i32,
    pub name: String,
    meshs: Vec<Arc<Mutex<Mesh>>>,
    pub layer: usize,
    device: Option<vk::PhysicalDevice>,
}

pub struct PipelineParams {
    pub name: String,
    pub layer: usize,
}

impl Pipeline {
    pub fn register_mesh(&mut self, mesh: Arc<Mutex<Mesh>>){
        self.meshs.push(mesh);
    }
}

pub struct RenderPipelineSystem {
    pub pipelines: Vec<Pipeline>,
    counter: AtomicI32,
    statusCode: gamesys::SatusCode,
    threadCount: usize,
    threads: Dict<usize, Arc<Mutex<Threader>>>,
    device: Option<vk::PhysicalDevice>,
}

impl RenderPipelineSystem{
    pub fn resgister_pipeline(&mut self, params: PipelineParams){
        let pipeline = Pipeline {
            id: self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            name: params.name.clone(),
            meshs: Vec::new(),
            layer: params.layer,
            device: self.device.clone()
        };
        if(self.threadCount < params.layer){
            self.threadCount = params.layer;
        }
        self.pipelines.push(pipeline);
    }

    pub fn register_mesh(&mut self, id: i32, mesh: Arc<Mutex<Mesh>>){
        for p in &mut self.pipelines {
            let mut pipeline = p;
            if(pipeline.id == id){
                let m = mesh.clone();
                pipeline.register_mesh(m);
                return;
            }
        }
    }

    pub fn new() -> RenderPipelineSystem {
        let pipSys = RenderPipelineSystem {
            pipelines: Vec::new(),
            counter: AtomicI32::new(1),
            statusCode: gamesys::SatusCode::RUNNING,
            threadCount: 0,
            threads: Dict::<usize, Arc<Mutex<Threader>>>::new(),
            device: None,
        };
        return pipSys;
    }

    fn processing(&'static mut self) -> i32{
        unsafe{
            
            // Initialise pipeline stuff
            #[cfg(feature = "vulkan")]self.init_vulkan();
            #[cfg(feature = "opengl")]self.init_ogl();
            #[cfg(feature = "gles")]self.init_gles();
             // prebake step (only for when we have a better file system)
             // Create a new thread purely for baking and wait for this to finish
             // TODO: Implement better file system for storing shaders


            // first we setup threads for each layer
            //let mut threads: Box<Dict<usize, Arc<Mutex<Threader>>>> = Box::new(Dict::<usize, Arc<Mutex<Threader>>>::new());
            
            while (!GAME.isExit()) {
                for p in &self.pipelines{
                    let pipeline = p;

                    let mut a = self.threads.get_or_insert(pipeline.layer, Arc::new(Mutex::new(Threader::new()))).unwrap();
                    let mut b = a.lock().unwrap();
                    if(b.isAlive()){
                        b.stop();
                    }
                    b.start(|| pipeline.render());
                }
            }
        
        }
        0
    }

    pub fn init(&'static mut self) -> std::thread::JoinHandle<i32> {
        // Start thread
        let thread = std::thread::Builder::new().spawn(|| {self.processing()});
        return thread.ok().unwrap();
    }

    #[cfg(feature = "vulkan")]
    unsafe fn init_vulkan(&mut self)  {
        use winit::dpi::Pixel;
        
        unsafe fn checkDeviceSuitability(physical_device: &vk::PhysicalDevice, instance: &Instance) -> bool{
            let physical_device_properties = instance.get_physical_device_properties(*physical_device);
            let physical_device_features = instance.get_physical_device_features(*physical_device);

            return (physical_device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU) && physical_device_features.geometry_shader == 1;
        }

        unsafe fn rate_physical_device(physical_device: &vk::PhysicalDevice, instance: &Instance) -> u32 {
            let mut score: u32 = 0;
            let physical_device_properties = instance.get_physical_device_properties(*physical_device);
            let physical_device_features = instance.get_physical_device_features(*physical_device);
            
            if(physical_device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU) {
                score += 1000000;
            }

            score += physical_device_properties.limits.max_image_dimension2_d;
            println!("{name}: {score} : {hasGeom}", name=std::ffi::CStr::from_ptr(physical_device_properties.device_name.as_ptr()).to_str().unwrap(), hasGeom = physical_device_features.geometry_shader);
            if physical_device_features.geometry_shader == 0 {
                return 0;
            }
            
            score
        }
        let entry = ash::Entry::load().expect("Failed to get entry!");
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!(
            "{} - Vulkan Instance {}.{}.{}",
            "cock and balls",
            vk::api_version_major(entry.try_enumerate_instance_version().unwrap().unwrap()),
            vk::api_version_minor(entry.try_enumerate_instance_version().unwrap().unwrap()),
            vk::api_version_patch(entry.try_enumerate_instance_version().unwrap().unwrap())
        );
    

        let mut appInfo = vk::ApplicationInfo::default();
        appInfo.s_type = vk::StructureType::APPLICATION_INFO;
        let gameName =  GAME.gameName.lock().unwrap().clone();
        let gm = std::ffi::CString::new(gameName.as_str()).unwrap();
        appInfo.p_application_name = gm.as_ptr();
        appInfo.application_version = vk::make_api_version(0, 0, 1, 0);
        let engine_name = std::ffi::CString::new("Rusty Engine").unwrap();
        appInfo.p_engine_name = engine_name.as_ptr();
        appInfo.engine_version = vk::make_api_version(0, 0, 1, 0);
        appInfo.api_version = vk::make_api_version(0, 1, 3, 241);
        
        let mut extension_names =
        GAME.window.vulkan_instance_extensions()
            .unwrap()
            .to_vec();
        extension_names.push(ash::extensions::ext::DebugUtils::name().to_str().unwrap());
        let layer_names = [std::ffi::CStr::from_bytes_with_nul_unchecked(
            b"VK_LAYER_KHRONOS_validation\0",
        )];
        let layers_names_raw: Vec<*const std::os::raw::c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();
        let create_flags = vk::InstanceCreateFlags::default();
        let mut createInfo = vk::InstanceCreateInfo::default();
        createInfo.p_application_info = &appInfo;
        createInfo.pp_enabled_extension_names = extension_names.into_iter().map(|s| s.as_ptr().cast::<i8>()).collect::<Vec<*const i8>>().as_ptr();
        createInfo.pp_enabled_layer_names = layers_names_raw.as_ptr();
        createInfo.flags = create_flags;
        println!("Creating inst.");
        
        let instance = entry.create_instance(&createInfo, None).expect("Failed to create Instance!");
        
        println!("Created Inst.");

        let exts = entry.enumerate_instance_extension_properties(None)
            .expect("Failed to get Extention data.");
        for ext in exts {
            println!("{}", std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()).to_str().unwrap());
        }

        // create devices

        let physical_devices = instance.enumerate_physical_devices().expect("Failed to get physical devices");

        if(physical_devices.len() == 0){
            panic!("Couldn't find any gpus with Vulkan support!! Try OpenGL instead!!");
        }
        let mut physical_device: Option<vk::PhysicalDevice> = None;
        let mut device_candidates: Vec<(u32, vk::PhysicalDevice)> = Vec::<(u32, vk::PhysicalDevice)>::new();
        for p_physical_device in &physical_devices {
            let score = rate_physical_device(p_physical_device, &instance);
            device_candidates.push((score, *p_physical_device));
        }
        let candidate = device_candidates.first().unwrap();
        if(candidate.0 > 0){
            physical_device = Some(candidate.1);
        }
        else{
            println!("{}", candidate.0);
            panic!("Failed to find suitible GPU!");
        }

        
    }


}
#[cfg(feature = "vulkan")]
pub trait VulkanRender {
    fn init(&self) -> i32;
    fn render(&self) -> i32;
}
#[cfg(feature = "opengl")]
pub trait OGLRender {
    fn init(&self) -> i32;
    fn render(&self) -> i32;
}
#[cfg(feature = "gles")]
pub trait GLESRender {
    fn init(&self) -> i32;
    fn render(&self) -> i32;
}

#[cfg(feature = "vulkan")]
impl VulkanRender for Pipeline {
    fn init(&self) -> i32 {
        0
    }

    fn render(&self) -> i32 {
        // Rendering per mesh
        // First generate all shaders and vertex buffers
        for m in &self.meshs {
            let mut mesh = m.lock().unwrap();
            // first the shader, we must compile and generate a pipeline
            let shader_modules = mesh.material.bake(self.device.clone());
            
            // Get vertex, index, texcoord buffers
            
        }
        0
    }
}
