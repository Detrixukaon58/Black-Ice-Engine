// this will be used to load asset packs into our game!!


use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::hash::Hash;
use std::os::unix::fs::MetadataExt;
use std::str::FromStr;
use std::{fmt, io};
use std::{fs::File, path::PathBuf};
use std::sync::Arc;

use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use colored::*;
use futures::future::err;
use parking_lot::*;
use crate::black_ice::common::Env;
use crate::black_ice::common::engine::asset_types::*;

use super::input;

#[derive(PartialEq, Clone)]
enum PathType {

    DIRECTORY,
    FILE,

}
//region Path Rep
#[derive(Clone)]
struct PathRep {
    pub name: String,
    pub path_type: PathType,
    pub meta_data: HashMap<String, String>,
    file_path: Option<PathBuf>,
    next: Option<HashMap<String, PathRep>>,
    data_offset: Option<usize>,
    data_size: Option<u64>
}

impl PathRep {
    pub fn new(name: String, path_type: PathType, meta_data: Option<HashMap<String,String>>) -> Self {
        Self{
            name: name,
            path_type: path_type,
            meta_data: meta_data.unwrap_or_else(|| {HashMap::<String,String>::new()}),
            file_path:None,
            next: None,
            data_offset: None,
            data_size: None
        }
    }

    pub fn set_next(&mut self, next: HashMap<String, PathRep>){
        if self.path_type == PathType::DIRECTORY{
            self.next = Some(next);
        }
    }

    pub fn set_data_offset(&mut self, data: usize){
        if self.path_type == PathType::FILE{
            self.data_offset = Some(data);
        }
    }

    pub fn get_data_offset(&self) -> Option<usize>{
        self.data_offset.clone()
    }

    pub fn set_data_size(&mut self, data: u64){
        if self.path_type == PathType::FILE{
            self.data_size = Some(data);
        }
    }

    pub fn get_data_size(&self) -> Option<u64>{
        self.data_size.clone()
    }
    pub fn get_next(&self, name: String) -> Option<&Self>{
        if self.path_type == PathType::DIRECTORY && self.next.is_some(){
            let path_reps = self.next.as_ref().unwrap();
            return path_reps.get(&name)
        }
        else{
            None
        }
    }

    pub fn is_file(&self) -> bool{
        self.path_type == PathType::FILE
    }

    pub fn is_dir(&self) -> bool{
        self.path_type == PathType::DIRECTORY
    }

    pub fn set_file_path(&mut self, path: PathBuf){
        self.file_path = Some(path);
    }

    pub fn get_file_path(&self) -> Option<PathBuf> {
        self.file_path.clone()
    }
}
//endregion

#[derive(Clone)]
pub struct AssetPack{
    pub asset_location: String,// the physical location of the asset pack
    version: u32,
    rep: PathRep
}

#[derive(Debug)]
pub enum AssetPackLoadErrorStackTrace {
    IoError(io::Error),
    FmtError(fmt::Error),
    DelimError,

}

impl fmt::Display for AssetPackLoadErrorStackTrace{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "{}", e),
            Self::FmtError(e) => write!(f, "{}", e),
            Self::DelimError => write!(f, "Deliminator is missing and/or corrupted. Please re-compile the asset pack again or call an issue on the git repo!")
        }
    }
}

impl Error for AssetPackLoadErrorStackTrace {}

#[derive(Debug)]
pub struct AssetPackLoadError {
    source: AssetPackLoadErrorStackTrace
}

impl fmt::Display for AssetPackLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to load the AssetPack!! Please look at the stack trace for more information.")
    }
}

impl Error for AssetPackLoadError {}

impl AssetPack{
    /** Format: 
    * version
    * <
    *     path_name (the name of the directory or file, stored as ascii, null terminated!!)
    *     path_type (a bit that tells us if it is a file or directory)
    *     (file_size only shown if previous is 0)
    *     metadata here, enclosed by some deliminator and seperated by commas
    *     < (start of data)
    *         data (this could either be more files and directories iff the parent is a directory)
    *              (or it could be some data for a file iff the parent is a file)
    *     > (end of file)
    *     | (tells us there are more after this!!)
    *     our_dir
    *     1 (Directory)
    *     <
    *         our_file
    *         0 (File)
    *         12 (in bytes, only applicable to files. Written as a 64 bit integer)
    *         [type:text,ext:txt,author:Detrix,width:100,height:100,key:value]
    *         <
    *             hello world
    *         >
    *     >
    * >
    */
    fn version_0(buff_reader: &mut BufReader<File>, paths: &mut HashMap<String, PathRep>, asset_file_metadata: &mut HashMap<String, Vec<String>>, asset_pack_name: String) -> Result<(), AssetPackLoadError>{
        
        fn read_until_or(r: &mut BufReader<File>, delim: u8, _count: usize, buf: &mut [u8]) -> Result<usize, std::io::Error>{
            let mut read = 0;
            let _early = false;
            let mut vec:Vec<u8> = vec![];
            'main: loop {
                let (done, used) = {
                    let available = match r.fill_buf() {
                        Ok(n) => n,
                        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e),
                    };
                    // check if the available space 
                    
                    match available.iter().position(|c| c == &delim){// in future replace with memchr::memchr crate!!
                        Some(i) => {
                            if i > buf.len() - read {
                                // only copy acceptable ammount
                                let selected = available.get(..buf.len() - read).expect("Failed to read buffer. Couldn't read buffer of large size into remainder of buffer.");
                                vec.extend_from_slice(selected);
                                (true, buf.len() - read)
                            }
                            else{
                                vec.extend_from_slice(available.get(..i).expect("Failed to get sub section!!"));
                                (true, i)
                            }
                        },
                        None => {
                            if available.len() > buf.len() - read {
                                // only copy acceptable ammount
                                let selected = available.get(..buf.len() - read).expect("Failed to read buffer. Couldn't read buffer of large size into remainder of buffer.");
                                vec.extend_from_slice(selected);
                                (true, buf.len() - read)
                            }
                            else{
                                vec.extend_from_slice(available);
                                (false, available.len())
                            }
                        }
                    }
                    
                };
                r.consume(used);
                read += used;
                if done || used == 0 {
                    break 'main;
                }
            }
            vec.as_slice().read_exact(buf).expect("Could not read into buffer!!");
            return Ok(read);
        }

        let mut is_finished = false;
        let mut _separator = 0;
        // check if the next character is a <
        // if not, then we crash out!!
        let mut delim: [u8; 1] = [0;1];
        match buff_reader.read_exact(&mut delim){
            Ok(_) => {},
            Err(e) => {return Err(AssetPackLoadError { source: AssetPackLoadErrorStackTrace::IoError(e)})}
        }
        if delim.is_ascii(){
            let temp: String = String::from_utf8(delim.to_vec()).expect("Failed to parse deliminator. The file may be corrupt!!");
            if temp == "<"{
                // this is correct, there is no corruption!!
                _separator += 1;
            }
            else {
                return Err(AssetPackLoadError { source: AssetPackLoadErrorStackTrace::DelimError});
            }
        }
        else{
            return Err(AssetPackLoadError { source: AssetPackLoadErrorStackTrace::DelimError});
        }
        delim.copy_from_slice(b" ");// clear the delim so that we do not have
        let mut paths_unfinished_vec: Vec<(String, u8)> = vec![(asset_pack_name, 1)];// This will store our paths that we have so far read but haven't totally commited!!
        // To commit a PathRep, we must encounter a closing deliminator
        let mut path_reps_vec: Vec<HashMap<String, PathRep>> = vec![HashMap::new()];// This stores the set of paths within a current directory!!
        let mut current_dir: usize = 0;// We start at directory 0, the root directory
        // This will be empty when we start since there are no directories stored yet!!

        while !is_finished {
            // get the path_name - the max character length is 512, and the deliminator is \0
            // make sure to go past the separator 
            // for the first itteration we will always treat the data as a dirtectory, We will not accept file only asset packs in this version
            
            // Get the path_name
            let mut path_name_u8: [u8; 512] = [0;512];
            let length = read_until_or(buff_reader, b'\0', 512, &mut path_name_u8).expect("Failed to read path name!!");
            let path_name = unsafe {String::from_raw_parts(path_name_u8.as_mut_ptr(), length, 512)};
            // now we read the next byte as a boolean value
            // We want to use a byte here to ensure that we have consistant spacing for the entire file!!
            let mut path_type_u8: [u8; 1] = [0;1];
            buff_reader.read_exact(&mut path_type_u8).expect("Failed to read path type!!");
            // now we just simply check if this byte is 0 or not 0
            match path_type_u8[0].clone() {
                0 => {// This is the file case
                    // We will read until the deliminator
                    // Since we don't know the size of our data and don't want to store this data in memory forever, we will have to quickly 
                    let mut size_u8: [u8; 8] = [0;8];
                    buff_reader.read_exact(&mut size_u8).expect("Failed to read file size!!");
                    let size: u64 = u64::from_le_bytes(size_u8);
                    // now we need to get the metadata of the file, This must be done using string, soo we will keep reading a new character
                    // into some string
                    let mut metadata: String = String::new();
                    // read first character to check if there is metadata
                    let mut mt: [u8; 1] = [0;1];
                    buff_reader.read_exact(&mut mt).expect("Cannot read metadata!!");
                    if mt[0] == b'[' {
                        // we have metadata
                        // now we keep reading till we reach the end of the ascii list!!
                        loop {
                            buff_reader.read_exact(&mut mt).expect("Cannot read metadata!!");
                            if mt[0] == b']' {
                                break;
                            }
                            metadata.push(mt[0].into());
                        }
                        
                    }
                    else{
                        // we don't have metadata
                        buff_reader.seek(io::SeekFrom::Current(-1)).expect("Failed to reset head!!");
                    }
                    let start = buff_reader.seek(io::SeekFrom::Current(0)).expect("Failed to get current position of head");
                    
                    // add the asset metadata to asset_file_metadata
                    let metadata_list: Vec<String> = metadata.clone().split(',').map(|s| {String::from(s)}).collect();
                    asset_file_metadata.insert(path_name.clone(), metadata_list.clone());

                    let mut metadata_map: HashMap<String,String> = HashMap::new();

                    for mdat in metadata_list {
                        let mdat_split = mdat.split(":").map(|f| {String::from(f)}).collect::<Vec<String>>();
                        metadata_map.insert(mdat_split[0].clone(), mdat_split[1].clone());
                    }
                    
                    let mut path_rep = PathRep::new(path_name.clone(), PathType::FILE, Some(metadata_map));
                    path_rep.set_data_offset(start.try_into().expect("Failed to convert u64 offset to usize offset!!"));
                    path_rep.set_data_size(size);
                    path_reps_vec[current_dir].insert(path_name, path_rep);
                    let seek: u64 = (size + 2).try_into().unwrap();
                    let _ = buff_reader.seek(io::SeekFrom::Current(seek.try_into().unwrap()));
                }
                _ => {// This is the Directory case!!
                    // We will increment our separator value
                    // This will ensure that the next run will focus on a new set of data
                    // We will also commit our current path name and path type to our unfinished vector
                    let _ = buff_reader.seek(io::SeekFrom::Current(1));
                    paths_unfinished_vec.push((path_name, path_type_u8[0]));
                    path_reps_vec.push(HashMap::new());
                    current_dir+= 1;
                }
            }
            // now check the next bit if it closes the file
            let mut byte: [u8; 1] = [0;1];
            let _ = buff_reader.read_exact(&mut byte);
            match &byte{
                b">" =>{
                    if current_dir > 0 {
                        // we close the directory and add it to the previous pathrep
                        let (a,_b) = &paths_unfinished_vec[current_dir - 1];
                        // we will add this directory as a pathrep
                        let mut r = PathRep::new(a.clone(), PathType::DIRECTORY, None);
                        r.set_next(path_reps_vec[current_dir -1].clone());

                        current_dir -= 1;
                        path_reps_vec[current_dir].insert(a.to_string(), r);
                    }
                    else{
                        // we have fully read the file!!
                        // we will now try to exit from here
                        *paths = path_reps_vec[0].clone();
                        path_reps_vec.clear();
                        paths_unfinished_vec.clear();
                        is_finished = true;
                    }
                    

                }
                _ =>{
                    return Err(AssetPackLoadError { source: AssetPackLoadErrorStackTrace::DelimError});
                }
            }

        }
        
        return Ok(());
    }

    pub fn load(path: PathBuf) -> Self {
        assert!(path.is_file());
        assert_eq!("pkg", path.extension().unwrap());
        // this is a file and we can load it correctly!!
        let asset_pack_file = File::open(path.clone()).expect("Failed to open file!!");
        let temp = path.as_path().file_name().unwrap();
        let temp2 = PathBuf::from(temp.to_str().unwrap());
        let _temp3 = temp2.file_stem().unwrap();
        let filename = temp2.to_str().unwrap();
        let mut buff_reader = BufReader::new(asset_pack_file);
        let mut paths = HashMap::<String, PathRep>::new();
        let mut metadata = HashMap::<String, Vec<String>>::new();

        // time to read the file and load the data
        // first we want to get some information about the file and how it is to be loaded!!
        let version: u32;// the first 4 bytes tells us the version of the file to allow for 
        // backwards compatability in fucture if in case updates are made to the file scheme
        let mut temp: [u8; 4] = [0;4];
        buff_reader.read_exact(&mut temp)
            .expect(format!("{}", "Failed to read file. There seems to be no data in this file or the file may be corrupted!!!".red()).as_str());
        
        version = u32::from_le_bytes(temp);// we will assume little endian bytes for all files loaded!!


        let error = match version.clone(){
            _ => {
                AssetPack::version_0(&mut buff_reader, &mut paths, &mut metadata, filename.to_string())
            }
        };

        match error {
            Err(e) => {
                //println!("{}", e);
                panic!("")
            },
            Ok(_) => {}
        }

        let mut rep = PathRep::new(filename.to_string(), PathType::DIRECTORY, None);// this will always be a directory
        rep.set_next(paths);
        Self{
            asset_location: filename.to_string(),
            version: version,
            rep: rep
        }
    }

    // We want to also be able to preload our shaders and any other data
    // This function should let us do that for the shaders
    // The next one should do it for any asset

}

pub enum AssetManagerData {

}

pub struct AssetFolder {
    pub directory_location: PathBuf,
    rep: PathRep,
}

impl AssetFolder {

    pub fn load(path: PathBuf) -> Self {
        // we will go through the directory recfursively and gathering the metadata for all the files
        let mut read_dir = path.read_dir().expect("Failed to read directory.");
        let mut paths = vec![(read_dir.enumerate(), false, path.clone(), HashMap::<String, PathRep>::new())];
        let mut is_finished = false;
        let mut path_rep = PathRep::new(
            String::from(path.clone().file_name()
                .expect("Failed to get directory filename")
                .to_str().expect("Failed to convert OSString")), 
            PathType::DIRECTORY, 
            None
        );

        while !is_finished {
            
            // we will go through the directory till we find another one, and then 
            let current = paths.last_mut().expect("Error during processing!! Please report this bug!!");
            let next = current.0.next();
            if let Some((_size, d)) = next {
                if let Ok(dir) = d {
                    let dir_path = dir.path();
                    if dir_path.is_dir() && !dir_path.is_symlink() {
                        // we will traverse this before continueing
                        read_dir = dir_path.read_dir().expect("Failed to read directory.");
                        paths.push((read_dir.enumerate(), false, dir_path.clone(), HashMap::<String, PathRep>::new()));
                    }
                    else if dir_path.is_file(){
                        // we will take note of this file in a hashmap
                        let mut meta_data: HashMap<String, String> = HashMap::<String,String>::new();

                        //lets read the file and get some metadata
                        let file = File::open(dir_path.clone()).expect("Failed to read file");
                        let file_metadata = file.metadata().expect("Failed to read metadata of the file");
                        let file_extension = String::from(dir_path.extension().expect("Failed to get file extension").to_str().unwrap());

                        let file_type = match file_extension.clone().to_lowercase().as_str() {
                            "png" | "jpeg" | "bmp" | "tga" | "dxt" | "dds" => "Image".to_string(),
                            "glsl" | "hlsl" | "pfx" | "comp" | "vert" | "frag" | "gdshad" | "fx" => "Shader".to_string(),
                            "obj" | "gltf" | "glb" | "stl" => "Mesh".to_string(),
                            "txt" | "json" | "xml" => "Text".to_string(), 
                            _ => "custom".to_string()
                        };

                        meta_data.insert("type".to_string(), file_type.clone());
                        meta_data.insert("ext".to_string(), file_extension.clone().to_ascii_lowercase());

                        match file_type.clone().as_str() {
                            "Image" => {
                                // we will get the image dimensions
                                let mut o_mage = image::ImageReader::new(BufReader::new(file));
                                o_mage.set_format(image::ImageFormat::from_extension(file_extension.clone()).expect(format!("Cannot currently load image formats of the type {}", file_extension).as_str()));
                                let mage = o_mage.decode().expect("Failed to read image!!");
                                meta_data.insert("width".to_string(), mage.width().to_string());
                                meta_data.insert("height".to_string(), mage.height().to_string());
                                meta_data.insert("depth".to_string(), "32".to_string());
                            },
                            "Shader" => {
                                // we need to preload this in order to ensure that we have the shaders ready!!

                                
                            }
                            _ => {}
                        };



                        let mut rep = PathRep::new(String::from(dir_path.file_name().unwrap().to_str().unwrap()), PathType::FILE, Some(meta_data));
                        
                        rep.set_data_size(file_metadata.size());
                        rep.set_file_path(dir_path.clone());
                        current.3.insert(String::from(dir_path.file_name().unwrap().to_str().unwrap()), rep);
                        
                    }
                }
            }
            else {
                // assume that we have traversed the full directory
                if paths.len() > 1{
                    // we will pop the last path and add it the the previous pathrep
                    let dir = paths.pop().expect("Error getting from paths list");

                    let mut rep = PathRep::new(String::from(dir.2.file_name().unwrap().to_str().unwrap()), PathType::DIRECTORY, None);
                    rep.set_next(dir.3);
                    

                    let last = paths.last_mut().unwrap();
                    last.3.insert(rep.name.clone(), rep);
                }
                else {
                    // we have finished the traversal
                    let dir = paths.pop().expect("Error getting from paths list");
                    path_rep.set_next(dir.3);
                    is_finished = true;
                }
            }

        }

        Self { directory_location: path, rep: path_rep }

    }

}

pub struct AssetData {

    pub asset_name: String,
    pub asset_path: String,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

pub struct AssetManager {

    asset_packs: HashMap<String, AssetPack>,
    asset_folders: HashMap<String, AssetFolder>,// changed this to asset folders so as to allow developers to not have to use asset packs
                                            // and to make it easier to implement asset pack creation from folders
    asset_data_reference: HashMap<String, Arc<AssetData>>, // we want this to be read only, as we do not want to edit the original assetpack!!,
    // If you want to edit files, then you must access them through the file system
    // This will mean that you will not be able to load them through the asset manager
    // and have to manage them yourself!!

}


impl AssetManager {

    pub fn new() -> Self{
        Self{
            asset_packs: HashMap::new(),
            asset_folders: HashMap::new(),
            asset_data_reference: HashMap::new()
        }
    }

    pub fn load_asset_pack(full_path: String){
        unsafe{
            let p_this = Env::get_asset_mgr();
            let asset_pack_path = PathBuf::from(full_path);
            let asset_pack = AssetPack::load(asset_pack_path);
            let mut this = p_this.lock();
            this.asset_packs.insert(asset_pack.asset_location.clone(), asset_pack);
            drop(this);
        }
    }

    pub fn load_asset_folder(full_path: String){
        unsafe{
            let p_this = Env::get_asset_mgr();
            let asset_pack_path = PathBuf::from(full_path);
            let mut this = p_this.lock();
            let temp = asset_pack_path.file_name().unwrap();
            let temp2 = PathBuf::from(temp.to_str().unwrap());
            let temp3 = temp2.file_stem().unwrap();
            let temp4 = temp3.to_str().unwrap();
            let asset_folder = AssetFolder::load(asset_pack_path);
            this.asset_folders.insert(String::from_str(temp4).unwrap(), asset_folder);
        }
    }

    pub fn load_asset<T>(path: String) -> T where T : AssetResource {
        // first we check if the asset has already been loaded
        unsafe {
            let p_asset_mg = Env::get_asset_mgr();
            let mut asset_mg = p_asset_mg.lock();
            

            let mut asset = T::new();

            // find the asset and read the data in!!
            // if we are in debug, then we will try to use the debug reference if the asset is not loaded already

            let mut asset_data: Option<Arc<AssetData>> = None;
            if let Some(pre_loaded_asset) = asset_mg.asset_data_reference.get(&path) {
                
                let ref_asset_data = pre_loaded_asset;
                asset_data = Some(ref_asset_data.clone());

            }
            else{

                // first lets get the asset pack that we need
                // This will need to be the first directory in the path
                // e.g. ASSET:pack/...
                let path_string: String = path[6..].to_string();
                let pack_name: String = path_string[..(path_string.find('/').unwrap_or_else(|| {path_string.len()}))].to_string();

                let packs_result = asset_mg.asset_packs.get(&pack_name);
                let folders_result = asset_mg.asset_folders.get(&pack_name);

                // now we traverse through the pathrep
                if let Some(result) = packs_result {
                    let mut temp = &result.rep;
                    let mut path_list = path_string.as_str().rsplit('/').collect::<VecDeque<&str>>();
                    
                    while !temp.is_file(){
                        if path_list.len() == 0 {
                            panic!("We couldn't find the asset!!! This could be a serious bug, please report this!!")
                        }
                        if temp.name == path_list[0] {
                            path_list.pop_front();
                            temp = {
                                let t = temp.next.as_ref().expect("not a directory");
                                let mut out: Option<&PathRep> = None;
                                for entry in t {
                                    if entry.0 == path_list[0] {
                                        out = Some(entry.1);
                                        break;
                                    }
                                }
                                out
                            }.expect(format!("Failed to find next path {}", path_list[0]).as_str());
                        }
                        
                    }

                    // then after we have found the file, we must load it!!
                    // lets get the asset pack file and read the specific data

                }
                else if let Some(result) = folders_result {
                    let mut temp = &result.rep;
                    let mut path_list = path_string.as_str().split('/').collect::<VecDeque<&str>>();
                    
                    while !temp.is_file(){
                        if path_list.len() == 0 {
                            panic!("We couldn't find the asset!!! This could be a serious bug, please report this!!")
                        }
                        if temp.name == path_list[0] {
                            path_list.pop_front();
                            temp = {
                                let t = temp.next.as_ref().expect("not a directory");
                                let mut out: Option<&PathRep> = None;
                                for entry in t {
                                    if entry.0 == path_list[0] {
                                        out = Some(entry.1);
                                        break;
                                    }
                                }
                                out
                            }.expect(format!("Failed to find next path {}", path_list[0]).as_str());
                        }
                        
                    }
                    // lets load the file
                    // we should have saved it when we first traversed it!!
                    let file = File::open(temp.get_file_path().expect("No file path associated with this path!! This is a bug!!")).expect("Failed to open the asset file!!");
                    let mut reader = BufReader::new(file);
                    let mut data: Vec<u8> = Vec::<u8>::new();
                    let _ = reader.read_to_end(&mut data);
                    let d = Arc::new(
                        AssetData {
                            asset_name: temp.name.clone(),
                            asset_path: path.clone(),
                            data: data,
                            metadata: temp.meta_data.clone()
                        }
                    );
                    asset_mg.asset_data_reference.insert(path.clone(), d.clone());

                    asset_data = Some(
                        d.clone()
                    );
                }
                

                // when we have loaded it, we must add it into the preloaded asset list so we can reference it again when we need to use it again
                // unless we have been asked to delist it for memory consuption minimisation (we will only do this for items that need to be edited, 
                // so must have an unique memory emtry)
                    

                
            }
            
            // now that we have the data, we can pass it along to the asset



            asset.init(asset_data.expect(format!("Failed to load an asset. Are you sure that the asset folder has been exposed? asset:{}", path.clone()).as_str()));


            asset

        }

    }
}
