use std::any::Any;
use std::fmt::{Display, Error};
use std::fs::{*, self};
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::ptr::*;
use serde::*;

use crate::common::{APP_DIR, concat_str};
use crate::common::engine::gamesys::*;


const ASSET_PATH: &str =  "F:\\Rust\\Program 1\\assets";

#[derive(Clone, Serialize, Deserialize)]
pub struct AssetPath {
    path: String,
}

impl Base for AssetPath{}

impl AssetPath {
    pub fn default() -> Self {
        Self { path: String::new() }
    }

    pub fn new(path: String) -> Self {
        Self { path: path }
    }

    pub fn open_as_file(&mut self) -> FileSys {
        let mut file = FileSys::new();
        file.open(self.path.as_str());
        file
    }

    pub fn get_file_name(&mut self) -> String {
        let mut temp = self.path.clone();
        temp = temp.replace("ASSET:", "");
        while temp.find("\\").is_some() {
            temp.remove(0);
        }

        temp
    }

    pub fn get_file_ext(&mut self) -> String{
        let mut temp = self.get_file_name();
        let a: Vec<&str> = temp.split(".").collect();
        String::from(*a.last().unwrap())
    }
}
pub struct FileSys{

    f: Option<File>,
    b: Option<BufReader<File>>,
    is_dir: bool,
    is_file: bool,
    p: Option<ReadDir>,
    total_len: usize,
    i: usize,
    pub path: String

}

impl Base for FileSys {}
impl Base for Option<File> {}
impl Reflection for FileSys {
    fn registerReflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));
        
        register.addProp(Property{
            name: Box::new("path"), 
            desc: Box::new("Path of file."), 
            reference: Box::new(&self.path), 
            refType: self.path.type_id()
        });
        return Ptr {b: register};
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MFType{

    NO_EXT,
    FBX,
    OBJ,
    MTL,
    PNG,
    SHADER,
    UNKNOWN


}

trait Handlers {
    fn fbxHandler(&mut self) -> String;
    fn objHandler(&mut self) -> String;
    fn mtlHandler(&mut self) -> String;
    fn pngHandler(&mut self) -> String;
}

pub trait Reader<U> {
    fn read(&mut self) -> U;
    fn read_file(&mut self) -> U;
    fn read_dir(&mut self) -> Result<U, Error>;
}

impl FileSys{

    pub fn new() -> FileSys{
        return FileSys { f: None, b: None, p: None, i: 0, total_len: 0, is_file: false, is_dir: false, path: String::from("") };
    }

    pub fn open(&mut self, _path: &str){
        // do more to getting it for only local systems e.g. get resources from pak files when in release and from local assets when in debug
        println!("{}", _path);
        let full_path = format!("{}\\{}", ASSET_PATH, _path[7..].to_owned());
        let dir = fs::metadata(full_path.clone()).unwrap();
        
        if(dir.is_file()){
            self.f = Option::Some(File::open(full_path.as_str()).expect(format!("File {} not found!", _path[7..].to_owned()).as_str()));// format "ASSET:\\path\\to\\file" => "DRIVE:\\path\\to\\assets\\path\\to\\file"
            self.path = String::from(_path);
            self.b = Option::Some(BufReader::new((*self.f.as_ref().unwrap()).try_clone().expect("Couldn't clone file for BufReader!!")));
            self.is_file = true;
            
        }
        else if (dir.is_dir()){
            self.p = Option::Some(fs::read_dir(full_path).unwrap());
            self.path = String::from(_path);
            self.is_dir = true;
            self.total_len = self.p.as_mut().unwrap().count();
        }
    }

    pub fn checkFileExt(&self) -> MFType{
        
        let mut lastI = 0;

        for i in 0..(self.path.len()){

            if(self.path.chars().nth(i).unwrap() == '.'){

                lastI = i;

            }
            
        }

        if(lastI == 0){
            return MFType::UNKNOWN;
        }

        let ext = self.path.get(lastI+1..).unwrap();
        let mut mf = MFType::NO_EXT;
        
        if(ext.eq("fbx") || ext.eq(".fbx")){
            mf = MFType::FBX;
        }
        if(ext.eq("png") || ext.eq(".png")){
            mf = MFType::PNG;
        }
        if(ext.eq(".obj") || ext.eq("obj")){
            mf = MFType::OBJ;
        }
        if(ext.eq("mtl") || ext.eq(".mtl")){
            mf = MFType::MTL;
        }
        if(ext.eq(".shad") || ext.eq("shad")){
            mf = MFType::SHADER;
        }
        return mf;

    }


}

impl Reader<String> for FileSys {
    fn read(&mut self) -> String {

        let mut result:String = String::from("");

        match self.checkFileExt() {
            MFType::FBX=>{
                result = self.fbxHandler();
            },
            MFType::OBJ=>{
                result = self.objHandler();
            },
            MFType::MTL=>{
                result = self.mtlHandler();
            },
            MFType::PNG=>{
                result = self.pngHandler();
            }
            _=>{

            }
        }

        return result;
        
    }

    fn read_file(&mut self) -> String {

        let mut result:String = String::from("");

        match self.checkFileExt() {
            MFType::FBX=>{
                result = self.fbxHandler();
            },
            MFType::OBJ=>{
                result = self.objHandler();
            },
            MFType::MTL=>{
                result = self.mtlHandler();
            },
            MFType::PNG=>{
                result = self.pngHandler();
            }
            _=>{

            }
        }

        return result;
    }

    /// This gets the path for each file in the directory
    fn read_dir(&mut self) -> Result<String, Error> {
        if(self.i == self.total_len){
            panic!("End of Directory!");
        }
        if(!self.is_dir){
            panic!("Not a Directory!!");
        }
        let mut entry: String = String::from("null");
        let mut j = 0;
        for mut path in self.p.as_mut().unwrap() {
            if(j == self.i){
                entry = path.unwrap().path().canonicalize().unwrap().display().to_string();
            }
        }
        if(entry.eq("null")){
            panic!("Error trying ot open file. Must be a programatical error. Please contact the developer!")
        }
        self.i +=1;
        return Result::Ok(entry);
    }

}

impl Handlers for FileSys{
    fn fbxHandler(&mut self) -> String {

        unimplemented!();
    }
    fn mtlHandler(&mut self) -> String {
        unimplemented!()
    }
    fn objHandler(&mut self) -> String {
        let mut buff = self.b.as_mut().unwrap();
        let mut result = String::from("");

        (*buff).read_to_string(&mut result).expect("Couldn't read anything!!");
        return result;
    }
    fn pngHandler(&mut self) -> String {
        let mut buff = self.b.as_mut().unwrap();
        let mut temp: Vec<u8> = vec![];
        let mut result = String::from("");

        (*buff).read_to_end(&mut temp).expect("Couldn't read anything!!");
        return result;
    }

}

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
            &MFType::NO_EXT=>{
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