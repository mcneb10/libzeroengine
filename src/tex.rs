use crate::ucfb::*;
use bincode::deserialize;
use ddsfile::{D3DFormat, Dds, FourCC, Header};
use image_dds::image::EncodableLayout;
use serde::Deserialize;

// TODO: sort this out with INFO chunks that are not exactly 16 bytes in size
#[derive(Debug, Deserialize)]
struct TextureHeader {
    format: u32,
    width: u16,
    height: u16,
    depth: u16,
    mipmap_count: u16,
    _detail_bias: u32,
}

impl TextureHeader {
    fn find_format(format: u32) -> Result<D3DFormat, HeaderError> {
        Ok(match format {
            // TODO: submit pull request to ddsfile to add function so I don't have to do this
            // TODO: also add all the formats that zeroengine games use
            FourCC::DXT1 => D3DFormat::DXT1,
            23 => D3DFormat::R5G6B5,
            _ => return Err(HeaderError::InvalidFourCC(format)),
        })
    }
    fn to_dds_header(&self, format: D3DFormat) -> Result<Header, HeaderError> {
        Ok(
            match Header::new_d3d(
                self.height as u32,
                self.width as u32,
                Some(self.depth as u32),
                format,
                Some(self.mipmap_count as u32),
                None,
            ) {
                Ok(v) => v,
                Err(e) => return Err(HeaderError::OtherError(e)),
            },
        )
    }
}

/// Object that represents a singular texture
#[derive(Debug, Clone)]
pub struct Texture {
    /// The texture format
    pub format: FourCC,
    /// The raw data
    pub data: Vec<u8>,
}

/// Object that temporarily contains data required to instantiate `Dds`
/// This is necessary because Dds is not cloneable
#[derive(Debug, Clone)]
struct DdsTemporaryInformationStorageObject {
    /// The dds header
    header: Header,
    /// The dds pixel data
    data: Vec<u8>,
}

/// Object that represents a texture container
#[derive(Debug, Clone)]
pub struct TextureContainer {
    /// The texture name
    pub name: String,
    /// List of textures
    formats: Vec<DdsTemporaryInformationStorageObject>,
}

/// Errors produced during header parsing
#[derive(Debug)]
pub enum HeaderError {
    /// FourCC is not recognized.
    /// Some files may cause this error as all formats are not implemented yet
    InvalidFourCC(u32),
    /// Some other error happened during parsing the header
    OtherError(ddsfile::Error),
}

/// Errors produced by this class
#[derive(Debug)]
pub enum TextureError {
    /// Chunk is not a texture
    NotATexture,
    /// Error parsing dds header
    HeaderError(HeaderError),
    /// Error parsing chunk
    ChunkParseError(UCFBError),
    /// Error during parsing of DDS info
    TextureInfoHeaderParseFailure(bincode::Error),
    /// General texture parsing error
    TextureParseError,
}

impl TextureContainer {
    // TODO: error handling
    /// Get texture from chunk
    pub fn from_chunk(chunk: Chunk) -> Result<Self, TextureError> {
        // Warning: this format is idiotic and whoever devised it is too
        if chunk.header.name != "tex_" {
            return Err(TextureError::NotATexture);
        }
        let subchunks = match extract_chunks_bytearray(&mut chunk.data.clone()) {
            Ok(v) => v,
            Err(e) => return Err(TextureError::ChunkParseError(e)),
        };
        // NAME chunk
        let mut name: String = match subchunks.get(0) {
            Some(v) => v,
            None => return Err(TextureError::TextureParseError),
        }
        .data
        .iter()
        .map(|&c| char::from(c))
        .collect();
        name = name.replace("\0", "");
        // INFO chunk
        let format_chunk_data = match subchunks.get(1) {
            Some(v) => v,
            None => return Err(TextureError::TextureParseError),
        }
        .data
        .clone();
        let format_count = u32::from_le_bytes(
            match (match format_chunk_data.get(0..4) {
                Some(v) => v,
                None => return Err(TextureError::TextureParseError),
            })
            .try_into()
            {
                Ok(v) => v,
                Err(_) => return Err(TextureError::TextureParseError),
            },
        );
        let mut formats: Vec<DdsTemporaryInformationStorageObject> = vec![];
        // Read the formats
        for i in 0..format_count {
            let texture_format_chunk = match subchunks.get(2 + i as usize) {
                Some(v) => v,
                None => return Err(TextureError::TextureParseError),
            };
            let texture_format_subchunks =
                match extract_chunks_bytearray(&mut texture_format_chunk.data.clone()) {
                    Ok(v) => v,
                    Err(e) => return Err(TextureError::ChunkParseError(e)),
                };
            // INFO chunk (format info)
            /*let info = match texture_format_subchunks.get(0) {
                Some(v) => v,
                None => return Err(TextureError::TextureParseError),
            }
            .data.clone();*/
            // FACE chunk
            let face = match texture_format_subchunks.get(1) {
                Some(v) => v,
                None => return Err(TextureError::TextureParseError),
            }
            .data
            .clone();
            let face_subchunks = match extract_chunks_bytearray(&mut face.clone()) {
                Ok(v) => v,
                Err(e) => return Err(TextureError::ChunkParseError(e)),
            };
            // FACE.LVL_ chunk
            let lvl_subchunks = match extract_chunks_bytearray(
                &mut match face_subchunks.get(0) {
                    Some(v) => v,
                    None => return Err(TextureError::TextureParseError),
                }
                .data
                .clone(),
            ) {
                Ok(v) => v,
                Err(e) => return Err(TextureError::ChunkParseError(e)),
            };
            // FACE.LVL_.INFO chunk (more format info)
            let info2: TextureHeader = match deserialize(
                match texture_format_subchunks.get(0) {
                    Some(v) => v,
                    None => return Err(TextureError::TextureParseError),
                }
                .data
                .as_bytes(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    //return Err(TextureError::TextureInfoHeaderParseFailure(e));
                    // Skip, should add a log message here
                    //println!("Skipped bc of err: {:?}", e);
                    continue;
                }
            };
            // FACE.LVL_.BODY chunk (texture data)
            let body = lvl_subchunks.get(1).unwrap().data.clone();
            let format_of_the_format: D3DFormat = match TextureHeader::find_format(info2.format) {
                Ok(v) => v,
                Err(e) => {
                    // Skip, should add a log message here
                    //println!("Skipped bc of err: {:?}", e);
                    continue;
                }
            };

            formats.push(DdsTemporaryInformationStorageObject {
                header: match info2.to_dds_header(format_of_the_format) {
                    Ok(v) => v,
                    Err(e) => return Err(TextureError::HeaderError(e)),
                },
                data: body,
            });
        }
        Ok(TextureContainer {
            name: name,
            formats: formats,
        })
    }

    // TODO: send pull request to ddsfile to make Dds cloneable?
    /// Convert the internal temporary storage object to Dds objects
    pub fn get_formats_dds_vec(&self) -> Vec<Dds> {
        let mut formats: Vec<Dds> = vec![];

        for format in self.formats.clone() {
            formats.push(Dds {
                header: format.header,
                header10: None,
                data: format.data,
            });
        }

        return formats;
    }
}
