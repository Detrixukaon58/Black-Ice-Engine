use std::borrow::BorrowMut;
use std::ptr::null;
use std::str::SplitWhitespace;
use std::any::Any;
use std::thread::current;
use crate::common::{vertex::*, transform::*, engine::gamesys::*};
use crate::common::filesystem::files::*;
use crate::common::{materials::*, *};

// TODO: Add layer reference so that correct pipelines can get the correct meshes
/// Type of resource
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Mesh {
    pub name: String,
    pub verts: Vec<Vec3>,
    pub faces: Vec<(i16, i16, i16)>, //made of 3 verts
    pub normals: Vec<(i16, Vec3)>,
    pub textureCoord: Vec<(i16, (f32, f32))>,
    pub material: Box<Material>,
    pub isConcave: bool
}

pub struct MeshFile {
    objects: Vec<Mesh>,
    pub meshFile: FileSys,
    pub meshFileType: MFType,
    pub useCustomMaterials: bool
}

pub trait MeshInstanciate<T> {
    fn new() -> T;
}

pub trait MeshRender {
    fn render(&self, t: Transform);
    fn getVert(&self, i: i16) -> Vec3;
    fn getVerts(&self) -> Vec<Vec3>;
    fn getEdges(&self) -> Vec<i16>;
    fn getEdge(&self, i: i16) -> (i16, i16);
    fn getEdgesofVert(&self, i: i16) -> Vec<(i16, i16)>;
    fn getFace(&self, i: i16) -> (i16, i16, i16);
    fn getFaces(&self, i: i16) -> Vec<i16>;
    fn getVertsofFace(&self, i: i16) -> Vec<i16>;

    fn translatedVerts(&self, t: Transform) -> Vec<Vec3>;
    fn translatedFaces(&self, t: Transform) -> Vec<(i16, i16, i16)>;
    fn translatedEdges(&self, t: Transform) -> Vec<(i16, i16, i16)>;

}

pub trait MeshFileSys {
    fn open(&mut self, f: &str);
    fn openFBX(&mut self);
    fn openOBJ(&mut self);
}

pub trait MeshConstruct {
    fn givePoints(&mut self, verts: Vec<Vec3>);
    fn definePoint(&mut self, vert: Vec3) -> i16;
    fn defineFace(&mut self, vert1: i16, vert2: i16, vert3: i16) -> i16;
    fn defineNormal(&mut self, index: i16, normal: Vec3);
    fn defineUV(&mut self, index: i16, coord: (f32, f32));
}

//region Mesh Reflection
impl Base for Mesh{}
impl Base for MeshFile{}

impl Reflection for Mesh{
    fn registerReflect(&'static self) -> Ptr<Register> 
    {
        let mut register = Box::new(Register::new(Box::new(self)));
        
        register.addProp(Property { name: Box::new("faces"), desc: Box::new("The Faces of the object"), reference: Box::new(&self.faces), refType: self.faces.type_id()});
        register.addProp(Property { name: Box::new("verts"), desc: Box::new("The Vertices of the object"), reference: Box::new(&self.verts), refType: self.verts.type_id()});
        register.addProp(Property { name: Box::new("normals"), desc: Box::new("The normals of each face. This is ordered in the order of the faces. e.g. faces[1] has normal normals[1], faces[n] has normal normal[n]"), reference: Box::new(&self.normals), refType: self.normals.type_id()});
        
        register.addPointer(Pointer {name: Box::new("material"), desc: Box::new("This is the material that is being used my this mesh. This is a script either provided by the engine or by the developer."), reference: self.material.registerReflect(), refType: self.material.type_id()});

        return Ptr{ b: register};
    }
}

impl Base for Vec<Mesh>{}

impl Reflection for MeshFile {
    fn registerReflect(&'static self) -> Ptr<Register>{
        let mut register = Box::new(Register::new(Box::new(self)));

        //register.addPointer(Pointer {name: Box::new("objects"), desc: Box::new("The objects that make up a mesh"), reference: self.object.registerReflect(), refType: self.object.type_id()});
        //register.addProp(Property {name: Box::new("objects"), desc: Box::new("The Objects that make up a mesh file."), reference: Box::new(&self.objects), refType: self.objects.type_id()});

        return Ptr{b: register};
    }
}
//endregion

impl MeshInstanciate<MeshFile> for MeshFile {
    fn new() -> MeshFile {
        return MeshFile {objects: Vec::<Mesh>::new(), meshFile: FileSys::new(), meshFileType: MFType::UNKNOWN, useCustomMaterials: false};
    }
}

impl MeshFileSys for MeshFile {
    
    fn open(&mut self, f: &str){
        self.meshFile.open(f);
        self.meshFileType = self.meshFile.checkFileExt();
        
        match self.meshFileType {
            MFType::FBX=>self.openFBX(),
            MFType::OBJ=>self.openOBJ(),
            _=>{

            }
        }

    }
    fn openFBX(&mut self){

    }
    fn openOBJ(&mut self){
        let buffer = self.meshFile.read();
        let mut lineCount = 0;
        let mut materialFiles: Vec<FileSys> = Vec::<FileSys>::new();
        let mut textureCoords: Vec<(f32, f32)> = Vec::<(f32,f32)>::new();
        let mut normals: Vec<Vec3> = Vec::<Vec3>::new();
        //let mut vertices: Vec<Vec3> = Vec::<Vec3>::new();
        let mut currentObject = 0;
        
        

        for i in 1..(getNumberOfLines(&buffer) - 1){

            let line = getLine(&buffer, i);
            let (lineType, split) = checkLine(line);
            match lineType{
                lntp::VERTEX =>         {
                    let vertex = Vec3::new(split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap(), split[3].parse::<f32>().unwrap());
                    self.objects[currentObject].definePoint(vertex);
                },
                lntp::VERTEX_TEXTURE => {textureCoords.push((split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap()));},
                lntp::VERTEX_NORMAL => {normals.push(Vec3::new(split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap(), split[3].parse::<f32>().unwrap()))},
                lntp::FACE => {
                    let faceVertices = &split[1..];
                    if(faceVertices.len() > 3){
                        // Dealing with a non triangular face

                    }
                    else{
                        let mut a = faceVertices[0].split('/');
                        let mut b = faceVertices[1].split('/');
                        let mut c = faceVertices[2].split('/');
                        self.objects[currentObject].defineFace(a.nth(0).unwrap().parse::<i16>().unwrap(), b.nth(0).unwrap().parse::<i16>().unwrap(), c.nth(0).unwrap().parse::<i16>().unwrap());
                    }
                },
                lntp::MTLLIB => {},
                lntp::OBJECT_NAME => {
                    for i in 0..(normals.len()) {
                        self.objects[currentObject].defineNormal(i.try_into().unwrap(), normals[i]);
                        self.objects[currentObject].defineUV(i.try_into().unwrap(), textureCoords[i]);
                    }

                    self.objects.push(Mesh { name: String::from(split[1]), verts: Vec::new(), faces: Vec::new(), normals: Vec::new(), textureCoord: Vec::new(), material: Box::new(Material::new()), isConcave: false });
                    currentObject += 1;

                },
                lntp::USE_MTL => {},
                lntp::NONE => {

                }
            }

        }


        
        enum lntp {
            VERTEX,
            VERTEX_TEXTURE,
            VERTEX_NORMAL,
            FACE,
            MTLLIB,
            OBJECT_NAME,
            USE_MTL,
            NONE
        }

        fn getNumberOfLines(buffer: &str) -> usize{
            return buffer.lines().collect::<Vec<_>>().len();
        }
        fn getLine(buffer: &str, i: usize) -> &str{

            let lines = buffer.lines().collect::<Vec<_>>();
            if(i > lines.len()){
                return "";
            }
            return lines[i];
        }

        fn checkLine(line: &str) -> (lntp, Vec<&str>)  {
            let mut result = lntp::NONE;
            let split: Vec<_> = line.split_whitespace().collect();
            
            if(split[0] == "v"){
                result = lntp::VERTEX;
            }
            if(split[0] == "vt"){
                result = lntp::VERTEX_TEXTURE;
            }
            if(split[0] == "vn"){
                result = lntp::VERTEX_NORMAL;
            }
            if(split[0] == "f"){
                result = lntp::FACE;
            }
            if(split[0] == "o"){
                result = lntp::OBJECT_NAME;
            }
            if(split[0] == "usemtl"){
                result = lntp::USE_MTL;
            }
            if(split[0] == "mtllib"){
                result = lntp::MTLLIB;
            }

            

            return (result, split);
        }


    }




}

impl MeshConstruct for Mesh {

    /// This gives a list of points that are in a mesh
    fn givePoints(&mut self, verts: Vec<Vec3>) {
        let &mut length = self.verts.len().borrow_mut();
        self.verts.append(&mut verts.clone());
        for i in 0..(&verts.len() - 1){
            self.normals.push(((length + i).try_into().unwrap(), Vec3::new(0, 0, 0)));
            self.textureCoord.push(((length + i).try_into().unwrap(), (0.0, 0.0)));
        }
    }
    fn defineFace(&mut self, vert1: i16, vert2: i16, vert3: i16) -> i16 {
        self.faces.push((vert1, vert2, vert3));
        return (self.faces.len() - 1).try_into().unwrap();
    }

    fn defineNormal(&mut self, index: i16, normal: Vec3) {
        for norm in &mut self.normals{
            if(norm.0 == index){
                norm.1 = normal.clone();
                return;
            }
        }
    }

    fn definePoint(&mut self, vert: Vec3) -> i16 {
        self.verts.push(vert);
        self.normals.push(((self.verts.len() - 1).try_into().unwrap(), Vec3::new(0, 0, 0)));
        self.textureCoord.push(((self.verts.len() - 1).try_into().unwrap(), (0.0, 0.0)));
        return (self.verts.len() - 1).try_into().unwrap();
    }

    fn defineUV(&mut self, index: i16, coord: (f32, f32)) {
        for uv in &mut self.textureCoord {
            if uv.0 == index {
                uv.1 = coord;
                return;
            }
        }
    }
}