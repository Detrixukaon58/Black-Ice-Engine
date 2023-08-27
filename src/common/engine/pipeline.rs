#![feature(mutex_unlock)]
#![allow(unused)]
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


extern crate raw_window_handle;

use crate::common::materials::*;
use crate::common::matrices::*;
use crate::common::mesh::*;
use crate::common::engine::*;
use crate::common::vertex::*;

use super::gamesys::*;
use super::threading::*;

#[cfg(feature = "vulkan")] use super::vulkan::*;
#[cfg(feature = "opengl")] use ogl33::*;
#[cfg(feature = "gles")] use opengles::*;

pub struct Pipeline {
    id: i32,
    pub name: String,
    pub meshs: Vec<Arc<Mutex<Mesh>>>,
    pub layer: usize,
    pub driver: Arc<Mutex<Option<DriverValues>>>,
}

#[derive(Clone)]
pub struct PipelineParams {
    pub name: String,
    pub layer: usize,
}

unsafe impl Send for Pipeline {}

impl Pipeline {
    pub fn register_mesh(&mut self, p_mesh: Arc<Mutex<Mesh>>){
        self.meshs.push(p_mesh);
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
            driver: this.driver_vals.clone(),
        };
        
        if(this.thread_count < params.layer){
            this.thread_count = params.layer;
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
        let pip_sys = RenderPipelineSystem {
            pipelines: Vec::new(),
            counter: AtomicI32::new(1),
            system_status: Arc::new(Mutex::new(gamesys::StatusCode::RUNNING)),
            thread_count: 0,
            threads: Dict::<usize, Arc<Mutex<Threader>>>::new(),
            thread_reciever: Arc::new(Mutex::new(Vec::new())),
            driver_vals: crate::common::components::component_system::ComponentRef_new(Some(DriverValues::default())),
        };
        return pip_sys;
    }

    pub fn processing<'a>(p_this: Arc<Mutex<Self>>) -> i32{
        unsafe{
            let this = p_this.lock().unwrap();
            // Initialise pipeline stuff
            #[cfg(feature = "vulkan")]DriverValues::init_vulkan(this.driver_vals.lock().unwrap().as_mut().unwrap());
            #[cfg(feature = "opengl")]RenderPipelineSystem::init_ogl(this);
            #[cfg(feature = "gles")]RenderPipelineSystem::init_gles(this);
             // prebake step (only for when we have a better file system)
             // Create a new thread purely for baking and wait for this to finish
             // TODO: Implement better file system for storing shaders


            // first we setup threads for each layer
            //let mut threads: Box<Dict<usize, Arc<Mutex<Threader>>>> = Box::new(Dict::<usize, Arc<Mutex<Threader>>>::new());

            
            drop(this);
            while !Game::isExit() {
                let this = p_this.lock().unwrap();
                let p_recv = this.thread_reciever.clone();
                let mut recv = p_recv.try_lock();
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
                drop(this);
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
                Ok(re) => re,
                Err(err) => continue 'test
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


