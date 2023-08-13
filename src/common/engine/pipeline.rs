#![feature(mutex_unlock)]
use std::any::Any;
use raw_window_handle::HasRawWindowHandle;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::*;
use std::sync::atomic::AtomicI32;
use std::future::*;
use sdl2::*;
use async_trait::*;
use winit::dpi::Pixel;
use vk::Handle;

extern crate raw_window_handle;

use crate::common::materials::BakeVulkan;
use crate::common::matrices::*;
use crate::common::mesh::*;
use crate::common::engine::*;
use crate::common::vertex::*;

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

#[derive(Clone)]
pub struct PipelineParams {
    pub name: String,
    pub layer: usize,
}

unsafe impl Send for Pipeline {}

impl Pipeline {
    pub fn register_mesh(&mut self, mesh: Arc<Mutex<Mesh>>){
        self.meshs.push(mesh);
    }
}

// TODO: Add mesh reference vector so that we don't reference meshes only in the pipeline
// (Saves on memory and don't want to store multiple instances of meshes)
pub struct RenderPipelineSystem {
    pub pipelines: Vec<Arc<Mutex<Pipeline>>>,
    counter: AtomicI32,
    system_status: Arc<Mutex<gamesys::StatusCode>>,
    threadCount: usize,
    threads: Dict<usize, Arc<Mutex<Threader>>>,
    thread_reciever: Arc<Mutex<Vec<ThreadData>>>,
    driver_vals: Option<DriverValues>,

}

#[derive(Clone, Default)]
struct QueueFamiltyIdices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
    
}

impl QueueFamiltyIdices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

impl RenderPipelineSystem{
    // TODO: Fix these so that it doesn't borrow self!!
    pub unsafe fn resgister_pipeline(params: PipelineParams){
        let p_this = Game::get_render_sys().clone();
        let mut this = p_this.lock().unwrap();
        let pipeline = Pipeline {
            id: this.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            name: params.name.clone(),
            meshs: Vec::new(),
            layer: params.layer,
            device: Some(this.driver_vals.as_ref().unwrap().physical_devices[0])
        };
        if(this.threadCount < params.layer){
            this.threadCount = params.layer;
        }
        this.pipelines.push(Arc::new(Mutex::new(pipeline)));
        
    }

    pub unsafe fn register_mesh(id: i32, mesh: Arc<Mutex<Mesh>>){
        let p_this = Game::get_render_sys().clone();
        let mut this = p_this.lock().unwrap();
        for p in &mut *this.pipelines {
            let mut pipeline = p.lock().unwrap();
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
            system_status: Arc::new(Mutex::new(gamesys::StatusCode::RUNNING)),
            threadCount: 0,
            threads: Dict::<usize, Arc<Mutex<Threader>>>::new(),
            thread_reciever: Arc::new(Mutex::new(Vec::new())),
            driver_vals: Some(DriverValues::default()),
        };
        return pipSys;
    }

    pub fn processing<'a>(this: &'a mut Self) -> i32{
        unsafe{
            
            // Initialise pipeline stuff
            #[cfg(feature = "vulkan")]RenderPipelineSystem::init_vulkan(this);
            #[cfg(feature = "opengl")]RenderPipelineSystem::init_ogl(this);
            #[cfg(feature = "gles")]RenderPipelineSystem::init_gles(this);
             // prebake step (only for when we have a better file system)
             // Create a new thread purely for baking and wait for this to finish
             // TODO: Implement better file system for storing shaders


            // first we setup threads for each layer
            //let mut threads: Box<Dict<usize, Arc<Mutex<Threader>>>> = Box::new(Dict::<usize, Arc<Mutex<Threader>>>::new());

            
            
            while !Game::isExit() {
                let mut recv = this.thread_reciever.try_lock();
                if let Ok(ref mut mutex) = recv {
                    for th in mutex.as_slice() {
                        let data = th.clone();
                        match data {
                            ThreadData::Empty => todo!(),
                            ThreadData::I32(i) => todo!(),
                            ThreadData::String(s) => todo!(),
                            ThreadData::Vec(vec) => todo!(),
                            ThreadData::Vec3(vec3) => todo!(),
                            ThreadData::Quat(quat) => todo!(),
                            ThreadData::Status(status) => {
                                let mut sys_status = this.system_status.lock().unwrap();
                                *sys_status = status;
                            },
                            ThreadData::QuickDraw(v, f, tx, d) => {

                            }
                            _ => {},
                        }
                    }
                    mutex.clear();
                }

                for p in &this.pipelines{
                    let pipeline = p.lock().unwrap();

                    // let mut a = this.threads.get_or_insert(pipeline.layer, Arc::new(Mutex::new(Threader::new()))).unwrap();
                    // let mut b = a.lock().unwrap();
                    // if(b.isAlive()){
                    //     b.stop();
                    // }
                    // b.start(|| {
                        #[cfg(feature = "vulkan")]VulkanRender::render(p.clone());
                    // });
                }

                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            println!("Closing thread!!");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        0
    }

    pub fn is_alive(this: &mut Self) -> bool {
        let p_sys_status = this.system_status.clone();
        let sys_status = p_sys_status.lock().unwrap();
        *sys_status != StatusCode::CLOSE
    }

    pub fn send_status(p_this: Arc<Mutex<Self>>, status: StatusCode) {
        let this = p_this.lock().unwrap();
        let p_stat = this.system_status.clone();
        let mut stat = p_stat.lock().unwrap();
        stat = stat;
    }

    pub fn init<'a>(this: Arc<Mutex<Self>>){
        // Start thread
        println!("Spawned Render Pipeline System");
        Self::processing(&mut this.lock().unwrap());
    }

    //region Vulkan Render 

    #[cfg(feature = "vulkan")]
    unsafe fn init_vulkan(this:&mut Self)  {
        use winit::dpi::Pixel;
        
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
        // for ext in extension_names.to_vec() {
        //     println!("{}", ext);
        // }
        let layer_names = [std::ffi::CStr::from_bytes_with_nul_unchecked(
            b"VK_LAYER_KHRONOS_validation\0",
        )];
        let layers_names_raw: Vec<*const std::os::raw::c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();
        let ext1 = extension_names.into_iter().map(|s| s.as_ptr().cast::<i8>()).collect::<Vec<*const i8>>();
        let create_flags = vk::InstanceCreateFlags::default();
        let mut createInfo = vk::InstanceCreateInfo::builder()
            .enabled_extension_names(ext1.as_slice());
        createInfo.p_application_info = &appInfo;
        
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
        
        let driver = this.driver_vals.as_mut().unwrap();

        driver.physical_devices = physical_devices;
        driver.instance = Some(instance);
        driver.entry = Some(entry);

        

        // Change to this if all else fails!!
        // let handle = driver.instance.as_ref().unwrap().handle().as_raw();

        // let surface_khr = GAME.window.vulkan_create_surface(handle as usize).expect("failed to create surface");

        // driver.surface = Some(vk::SurfaceKHR::from_raw(surface_khr));

        driver.surface = RenderPipelineSystem::get_surface(driver);
        
        let mut indices = RenderPipelineSystem::find_queue_families(driver, 0);

        let mut queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.graphics_family.unwrap())
            .build();
        queue_create_info.s_type = vk::StructureType::DEVICE_QUEUE_CREATE_INFO;
        queue_create_info.queue_count = 1;

        let queue_priority = 1.0;
        queue_create_info.p_queue_priorities = &queue_priority;

        let device_features = vk::PhysicalDeviceFeatures::default();

        let mut device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&[queue_create_info])
            .enabled_features(&device_features)
            .build();

        device_info.s_type = vk::StructureType::DEVICE_CREATE_INFO;
        device_info.queue_create_info_count = 1;
        device_info.enabled_extension_count = 0;

        let mut logical_device = driver.instance.as_ref().unwrap().create_device(driver.physical_devices[0], &device_info, None).expect("failed to create logical device!");

        driver.logical_devices.push(Some(logical_device));

        let queue = driver.logical_devices[0].as_ref().unwrap().get_device_queue( indices.graphics_family.unwrap(), 0);


        let mut vt_input = vk::VertexInputBindingDescription::default();
        vt_input.input_rate = vk::VertexInputRate::VERTEX;
        vt_input.stride = std::mem::size_of::<[f32; 3]>() as u32;
        vt_input.binding = 0;
        
    }

    #[cfg(feature = "vulkan")]
    unsafe fn get_best_device(driver:&mut DriverValues) -> usize {
        if(driver.physical_devices.len() == 0){
            panic!("Couldn't find any gpus with Vulkan support!! Try OpenGL instead!!");
        }
        let mut device_candidates: Vec<(u32, usize)> = Vec::<(u32, usize)>::new();
        let mut i = 0;
        for p_physical_device in &driver.physical_devices {
            let score = RenderPipelineSystem::rate_physical_device(p_physical_device, driver.instance.as_ref().unwrap());
            device_candidates.push((score, i));
            i += 1;
        }
        for candidate in device_candidates {
            if candidate.0 > 0 {
                if RenderPipelineSystem::checkDeviceSuitability(driver, candidate.1) {
                    return candidate.1;
                }
            }
        }

        0
    }

    #[cfg(feature = "vulkan")]
    unsafe fn checkDeviceSuitability(driver:&mut DriverValues, device: usize) -> bool{
        let indices = RenderPipelineSystem::find_queue_families(driver, device);

        return indices.is_complete();
    }

    #[cfg(feature = "vulkan")]
    unsafe fn find_queue_families(driver:&mut DriverValues, device: usize) -> QueueFamiltyIdices{

        let mut indices: QueueFamiltyIdices = QueueFamiltyIdices::default();
        let mut queue_fams = driver.instance.as_ref().unwrap().get_physical_device_queue_family_properties(driver.physical_devices[device]);
        let mut surface_loader = extensions::khr::Surface::new(driver.entry.as_ref().unwrap(), driver.instance.as_ref().unwrap());
        

        let mut i: u32 = 0;
        for queue_family in queue_fams {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i);
                
            }

            let present_support = surface_loader.get_physical_device_surface_support(driver.physical_devices[device], i, driver.surface.unwrap()).expect("Failed to check surface support!!");
            if present_support {
                indices.present_family = Some(i);
            }
            

            if indices.is_complete() {
                break;
            }
            i +=1;
        }

        indices
    }

    #[cfg(feature = "vulkan")]
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

    #[cfg(feature = "vulkan")]
    pub unsafe fn get_surface(driver: &mut DriverValues) -> Option<vk::SurfaceKHR>{

        

        #[cfg(target_os = "windows")]
        unsafe fn get_surface_a(driver: &mut DriverValues) -> Option<vk::SurfaceKHR> {
            let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
            let bb  = SDL_GetWindowWMInfo(GAME.window.raw(), &mut window_info);

            let display_handle = raw_window_handle::WindowsDisplayHandle::empty();

            let mut window_handle = raw_window_handle::Win32WindowHandle::empty();
            window_handle.hinstance = window_info.info.win.hinstance.cast();
            window_handle.hwnd = window_info.info.win.window.cast();

            let surface = ash_window::create_surface(driver.entry.as_ref().unwrap(), 
                driver.instance.as_ref().unwrap(), 
                raw_window_handle::RawDisplayHandle::Windows(display_handle), 
                raw_window_handle::RawWindowHandle::Win32(window_handle), None)
                .expect("Failed to create surface!!");

            Some(surface)
        }
        


        #[cfg(target_os = "linux")]
        unsafe fn get_surface_a(driver: &mut DriverValues) -> Option<vk::SurfaceKHR> {
            // Assume wayland!!
            let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
            SDL_GetWindowWMInfo(GAME.window.raw(), &mut window_info);

            let mut display_handle = raw_window_handle::WaylandDisplayHandle::empty();

            display_handle.display = window_info.info.wl.display;

            let mut window_handle = raw_window_handle::WaylandWindowHandle::empty();

            window_handle.surface = window_info.info.wl.surface;

            let surface = ash_window::create_surface(driver.entry.as_ref().unwrap(), 
                driver.instance.as_ref().unwrap(), 
                raw_window_handle::RawDisplayHandle::Wayland(display_handle), 
                raw_window_handle::RawWindowHandle::Wayland(window_handle), None)
                .expect("Failed to create surface!!");
            Some(surface)
        }

        #[cfg(target_os = "macos")]
        unsafe fn get_surface_a(driver: &mut DriverValues) -> Option<vk::SurfaceKHR> {

            let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
            SDL_GetWindowWMInfo(GAME.window.raw(), &mut window_info);

            let mut display_handle = raw_window_handle::AppKitDisplayHandle::empty();

            

            let mut window_handle = raw_window_handle::AppKitWindowHandle::empty();

            window_handle.ns_window = window_info.info.cocoa.window;

            let surface = ash_window::create_surface(driver.entry.as_ref().unwrap(), 
                driver.instance.as_ref().unwrap(), 
                raw_window_handle::RawDisplayHandle::AppKit(display_handle), 
                raw_window_handle::RawWindowHandle::AppKit(window_handle), None)
                .expect("Failed to create surface!!");
            Some(surface)

        }

        get_surface_a(driver)

    }

    //endregion

    //region OpenGL


    //endregion

    //region GLES


    //endregion


    pub fn quick_redner<'a>(&mut self, vertices: Vec<Vertex>, faces: Vec<(i32, i32, i32)>, tex_coord: Vec<[f32; 2]>, image: std::iter::Enumerate<imagine::png::PngRawChunkIter<'a>>){
        
        let mut pallete = Vec::<Vec4>::new();
        let mut data = Vec::<Vec4>::new();
        
        for (n, raw_chunk) in image {
            let chunk_res = imagine::png::PngChunk::try_from(raw_chunk).unwrap();
            match chunk_res {
                imagine::png::PngChunk::sRGB(srgb) => {
                    
                },
                imagine::png::PngChunk::PLTE(plte) => {
                    for d in plte.entries() {
                        pallete.push(Vec4::new(d[0].cast(), d[1].cast(), d[2].cast(), 0));
                    }
                },
                imagine::png::PngChunk::tRNS(trns) => {
                    for (a, c) in trns.to_alphas().iter().zip(pallete.iter_mut()) {
                        c.w = a.cast();
                    }
                },
                imagine::png::PngChunk::IDAT(idat) => {
                    let data_string = format!("{:?}", idat);
                    for b in data_string.as_bytes() {
                        data.push(pallete[*b as usize]);
                    }
                },
                _ => continue
            }
        }
        'test: loop{
            let p_recv = self.thread_reciever.clone();
            let mut recv = match p_recv.try_lock() {
                Ok(re) => re,
                Err(err) => continue 'test
            };
            recv.push(ThreadData::QuickDraw(vertices.clone(), faces.clone(), tex_coord.clone(), data.clone()));
            drop(recv);
            break;
        }
    }

}
extern "C" {
    fn SDL_GetWindowWMInfo(window: *mut sdl2::sys::SDL_Window, info: *mut sdl2::raw_window_handle::SDL_SysWMinfo) -> sdl2::sys::SDL_bool;
}

#[cfg(feature = "vulkan")]
pub trait VulkanRender {
    fn init(&self) -> i32;
    fn render(th: Arc<Mutex<Self>>) -> i32;
}

#[cfg(feature = "vulkan")]
#[derive(Default, Clone)]
pub struct DriverValues {
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub physical_devices: Vec<vk::PhysicalDevice>,
    pub logical_devices : Vec<Option<Device>>,
    pub surface : Option<vk::SurfaceKHR>,
    pub enable_validation_layers: bool,

}

#[cfg(feature = "opengl")]
pub trait OGLRender {
    fn init(&self) -> i32;
    fn render(th: Arc<Mutex<Self>>) -> i32;
}

#[cfg(feature = "opengl")]
#[derive(Default, Clone)]
pub struct DriverValues {}

#[cfg(feature = "gles")]
pub trait GLESRender {
    fn init(&self) -> i32;
    fn render(th: Arc<Mutex<Self>>) -> i32;
}

#[cfg(feature = "gles")]
#[derive(Default, Clone, Clone)]
pub struct DriverValues {}

#[cfg(feature = "vulkan")]
impl VulkanRender for Pipeline {
    fn init(&self) -> i32 {
        0
    }

    fn render(th: Arc<Mutex<Self>>) -> i32 {
        let this = th.lock().unwrap();
        // Rendering per mesh
        // First generate all shaders and vertex buffers
        for m in &this.meshs {
            let mut mesh = m.lock().unwrap();
            // first the shader, we must compile and generate a pipeline
            let shader_modules = mesh.material.bake(this.device.clone());
            
            // Get vertex, index, texcoord buffers
            
        }
        0
    }
}

