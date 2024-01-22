use crate::ucfb::*;
use bincode::deserialize;
use ddsfile::{D3DFormat, Dds, FourCC, Header};
use image_dds::image::EncodableLayout;
use serde::Deserialize;

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
            20 => D3DFormat::R8G8B8,
            21 => D3DFormat::A8R8G8B8,
            22 => D3DFormat::X8R8G8B8,
            23 => D3DFormat::R5G6B5,
            24 => D3DFormat::X1R5G5B5,
            25 => D3DFormat::A1R5G5B5,
            26 => D3DFormat::A4R4G4B4,
            FourCC::DXT2 => D3DFormat::DXT2,
            FourCC::DXT3 => D3DFormat::DXT3,
            FourCC::DXT4 => D3DFormat::DXT4,
            FourCC::DXT5 => D3DFormat::DXT5,
            FourCC::R8G8_B8G8 => D3DFormat::R8G8_B8G8,
            FourCC::G8R8_G8B8 => D3DFormat::G8R8_G8B8,
            FourCC::A16B16G16R16 => D3DFormat::A16B16G16R16,
            FourCC::Q16W16V16U16 => D3DFormat::Q16W16V16U16,
            FourCC::R16F => D3DFormat::R16F,
            FourCC::G16R16F => D3DFormat::G16R16F,
            FourCC::A16B16G16R16F => D3DFormat::A16B16G16R16F,
            FourCC::R32F => D3DFormat::R32F,
            FourCC::G32R32F => D3DFormat::G32R32F,
            FourCC::A32B32G32R32F => D3DFormat::A32B32G32R32F,
            FourCC::UYVY => D3DFormat::UYVY,
            FourCC::YUY2 => D3DFormat::YUY2,
            FourCC::CXV8U8 => D3DFormat::CXV8U8,
            _ => return Err(HeaderError::InvalidFourCC(format)),
        })
    }
    fn to_dds_header(&self, format: D3DFormat) -> Result<Header, HeaderError> {
        Ok(Header::new_d3d(
            self.height as u32,
            self.width as u32,
            Some(self.depth as u32),
            format,
            Some(self.mipmap_count as u32),
            None,
        )
        .map_err(|e| HeaderError::OtherError(e))?)
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
        let subchunks = extract_chunks_bytearray(&mut chunk.data.clone())
            .map_err(|e| TextureError::ChunkParseError(e))?;
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
            (match format_chunk_data.get(0..4) {
                Some(v) => v,
                None => return Err(TextureError::TextureParseError),
            })
            .try_into()
            .map_err(|_| TextureError::TextureParseError)?,
        );
        let mut formats: Vec<DdsTemporaryInformationStorageObject> = vec![];
        // Read the formats
        for i in 0..format_count {
            let texture_format_chunk = match subchunks.get(2 + i as usize) {
                Some(v) => v,
                None => return Err(TextureError::TextureParseError),
            };
            let texture_format_subchunks =
                extract_chunks_bytearray(&mut texture_format_chunk.data.clone())
                    .map_err(|e| TextureError::ChunkParseError(e))?;
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
            let face_subchunks = extract_chunks_bytearray(&mut face.clone())
                .map_err(|e| TextureError::ChunkParseError(e))?;
            // FACE.LVL_ chunk
            let lvl_subchunks = extract_chunks_bytearray(
                &mut match face_subchunks.get(0) {
                    Some(v) => v,
                    None => return Err(TextureError::TextureParseError),
                }
                .data
                .clone(),
            )
            .map_err(|e| TextureError::ChunkParseError(e))?;
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
                header: info2
                    .to_dds_header(format_of_the_format)
                    .map_err(|e| TextureError::HeaderError(e))?,
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
