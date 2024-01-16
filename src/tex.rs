/*
TODO: this is annoying
use crate::ucfb::*;
use bincode::deserialize;
use ddsfile::{Dds, FourCC, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct TextureHeader {
    format: u32,
    width: u16,
    height: u16,
    depth: u16,
    mipmap_count: u16,
    detail_bias: u32,
}

impl TextureHeader {
    fn to_dds_header(&self, format: FourCC) -> Header {
        Header::new_d3d(self.height, self.width, self.depth, , self.mipmap_count, None //TODO?);
    }
}

/// Types of textures contained in the `tex_` chunk
#[derive(Debug, Clone, Copy)]
pub enum TextureType {}

/// Object that represents a singular texture
#[derive(Debug, Clone)]
pub struct Texture {
    /// The texture format
    pub format: FourCC,
    /// The raw data
    pub data: Vec<u8>,
}

/// Object that represents a texture container
#[derive(Debug)]
pub struct TextureContainer {
    /// The texture name
    pub name: String,
    /// List of textures
    pub formats: Vec<Dds>,
}

/// Errors produced by this class
#[derive(Debug, Clone, Copy)]
pub enum TextureError {
    NotATexture,
}

impl TextureContainer {
    fn make_dds(format: FourCC, data: Vec<u8>) -> Dds {
        Dds {
            header: Header {},
            header10: None,
            data: data,
        }
    }
    pub fn from_chunk(chunk: Chunk) -> Result<Self, TextureError> {
        // Warning: this format is idiotic and whoever devised it is too
        if chunk.header.name != "tex_" {
            return Err(TextureError::NotATexture);
        }
        let subchunks = extract_chunks_bytearray(&mut chunk.data)?;
        // NAME chunk
        let name = subchunks
            .get(0)?
            .data
            .iter()
            .map(|&c| char::from(c))
            .collect();
        // INFO chunk
        let format_chunk_data = subchunks.get(1)?.data;
        let format_count = u32::from_le_bytes(format_chunk_data.get(0..4)?.try_into()?);
        let mut formats: Vec<Dds> = vec![];
        // Read the formats
        for i in 0..format_count {
            let mut texture_format_chunk = subchunks.get(2 + i as usize)?;
            let texture_format_subchunks =
                extract_chunks_bytearray(&mut texture_format_chunk.data)?;
            // INFO chunk (format info)
            let info = texture_format_subchunks.get(0).unwrap().data;
            // FACE chunk
            let face = texture_format_subchunks.get(1).unwrap().data;
            let face_subchunks = extract_chunks_bytearray(&mut face).unwrap();
            // FACE.LVL_ chunk
            let lvl_subchunks =
                extract_chunks_bytearray(&mut face_subchunks.get(0).unwrap().data).unwrap();
            // FACE.LVL_.INFO chunk (more format info)
            let info2 = lvl_subchunks.get(0).unwrap().data;
            // FACE.LVL_.BODY chunk (texture data)
            let body = lvl_subchunks.get(0).unwrap().data;

            //let format: FourCC = FourCC(u32::from_le_bytes(format_chunk_data.get(offset..offset+4)?.try_into()?));
            formats.push(Self::make_dds(format, body));
        }
        Ok(TextureContainer {
            name: name,
            formats: formats,
        })
    }
}
*/
