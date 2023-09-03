#![feature(mutex_unlock)]
#![allow(unused)]
use std::any::Any;
use raw_window_handle::HasRawWindowHandle;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, atomic::*};
use parking_lot::*;
use std::sync::atomic::AtomicI32;
use std::future::*;
use sdl2::*;
use async_trait::*;
use winit::dpi::Pixel;


extern crate raw_window_handle;

use crate::common::materials::*;
use crate::common::matrices::*;
use crate::common::mesh::*;
use crate::common::engine::*;
use crate::common::vertex::*;

use super::gamesys::*;
use super::threading::*;

#[cfg(feature = "vulkan")] use super::vulkan::*;
#[cfg(feature = "opengl")] use super::opengl::*;
#[cfg(feature = "gles")] use super::gles::*;

pub struct Pipeline {
    pub id: i32,
    pub name: String,
    pub meshs: Vec<Arc<Mutex<Mesh>>>,
    pub cameras: Vec<Arc<Mutex<Camera>>>,
    pub layer: u32,
    pub driver: Arc<Mutex<Option<DriverValues>>>,
    pub is_init: bool
}

#[derive(Clone)]
pub struct PipelineParams {
    pub name: String,
    pub layer: u32,
}

unsafe impl Send for Pipeline {}

impl Pipeline {
    pub fn register_mesh(&mut self, p_mesh: Arc<Mutex<Mesh>>){
        self.meshs.push(p_mesh.clone());
    }

    pub fn register_camera(&mut self, p_camera: Arc<Mutex<Camera>>){
        self.cameras.push(p_camera.clone());
    }
}

#[derive(Clone)]
pub struct Camera {
    pub cam_id: i32,
    pub projection: MatrixProjection,
    pub transform: Matrix34,
    pub render_texture: Option<RenderTexture>,
    is_active: bool,
}

impl Camera {
    
    pub fn new(id: i32) -> Self {
        Self { cam_id: id, projection: MatrixProjection::new(), transform: Matrix34::identity(), render_texture: None, is_active: false }
    }

}

// TODO: Add mesh reference vector so that we don't reference meshes only in the pipeline
// (Saves on memory and don't want to store multiple instances of meshes)
pub struct RenderPipelineSystem {
    pub pipelines: Vec<Arc<Mutex<Pipeline>>>,
    counter: AtomicI32,
    system_status: Arc<Mutex<gamesys::StatusCode>>,
    thread_count: usize,
    threads: Dict<usize, Arc<Mutex<Threader>>>,
    thread_reciever: Arc<Mutex<Vec<ThreadData>>>,
    driver_vals: Arc<Mutex<Option<DriverValues>>>,
    cameras: Vec<Arc<Mutex<Camera>>>,
    active_camera: i32

}

impl RenderPipelineSystem{
    // TODO: Fix these so that it doesn't borrow self!!
    pub unsafe fn resgister_pipeline(params: PipelineParams) -> i32{
        let p_this = Game::get_render_sys().clone();
        let mut this = p_this.write();
        let id = this.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let p = Arc::new(Mutex::new(Pipeline {
            id: id.clone(),
            name: params.name.clone(),
            meshs: Vec::new(),
            layer: params.layer,
            driver: this.driver_vals.clone(),
            cameras: Vec::new(),
            is_init: false
        }));
        
        this.pipelines.push(p);
        id
    }

    pub unsafe fn register_mesh(&mut self, layer: u32, mesh: Arc<Mutex<Mesh>>){
        
        let p_pipelines = self.pipelines.clone();
        for p in &p_pipelines {
            let mut pipeline = p.lock();
            if(pipeline.layer == layer){
                let m = mesh.clone();
                pipeline.register_mesh(m);
                
            }
        }
        
    }

    pub unsafe fn register_camera(&mut self, layer: u32) -> i32 {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let p_cam = Arc::new(Mutex::new(Camera::new(id.clone())));

        let pos = self.cameras.len();

        self.cameras.push(p_cam.clone());

        for p in &self.pipelines {
            let mut pipeline = p.lock();
            if pipeline.layer == layer {
                pipeline.cameras.push(p_cam.clone());
            }
        }

        id

    }

    pub fn update_camera(&mut self, id: i32, projection: &MatrixProjection, transform: &Matrix34) {
        let p_cam = self.cameras.iter().find(|v| {let vv = v.lock(); vv.cam_id == id}).clone().expect("No such registered camera!!");
        let mut cam = p_cam.lock();
        cam.projection = projection.clone();
        cam.transform = transform.clone();
        
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

    pub fn new() -> RenderPipelineSystem {
        let pip_sys = RenderPipelineSystem {
            pipelines: Vec::new(),
            counter: AtomicI32::new(1),
            system_status: Arc::new(Mutex::new(gamesys::StatusCode::RUNNING)),
            thread_count: 0,
            threads: Dict::<usize, Arc<Mutex<Threader>>>::new(),
            thread_reciever: Arc::new(Mutex::new(Vec::new())),
            driver_vals: crate::common::components::component_system::ComponentRef_new(Some(DriverValues::default())),
            cameras: Vec::new(),
            active_camera: 0
        };
        return pip_sys;
    }

    pub fn processing<'a>(p_this: Arc<RwLock<Self>>) -> i32{
        unsafe{
            let this = p_this.write();
            // Initialise pipeline stuff
            #[cfg(feature = "vulkan")]DriverValues::init_vulkan(this.driver_vals.lock().as_mut().unwrap());
            #[cfg(feature = "opengl")]DriverValues::init_ogl(this.driver_vals.lock().as_mut().unwrap());
            #[cfg(feature = "gles")]DriverValues::init_gles(this);
             // prebake step (only for when we have a better file system)
             // Create a new thread purely for baking and wait for this to finish
             // TODO: Implement better file system for storing shaders


            // first we setup threads for each layer
            //let mut threads: Box<Dict<usize, Arc<Mutex<Threader>>>> = Box::new(Dict::<usize, Arc<Mutex<Threader>>>::new());
            
            drop(this);
            while !Game::isExit() {
                let this = match p_this.try_read() {
                    Some(v) => v,
                    None => {
                        
                        continue;
                    }
                };
                let p_recv = this.thread_reciever.clone();
                let p_pipelines = this.pipelines.clone();
                let system_status = this.system_status.clone();
                drop(this);
                let mut recv = p_recv.try_lock();
                if let Some(ref mut mutex) = recv {
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
                                let mut sys_status = system_status.lock();
                                *sys_status = status;
                            },
                            ThreadData::QuickDraw(v, f, tx, d) => {

                            },
                            ThreadData::Mesh(layer, mesh) => {
                                for p in &p_pipelines {
                                    let mut pipeline = match p.try_lock() {
                                        Some(v) => v,
                                        None => continue
                                    };
                                    if(pipeline.layer == layer){
                                        let m = mesh.clone();
                                        pipeline.register_mesh(m);
                                        
                                    }
                                }
                            }
                            _ => {},
                        }
                    }
                    mutex.clear();
                }

                for p in p_pipelines{
                    let pipe = p.lock();
                    let is_init = pipe.is_init.clone();
                    drop(pipe);
                    if !is_init {
                        #[cfg(feature = "vulkan")]VulkanRender::init(p.clone());
                        #[cfg(feature = "opengl")]OGlRender::init(p.clone());
                        #[cfg(feature = "gles")]GLESRender::init(p.clone());
                    }
                    
                    #[cfg(feature = "vulkan")]VulkanRender::render(p.clone());
                    #[cfg(feature = "opengl")]OGlRender::render(p.clone());
                    #[cfg(feature = "gles")]GLESRender::render(p.clone());
                }
                
                // std::thread::sleep(std::time::Duration::from_millis(5));
                
            }
            println!("Closing thread!!");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        0
    }

    pub fn is_alive(this: &mut Self) -> bool {
        let p_sys_status = this.system_status.clone();
        let sys_status = p_sys_status.lock();
        *sys_status != StatusCode::CLOSE
    }

    pub fn send_status(p_this: Arc<RwLock<Self>>, status: StatusCode) {
        loop {
            let this = match p_this.try_read_for(std::time::Duration::from_millis(1)){
                Some(v) => v,
                None => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    continue;
                }
            };
            let p_stat = this.system_status.clone();
            let mut stat = p_stat.lock();
            stat = stat;
            break;
        }
    }

    pub fn init<'a>(this: Arc<RwLock<Self>>){
        // Start thread
        println!("Spawned Render Pipeline System");
        Self::processing(this.clone());
    }

    //region Vulkan Render 


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
                Some(re) => re,
                None => continue 'test
            };
            recv.push(ThreadData::QuickDraw(vertices.clone(), faces.clone(), tex_coord.clone(), data.clone()));
            drop(recv);
            break;
        }
    }

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


