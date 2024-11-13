// this will be used to load asset packs into our game!!


use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::{fmt, io};
use std::{fs::File, path::PathBuf};
use std::sync::Arc;

use std::io::{BufRead, BufReader, Read, Seek};
use colored::*;
use parking_lot::*;
use crate::black_ice::common::Env;
use crate::black_ice::common::engine::asset_types::*;

#[derive(PartialEq)]
enum PathType {

    DIRECTORY,
    FILE,

}
//region Path Rep
struct PathRep {
    pub name: String,
    pub path_type: PathType,
    next: Option<HashMap<String, PathRep>>,
    data_offset: Option<usize>,
    data_size: Option<usize>
}

impl PathRep {
    pub fn new(name: String, path_type: PathType) -> Self {
        Self{
            name: name,
            path_type: path_type,
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

    pub fn set_data_size(&mut self, data: usize){
        if self.path_type == PathType::FILE{
            self.data_size = Some(data);
        }
    }

    pub fn get_data_size(&self) -> Option<usize>{
        self.data_size.clone()
    }
    pub fn get_next(&self, name: String) -> Option<&Self>{
        if self.path_type == PathType::DIRECTORY && self.next.is_some(){
            let path_reps = self.next.unwrap();
            return path_reps.get(&name)
        }
        else{
            None
        }
    }

    pub fn is_file(self) -> bool{
        self.path_type == PathType::FILE
    }

    pub fn is_dir(self) -> bool{
        self.path_type == PathType::DIRECTORY
    }
}
//endregion
pub struct AssetPack{
    pub asset_names: Vec<String>,
    pub asset_count: usize,
    pub asset_location: String,
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
    *     metadata here, enclosed by some deliminator
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
    *         [text]
    *         <
    *             hello world
    *         >
    *     >
    * >
    */
    fn version_0(buff_reader: &BufReader<File>, paths: &HashMap<String, PathRep>) -> Result<(), AssetPackLoadError>{
        
        fn read_until_or(r: &BufReader<File>, delim: u8, count: usize, buf: &mut [u8]) -> Result<usize, std::io::Error>{
            let mut read = 0;
            let mut early = false;
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
        let mut separator = 0;
        // check if the next character is a <
        // if not, then we crash out!!
        let mut delim: [u8; 1];
        match buff_reader.read_exact(&mut delim){
            Ok(_) => {},
            Err(e) => {return Err(AssetPackLoadError { source: AssetPackLoadErrorStackTrace::IoError(e)})}
        }
        if delim.is_ascii(){
            let temp: String = String::from_utf8(delim.to_vec()).expect("Failed to parse deliminator. The file may be corrupt!!");
            if temp == "<"{
                // this is correct, there is no corruption!!
                separator += 1;
            }
            else {
                return Err(AssetPackLoadError { source: AssetPackLoadErrorStackTrace::DelimError});
            }
        }
        else{
            return Err(AssetPackLoadError { source: AssetPackLoadErrorStackTrace::DelimError});
        }
        delim.copy_from_slice(b" ");// clear the delim so that we do not have
        let mut paths_unfinished_vec: Vec<(String, u8)> = vec![];// This will store our paths that we have so far read but haven't totally commited!!
        // To commit a PathRep, we must encounter a closing deliminator
        let mut path_reps_vec: Vec<HashMap<String, PathRep>> = vec![];// This stores the set of paths within a current directory!!
        let mut current_dir: usize = 0;// We start at directory 0, the root directory
        // This will be empty when we start since there are no directories stored yet!!

        while !is_finished {
            // get the path_name - the max character length is 512, and the deliminator is \0
            // make sure to go past the separator 
            // for the first itteration we will always treat the data as a dirtectory, We will not accept file only asset packs in this version
            
            // Get the path_name
            let mut path_name_u8: [u8; 512];
            let length = read_until_or(buff_reader, b'\0', 512, &mut path_name_u8).expect("Failed to read path name!!");
            let mut path_name = unsafe {String::from_raw_parts(path_name_u8.as_mut_ptr(), length, 512)};
            // now we read the next byte as a boolean value
            // We want to use a byte here to ensure that we have consistant spacing for the entire file!!
            let mut path_type_u8: [u8; 1];
            buff_reader.read_exact(&mut path_type_u8).expect("Failed to read path type!!");
            // now we just simply check if this byte is 0 or not 0
            match path_type_u8[0].clone() {
                0 => {// This is the file case
                    // We will read until the deliminator
                    // Since we don't know the size of our data and don't want to store this data in memory forever, we will have to quickly 
                    let mut size_u8: [u8; 8];
                    buff_reader.read_exact(&mut size_u8).expect("Failed to read file size!!");
                    let size: usize = usize::from_le_bytes(size_u8);
                    // now we need to get the metadata of the file, This must be done using string, soo we will keep reading a new character
                    // into some string
                    let mut metadata: String = String::new();
                    // read first character to check if there is metadata
                    let mut mt: [u8; 1];
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

                    let mut path_rep = PathRep::new(path_name, PathType::FILE);
                    path_rep.set_data_offset(start.try_into().expect("Failed to convert u64 offset to usize offset!!"));
                    path_rep.set_data_size(size);
                    path_reps_vec[current_dir].insert(path_name, path_rep);
                    let seek: u64 = (size + 2).try_into().unwrap();
                    buff_reader.seek(io::SeekFrom::Current(seek.try_into().unwrap()));
                }
                _ => {// This is the Directory case!!
                    // We will increment our separator value
                    // This will ensure that the next run will focus on a new set of data
                    // We will also commit our current path name and path type to our unfinished vector
                    buff_reader.seek(io::SeekFrom::Current(1));
                    paths_unfinished_vec.push((path_name, path_type_u8[0]));
                    path_reps_vec.push(HashMap::new());
                    current_dir+= 1;
                }
            }
            // now check the next bit if it closes the file
            let mut byte: [u8; 1];
            buff_reader.read_exact(&mut byte);
            match &byte{
                b">" =>{
                    // we can continue perfectly fine!!
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
        let temp3 = temp2.file_stem().unwrap();
        let filename = temp2.to_str().unwrap();
        let mut buff_reader = BufReader::new(asset_pack_file);
        let mut asset_names = Vec::<String>::new();
        let mut paths = HashMap::<String, PathRep>::new();

        // time to read the file and load the data
        // first we want to get some information about the file and how it is to be loaded!!
        let mut version: u32;// the first 4 bytes tells us the version of the file to allow for 
        // backwards compatability in fucture if in case updates are made to the file scheme
        let mut temp: [u8; 4];
        buff_reader.read_exact(&mut temp)
            .expect(format!("{}", "Failed to read file. There seems to be no data in this file or the file may be corrupted!!!".red()).as_str());
        
        version = u32::from_le_bytes(temp);// we will assume little endian bytes for all files loaded!!
        let error = match version.clone(){
            _ => {
                AssetPack::version_0(&buff_reader, &paths)
            }
        };

        match error {
            Err(e) => {
                println!("{}", e);
                panic!("")
            },
            Ok(_) => {}
        }

        let mut rep = PathRep::new(filename.to_string(), PathType::DIRECTORY);// this will always be a directory
        rep.set_next(paths);
        Self{
            asset_names: asset_names,
            asset_count: 0,
            asset_location: "".to_string(),
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


pub struct AssetData {

    pub data: Vec<u8>,
    pub metadata: Vec<String>,
}

pub struct AssetManager {

    asset_packs: HashMap<String, AssetPack>,
    debug_asset_pack_references: HashMap<String, PathBuf>,
    is_start: bool,
    thread_reciever: Arc<Mutex<Vec<AssetManagerData>>>,
    asset_data_reference: HashMap<String, Arc<AssetData>>, // we want this to be read only, as we do not want to edit the original assetpack!!,
    // If you want to edit files, then you must access them through the file system
    // This will mean that you will not be able to load them through the asset manager
    // and have to manage them yourself!!

}


impl AssetManager {

    pub fn new() -> Self{
        Self{
            asset_packs: HashMap::new(),
            debug_asset_pack_references: HashMap::new(),
            is_start: false,
            thread_reciever: Arc::new(Mutex::new(vec![])),
            asset_data_reference: HashMap::new()
        }
    }

    pub fn load_asset_pack(p_this: Arc<Mutex<Self>>, full_path: String){
        let asset_pack_path = PathBuf::from(full_path);
        let asset_pack = AssetPack::load(asset_pack_path);
        let mut this = p_this.lock();
        this.asset_packs.insert(asset_pack.asset_location, asset_pack);
        drop(this);
    }

    pub fn load_folder_as_asset_pack(p_this: Arc<Mutex<Self>>, full_path: String){
        let asset_pack_path = PathBuf::from(full_path);
        let mut this = p_this.lock();
        let temp = asset_pack_path.clone().file_name().unwrap();
        let temp2 = PathBuf::from(temp.to_str().unwrap());
        let temp3 = temp2.file_stem().unwrap();
        let temp4 = temp3.to_str().unwrap();
        this.debug_asset_pack_references.insert(String::from_str(temp4).unwrap(), asset_pack_path);
    }

    pub fn load_asset<T>(path: String) -> T where T : AssetResource {
        // first we check if the asset has already been loaded
        unsafe {
            let p_asset_mg = Env::get_asset_mgr();
            let mut asset_mg = p_asset_mg.lock();
            let debug_asset_path = asset_mg.debug_asset_pack_references.get(&path);

            let mut asset = T::new();

            // find the asset and read the data in!!
            // if we are in debug, then we will try to use the debug reference
            let mut asset_data: Some<AssetData> = None;
            if debug_asset_path.is_some() {
                let mut debug_asset = debug_asset_path.unwrap();
                let mut debug_asset_file = File::open(debug_asset).expect("Failed to open asset file!!");
                let mut debug_assset_reader = BufReader::new(debug_asset_file);
                let mut debug_asset_data = Vec::<u8>::new();
                debug_assset_reader.read_to_end(&mut debug_asset_data);

                let mut debug_asset_metadata = Vec::<String>::new();

                // read the file's metadata
                let mut temp: AssetData = AssetData { data: debug_asset_data, metadata: debug_asset_metadata };
            }
            else{
                // first lets get the asset pack that we need
                // This will need to be the first path in the 
            }

            asset.init();


            asset

        }

    }

    pub fn processing(p_this: Arc<Mutex<Self>>){

        loop {
            let this = p_this.lock();
            if this.is_start {
                drop(this);
                break;
            }
            drop(this);
        }

        unsafe{
            while !Env::isExit(){
                // we will organise all asset packs here
                let this = p_this.lock();
                let mut p_recv = this.thread_reciever.clone();
                drop(this);
                let mut recv = p_recv.try_lock();
                if let Some(ref mut mutex) = recv {
                    for th in mutex.as_slice() {
                        let data = th.clone();
                        match data {
                            _ => ()
                        }
                    }
                    mutex.clear();
                }
                drop(recv);

            }
        }

    }


    pub fn start(p_this: Arc<Mutex<Self>>){
        let mut this = p_this.lock();
        this.is_start = true;
        drop(this);
    }

    pub fn init(this: Arc<Mutex<Self>>){
        Self::processing(this);
    }
}
