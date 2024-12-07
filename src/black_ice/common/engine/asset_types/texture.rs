
use crate::{black_ice::common::engine::{asset_mgr::*, asset_types::*, pipeline::Image}, Env};
use core::{error, panic};
use std::{collections::HashMap, io::{BufRead, BufReader, Cursor, Error, Read, SeekFrom}, string::ParseError, sync::Arc};
use image::GenericImageView;
use parking_lot::Mutex;
use sdl2::render;

pub struct Texture {

    image_data: Option<Arc<Mutex<Image>>>,// we will store this in the render server if we have loaded it!!
    // sometimes, out image may be stored in an atlas, so we may be referencing a single image shared my many other Textures
    // that may be updated during runtime

    pub asset_path: String,

}

impl Texture {

    pub fn parse_png(data: &Vec<u8>) -> Result<(Vec<[u32; 4]>, u32, u32), Error> {

        let mut mage = image::ImageReader::new(Cursor::new(data.as_slice()));
        mage.set_format(image::ImageFormat::Png);
        let mut decoded = mage.decode().expect("Failed to read image data. PNG may be corrupted!! Please Report!");
        let mut buf = decoded.to_rgba16().into_iter().map(|x| {u32::from(x.clone())}).collect::<Vec<u32>>();
        let buf_2 = buf.chunks_exact(4).map(|chunk| {<[u32; 4]>::try_from(chunk).unwrap()}).collect::<Vec<[u32; 4]>>();
        let result: (Vec<[u32; 4]>, u32, u32) = (buf_2, decoded.width(), decoded.height());
        // // lets check the first 8 bytes for a valid png
        // if !(data[0..7] == [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n']){
        //     // we have a non png file!!
        //     return Err(Error::new(std::io::ErrorKind::InvalidData, "This Image is not a valid png file. Please check if it has been corrupted"));
        // }

        // let mut p: usize = 8;
        // let mut eof = false;
        // let mut chunks: Vec<(String, usize, Vec<u8>)>= Vec::new();
        // let mut result: (Vec<(u32, u32, u32, u32)>, u32, u32) = (vec![], 0 ,0);
        // while !eof {
        //     let mut length_buf: [u8; 4] = [0; 4];
        //     length_buf.copy_from_slice(&data[p..p+4]);
        //     let length = usize::try_from(u32::from_be_bytes(length_buf)).unwrap();.unwrap()
        //     let mut type_buf: [u8; 4] = [0; 4];
        //     type_buf.copy_from_slice(&data[p+4..p+8]);
        //     let chunk_type = String::from_utf8(type_buf.to_vec()).unwrap();

        //     // now we copy the chunk data
        //     let mut chunk_data: Vec<u8> = Vec::<u8>::new();
        //     chunk_data.copy_from_slice(&data[p+8..p+length]);

        //     let mut crc_buf: [u8; 4] = [0; 4];
        //     crc_buf.copy_from_slice(&data[p+length..p+length+4]);

        //     p += length + 12;

        //     chunks.push((chunk_type, length, chunk_data));
        //     if p + 12 > data.len() {
        //         eof = true;
        //     }
            
        // }
        // // now we should have our chunks, we must go through each one of them in order to get the image data
        // let mut height: u32 = 0;
        // let mut width: u32 = 0;
        // let mut colour_depth: u8 = 0;
        // let mut colour_type: u8 = 0;
        // let mut pallete: Vec<[u8; 3]> = vec![];
        // let mut data_chunks: Vec<(u8, Vec<u8>)> = vec![];
        // let mut uncompressed_data:Vec<u8> = vec![];
        // let mut background_colour: [u32; 3] = [0; 3];
        // for (chunk_type, size, data) in chunks {
        //     match chunk_type.as_str() {
        //         "IHDR" => {
        //             let mut buf_1: [u8; 4] = [0; 4];
        //             buf_1.copy_from_slice(&data[0..4]);
        //             width = u32::from_be_bytes(buf_1);
        //             let mut buf_2: [u8; 4] = [0; 4];
        //             buf_2.copy_from_slice(&data[4..8]);
        //             height = u32::from_be_bytes(buf_2);
        //             colour_depth = data[8].clone();
        //             colour_type = data[9].clone();

        //         },
        //         "PLTE" => {
        //             for i in (0..(size)).step_by(3){
        //                 let mut buf: [u8; 3] = [0;3];
        //                 buf.copy_from_slice(&data[i..i+3]);
        //                 pallete.push(buf);
        //             }

        //         },
        //         "IDAT" => {
        //             let filter: u8 = data[0];
        //             let chunk_image_data = data[1..].to_vec();
        //             data_chunks.push((filter, chunk_image_data));

                    
        //         },
        //         "IEND" => {

        //         },
        //         "bKGD" => {
        //             match colour_type {
        //                 3 => {
        //                     // pallete colour!!
        //                     let c = usize::try_from(data[0]).unwrap();
        //                     let value = pallete[c].clone();
        //                     background_colour.copy_from_slice(&[
        //                         u32::try_from(value[0]).unwrap(),
        //                         u32::try_from(value[1]).unwrap(),
        //                         u32::try_from(value[2]).unwrap()
        //                     ]);
        //                 },
        //                 0 | 4 => {
        //                     let mut buf: [u8; 2] = [0; 2];
        //                     buf.copy_from_slice(&data[0..]);
        //                     let grey_value = u32::try_from(u16::from_be_bytes(buf)).unwrap();
        //                     background_colour.copy_from_slice(&[grey_value, grey_value, grey_value]);
        //                 },
        //                 2 | 6 => {
        //                     let mut buf_r: [u8; 2] = [0; 2];
        //                     let mut buf_g: [u8; 2] = [0; 2];
        //                     let mut buf_b: [u8; 2] = [0; 2];
        //                     buf_r.copy_from_slice(&data[0..2]);
        //                     buf_g.copy_from_slice(&data[2..4]);
        //                     buf_b.copy_from_slice(&data[4..]);
        //                     let r = u32::try_from(u16::from_be_bytes(buf_r)).unwrap();
        //                     let g = u32::try_from(u16::from_be_bytes(buf_g)).unwrap();
        //                     let b = u32::try_from(u16::from_be_bytes(buf_b)).unwrap();
        //                     background_colour.copy_from_slice(&[r,g,b]);
        //                 }
        //                 _ => {}
        //             }
        //         },
        //         "cHRM" => {

        //         },
        //         "cICP" => {

        //         }
        //         "gAMA" => {

        //         },
        //         "pHYs" => {

        //         },
        //         "tRNS" => {

        //         },

        //         _ => {}
        //     }
        // }
        // result.1 = width;
        // result.2 = height;

        // // we need to now decompress the image and apply any filters
        // let mut decomp = flate2::Decompress::new(false);
        // let mut decomp_data_chunks = Vec::<(u32, Vec<u8>)>::new();
        // for (filter, data) in data_chunks {
        //     let mut decomp_data = Vec::<u8>::new();
        //     decomp.decompress_vec(&data, &mut decomp_data, flate2::FlushDecompress::Finish).expect("Failed to decompress chunk!!");
        //     // now filter

        // }

        return Ok(result);
    }


}

impl AssetResource for Texture {
    fn new() -> Self {
        Texture { image_data:None, asset_path: "".to_string() }
    }

    fn init(&mut self, data: std::sync::Arc<AssetData>){
        // the asset data we load in will either be fresh, or already loaded by our render server
        // first we should check with the render server first
        unsafe {
            // this step will be unsafe, so be careful!!
            let p_render_sys = Env::get_render_sys();
            let mut render_sys = p_render_sys.write();

            // lets start processing the image data
            // this will be dropped when we finish loading the image
            let im_data = &data.data;

            if let Ok(image) = render_sys.find_image(data.asset_path.clone()) {
                self.image_data = Some(image);
                self.asset_path = data.asset_path.clone();
                return;
            }
            else {
                let mut mage = image::ImageReader::new(Cursor::new(im_data.as_slice()));
                let format = image::ImageFormat::from_extension(data.metadata.get("ext").unwrap()).expect("Failed to find image format!!");
                mage.set_format(format);
                let decoded = mage.decode().expect("Failed to read image data. PNG may be corrupted!! Please Report!");
                let buf = decoded.to_rgba16().into_iter().map(|x| {u32::from(x.clone())}).collect::<Vec<u32>>();
                let buf_2 = buf.chunks_exact(4).map(|chunk| {<[u32; 4]>::try_from(chunk).unwrap()}).collect::<Vec<[u32; 4]>>();
                let image = render_sys.register_image(&buf_2, decoded.width(), decoded.height(), 32, data.asset_path.clone());
                self.image_data = Some(image);
                self.asset_path = data.asset_path.clone();
            }

        }
    }

    fn update(&mut self) -> Result<OutputData, AssetResourceUpdateError> {

        Ok(OutputData::BYTEARRAY(vec![]))
    }

    fn unload(&mut self){

    }
}
