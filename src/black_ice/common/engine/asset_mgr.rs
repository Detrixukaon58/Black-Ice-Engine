// this will be used to load asset packs into our game!!


use std::collections::HashMap;
use std::error::Error;
use std::{fmt, io};
use std::{fs::File, path::PathBuf};

use std::io::{BufRead, BufReader, Read, Seek};
use colored::*;

#[derive(PartialEq)]
enum PathType {

    DIRECTORY,
    FILE,

}

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
        let mut paths_unfinished_vec: Vec<(String, u8)> = vec![()];// This will store our paths that we have so far read but haven't totally commited!!
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
            // now check the next bit if it closes the 
            
        }
        
        return Ok(());
    }

    pub fn load(path: PathBuf) -> Self {
        assert!(path.is_file());
        assert_eq!("pkg", path.extension().unwrap());
        // this is a file and we can load it correctly!!
        let asset_pack_file = File::open(path.clone()).expect("Failed to open file!!");
        let temp = path.file_stem().unwrap();
        let filename = temp.to_str().unwrap();
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