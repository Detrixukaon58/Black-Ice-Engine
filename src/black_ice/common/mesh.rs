use std::borrow::BorrowMut;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use engine::asset_mgr::AssetManager;
use parking_lot::*;
use crate::black_ice::common::{vertex::*, transform::*, engine::gamesys::*};
use crate::black_ice::common::filesystem::files::*;
use crate::black_ice::common::{engine::asset_types::materials::*, *};

use super::components::component_system::ConstructorDefinition;

// TODO: Add layer reference so that correct pipelines can get the correct meshes
/// Type of resource

#[derive(PartialEq)]
pub enum SurfaceType {
    TRIANGLES,
    LINES,
    POINTS
}

pub struct Surface {
    pub id: u32,
    pub name: String,
    pub verts: Vec<Vec3>,
    pub indices: Vec<i16>, //made of 3 verts
    pub normals: Vec<(i16, Vec3)>,
    pub texture_coord: Vec<(i16, (f32, f32))>,
    pub is_concave: bool,
    pub surface_type: SurfaceType
}

pub struct Mesh {
    pub surfaces: Vec<Arc<Mutex<Surface>>>,
    pub transform: matrices::Matrix34,
    pub materials: HashMap<u32, Arc<Mutex<Material>>>,
    counter: AtomicU32
}

impl Mesh {

    pub fn triangles(&mut self) {
        let mut mesh_object = Surface::new("triangle".to_string(), SurfaceType::TRIANGLES);
        
        mesh_object.add_point(Vec3::new(-25.0, -25.0, 0.0));
        mesh_object.add_point(Vec3::new(25.0, 25.0, 0.0));
        mesh_object.add_point(Vec3::new(50.0, 0.0, 0.0));

        mesh_object.add_face(0, 1, 2);
        mesh_object.add_normal(0, Vec3::new(0.0, 0.0, 1.0));
        mesh_object.add_normal(1, Vec3::new(0.0, 0.0, 1.0));
        mesh_object.add_normal(2, Vec3::new(0.0, 0.0, 1.0));

        mesh_object.add_uv(0, (0.0, 0.0));
        mesh_object.add_uv(1, (0.5, 0.5));
        mesh_object.add_uv(2, (1.0, 0.0));
        mesh_object.id = self.counter.fetch_add(1, std::sync::atomic::Ordering::Acquire);
        self.materials.insert(mesh_object.id.clone(), Arc::new(Mutex::new(Material::new())));
        self.surfaces.push(Arc::new(Mutex::new(mesh_object)));

        
    }

    pub fn square(&mut self) {
        let mut mesh_object = Surface::new("square".to_string(), SurfaceType::TRIANGLES);
        let v = 5.0;
        mesh_object.add_point(Vec3::new(-v, -v, 0.0));
        mesh_object.add_point(Vec3::new(v, -v, 0.0));
        mesh_object.add_point(Vec3::new(v, v, 0.0));

        mesh_object.add_face(0, 1, 2);
        mesh_object.add_normal(0, Vec3::new(0.0, 0.0, 1.0));
        mesh_object.add_normal(1, Vec3::new(0.0, 0.0, 1.0));
        mesh_object.add_normal(2, Vec3::new(0.0, 0.0, 1.0));

        mesh_object.add_point(Vec3::new(-v, v, 0.0));

        mesh_object.add_face(2, 3, 0);
        mesh_object.add_normal(3, Vec3::new(0.0, 0.0, 1.0));

        mesh_object.add_uv(0, (0.0, 0.0));
        mesh_object.add_uv(1, (1.0, 0.0));
        mesh_object.add_uv(2, (1.0, 1.0));
        mesh_object.add_uv(3, (0.0, 1.0));
        mesh_object.id = self.counter.fetch_add(1, std::sync::atomic::Ordering::Acquire);
        self.materials.insert(mesh_object.id.clone(), Arc::new(Mutex::new(Material::new())));
        self.surfaces.push(Arc::new(Mutex::new(mesh_object)));
    }

    pub fn new() -> Self {
        Self { surfaces: Vec::new(), transform: matrices::Matrix34::identity(), materials: HashMap::new(),counter: AtomicU32::new(0)}
    }
}

pub struct MeshFile {
    id_counter: AtomicU32,
    surfaces: Vec<Arc<Mutex<Surface>>>,
    pub mesh_file: FileSys,
    pub mesh_file_type: MFType,
    pub use_custom_materials: bool,
    pub materials: HashMap<u32, Arc<Mutex<Material>>>,
}

pub trait MeshInstanciate<T> {
    fn new() -> T;
}

pub trait MeshRender {
    fn render(&self, t: Transform);
    fn get_vert(&self, i: i16) -> Vec3;
    fn get_verts(&self) -> Vec<Vec3>;
    fn get_edges(&self) -> Vec<i16>;
    fn get_edge(&self, i: i16) -> (i16, i16);
    fn get_edges_of_vert(&self, i: i16) -> Vec<(i16, i16)>;
    fn get_face(&self, i: i16) -> (i16, i16, i16);
    fn get_faces(&self, i: i16) -> Vec<i16>;
    fn get_verts_of_face(&self, i: i16) -> Vec<i16>;

    fn translated_verts(&self, t: Transform) -> Vec<Vec3>;
    fn translated_faces(&self, t: Transform) -> Vec<(i16, i16, i16)>;
    fn translated_edges(&self, t: Transform) -> Vec<(i16, i16, i16)>;

}

pub trait MeshFileSys {
    fn open(&mut self, f: &str);
    fn open_fbx(&mut self);
    fn open_obj(&mut self);
}

pub trait MeshConstruct {
    fn add_points(&mut self, verts: Vec<Vec3>);
    fn add_point(&mut self, vert: Vec3) -> i16;
    fn add_face(&mut self, vert1: i16, vert2: i16, vert3: i16);
    fn add_edge(&mut self, vert1: i16, vert2: i16);
    fn add_normal(&mut self, index: i16, normal: Vec3);
    fn add_uv(&mut self, index: i16, coord: (f32, f32));
}

//region Mesh Reflection
impl Base for Surface{}
impl Base for MeshFile{}

impl Reflection for Surface{
    fn register_reflect(&'static self) -> Ptr<Register> 
    {
        let mut register = Box::new(Register::new(Box::new(self)));
        
        register.addProp(Property { 
            name: Box::new("faces"),
            desc: Box::new("The Faces of the object"), 
            reference: Box::new(&self.indices), 
            ref_type: self.indices.type_id()
        });
        register.addProp(Property { 
            name: Box::new("verts"), 
            desc: Box::new("The Vertices of the object"), 
            reference: Box::new(&self.verts), 
            ref_type: self.verts.type_id()});
        register.addProp(Property { 
            name: Box::new("normals"), 
            desc: Box::new("The normals of each face. This is ordered in the order of the faces. e.g. faces[1] has normal normals[1], faces[n] has normal normal[n]"), 
            reference: Box::new(&self.normals), 
            ref_type: self.normals.type_id()});

        return Ptr{ b: register};
    }
}

impl Base for Vec<Mesh>{}

impl Reflection for MeshFile {
    fn register_reflect(&'static self) -> Ptr<Register>{
        let mut _register = Box::new(Register::new(Box::new(self)));

        //register.addPointer(Pointer {name: Box::new("objects"), desc: Box::new("The objects that make up a mesh"), reference: self.object.registerReflect(), refType: self.object.type_id()});
        //register.addProp(Property {name: Box::new("objects"), desc: Box::new("The Objects that make up a mesh file."), reference: Box::new(&self.objects), refType: self.objects.type_id()});

        return Ptr{b: _register};
    }
}
//endregion

impl MeshInstanciate<MeshFile> for MeshFile {
    fn new() -> MeshFile {
        return MeshFile {id_counter: AtomicU32::new(0), surfaces: Vec::<Arc<Mutex<Surface>>>::new(), mesh_file: FileSys::new(), mesh_file_type: MFType::UNKNOWN, use_custom_materials: false, materials: HashMap::new()};
    }
}

impl MeshFileSys for MeshFile {
    
    fn open(&mut self, f: &str){
        self.mesh_file.open(f);
        self.mesh_file_type = self.mesh_file.check_file_ext();
        
        match self.mesh_file_type {
            MFType::FBX=>self.open_fbx(),
            MFType::OBJ=>self.open_obj(),
            _=>{

            }
        }

    }
    fn open_fbx(&mut self){

    }
    fn open_obj(&mut self){
        let buffer = self.mesh_file.read();
        let mut _line_count = 0;
        let mut _material_files: Vec<FileSys> = Vec::<FileSys>::new();
        let mut texture_coords: Vec<(f32, f32)> = Vec::<(f32,f32)>::new();
        let mut normals: Vec<Vec3> = Vec::<Vec3>::new();
        //let mut vertices: Vec<Vec3> = Vec::<Vec3>::new();
        let mut current_object = 0;
        
        

        for i in 1..(get_number_of_lines(&buffer) - 1){

            let line = get_line(&buffer, i);
            let (line_type, split) = check_line(line);
            let p_current = self.surfaces[current_object].clone();
            let mut obj = p_current.lock();
            match line_type{
                Lntp::VERTEX =>         {
                    let vertex = Vec3::new(split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap(), split[3].parse::<f32>().unwrap());
                    obj.add_point(vertex);
                },
                Lntp::VERTEX_TEXTURE => {texture_coords.push((split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap()));},
                Lntp::VERTEX_NORMAL => {normals.push(Vec3::new(split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap(), split[3].parse::<f32>().unwrap()))},
                Lntp::FACE => {
                    let face_vertices = &split[1..];
                    if face_vertices.len() > 3 {
                        // Dealing with a non triangular face

                    }
                    else{
                        let mut a = face_vertices[0].split('/');
                        let mut b = face_vertices[1].split('/');
                        let mut c = face_vertices[2].split('/');
                        obj.add_face(a.nth(0).unwrap().parse::<i16>().unwrap(), b.nth(0).unwrap().parse::<i16>().unwrap(), c.nth(0).unwrap().parse::<i16>().unwrap());
                    }
                },
                Lntp::MTLLIB => {},
                Lntp::OBJECT_NAME => {
                    for i in 0..(normals.len()) {
                        obj.add_normal(i.try_into().unwrap(), normals[i]);
                        obj.add_uv(i.try_into().unwrap(), texture_coords[i]);
                    }
                    let id = self.id_counter.fetch_add(0, std::sync::atomic::Ordering::Acquire);
                    self.surfaces.push(components::component_system::ComponentRef_new(
                        Surface { 
                            id: id.clone(),
                            name: String::from(split[1]), 
                            verts: Vec::new(), 
                            indices: Vec::new(), 
                            normals: Vec::new(), 
                            texture_coord: Vec::new(), 
                            is_concave: false,
                            surface_type: SurfaceType::TRIANGLES
                        }
                    ));
                    self.materials.insert(id, Arc::new(Mutex::new(Material::new())));
                    current_object += 1;

                },
                Lntp::USE_MTL => {},
                Lntp::NONE => {

                }
            }

        }


        #[allow(non_camel_case_types)]
        enum Lntp {
            VERTEX,
            VERTEX_TEXTURE,
            VERTEX_NORMAL,
            FACE,
            MTLLIB,
            OBJECT_NAME,
            USE_MTL,
            NONE
        }

        fn get_number_of_lines(buffer: &str) -> usize{
            return buffer.lines().collect::<Vec<_>>().len();
        }
        fn get_line(buffer: &str, i: usize) -> &str{

            let lines = buffer.lines().collect::<Vec<_>>();
            if i > lines.len() {
                return "";
            }
            return lines[i];
        }

        fn check_line(line: &str) -> (Lntp, Vec<&str>)  {
            let mut result = Lntp::NONE;
            let split: Vec<_> = line.split_whitespace().collect();
            
            if split[0] == "v" {
                result = Lntp::VERTEX;
            }
            if split[0] == "vt" {
                result = Lntp::VERTEX_TEXTURE;
            }
            if split[0] == "vn" {
                result = Lntp::VERTEX_NORMAL;
            }
            if split[0] == "f" {
                result = Lntp::FACE;
            }
            if split[0] == "o" {
                result = Lntp::OBJECT_NAME;
            }
            if split[0] == "usemtl" {
                result = Lntp::USE_MTL;
            }
            if split[0] == "mtllib" {
                result = Lntp::MTLLIB;
            }

            

            return (result, split);
        }


    }




}

impl MeshFile {
    pub fn construct(definition: ConstructorDefinition) -> Self {
        let mesh_file_path = definition.get("mesh_file_path").unwrap().as_str().expect("Failed to get Mesh File Path");
        let mut mesh_file = Self::new();
        mesh_file.open(mesh_file_path.as_str());
        mesh_file
    }

    pub fn as_mesh(&self) -> Mesh {
        Mesh { surfaces: self.surfaces.clone(), transform: matrices::Matrix34::identity(), materials: self.materials.clone(), counter: AtomicU32::new(0)}
    }
}

impl MeshConstruct for Surface {

    /// This gives a list of points that are in a mesh
    fn add_points(&mut self, verts: Vec<Vec3>) {
        let &mut length = self.verts.len().borrow_mut();
        self.verts.append(&mut verts.clone());
        for i in 0..(&verts.len() - 1){
            self.normals.push(((length + i).try_into().unwrap(), Vec3::new(0, 0, 0)));
            self.texture_coord.push(((length + i).try_into().unwrap(), (0.0, 0.0)));
        }
    }
    fn add_face(&mut self, vert1: i16, vert2: i16, vert3: i16) {
        assert!(self.surface_type == SurfaceType::TRIANGLES);// Must be triangles for 
        self.indices.append(&mut vec![vert1, vert2, vert3]);
    }

    fn add_edge(&mut self, vert1: i16, vert2: i16){

    }

    fn add_normal(&mut self, index: i16, normal: Vec3) {
        for norm in &mut self.normals{
            if norm.0 == index {
                norm.1 = normal.clone();
                return;
            }
        }
    }

    fn add_point(&mut self, vert: Vec3) -> i16 {
        self.verts.push(vert);
        self.normals.push(((self.verts.len() - 1).try_into().unwrap(), Vec3::new(0, 0, 0)));
        self.texture_coord.push(((self.verts.len() - 1).try_into().unwrap(), (0.0, 0.0)));
        return (self.verts.len() - 1).try_into().unwrap();
    }

    fn add_uv(&mut self, index: i16, coord: (f32, f32)) {
        for uv in &mut self.texture_coord {
            if uv.0 == index {
                uv.1 = coord;
                return;
            }
        }
    }
}

impl Surface {
    pub fn new(name: String, surface_type: SurfaceType) -> Self {
        Self { id: 0, name: name.clone(), verts: Vec::new(), indices: Vec::new(), normals: Vec::new(), texture_coord: Vec::new(), is_concave: false, surface_type: surface_type }
    }
}