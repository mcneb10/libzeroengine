use nom::{
    bytes::complete::tag, combinator::map, multi::count, number::streaming::*, sequence::tuple,
    IResult, ToUsize,
};
use std::fs::File;
use std::io::SeekFrom;
use std::{fmt::Debug, io::prelude::*};

use crate::mvs::Movie;
use crate::script::Script;

/// This object represents the ucfb file
#[derive(Debug, Clone)]
pub struct UCFBFile {
    /// The ucfb header
    pub header: UCFBHeader,
    /// List of chunks in the ucfb file
    pub chunks: Vec<Chunk>,
}

/// Header for ucfb file
#[derive(Debug, Clone)]
pub struct UCFBHeader {
    /// Size of ucfb file
    pub size: u32,
}

/// Header for ucfb chunk
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    /// Chunk name
    pub name: String,
    /// Chunk size
    pub size: u32,
}

/// A container for an object that stores information about a deciphered chunk
#[derive(Debug, Clone)]
pub enum DecipheredChunk {
    /// Chunk that represents a lua script
    Script(Script),
    /// Chunk that represents a bink movie
    Movie(Movie),
    /// Chunk that represents embedded ucfb
    UCFB(UCFBFile),
}

/// ucfb chunk
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Chunk header
    pub header: ChunkHeader,
    /// Chunk data
    pub data: Vec<u8>,
    /// Class representing the chunk data
    pub deciphered_chunk: Option<DecipheredChunk>,
}

/// Error returned by this namespace
#[derive(Debug, Clone, Copy)]
pub enum UCFBError {
    /// File is too small to parse
    FileTooSmall,
    /// Size is specified incorrectly in header
    WrongHeaderSize,
    /// File is not valid ucfb file
    NotAUCFBFile,
    /// Failure reading file
    IOError,
    /// Failure to parse chunk file
    InvalidChunkName,
    /// Failure during alignment
    BadAlignment,
}

fn parse_header(input: &[u8]) -> IResult<&[u8], UCFBHeader> {
    map(tuple((tag("ucfb"), le_u32)), |(_, s)| UCFBHeader {
        size: s,
    })(input)
}

fn parse_chunk_header(input: &[u8]) -> IResult<&[u8], ChunkHeader> {
    map(tuple((count(le_u8, 4), le_u32)), |(n, s)| ChunkHeader {
        name: n.iter().map(|&c| char::from(c)).collect(), // Convert the bytes to ascii and not utf8
        size: s,
    })(input)
}

fn align_file_pointer(f: &mut File) -> Result<(), UCFBError> {
    let offset = 4 - i64::try_from(
        match f.stream_position() {
            Ok(v) => v,
            Err(_) => return Err(UCFBError::BadAlignment),
        } % 4,
    )
    .unwrap();
    // Don't align if aligned already
    if offset != 4 {
        if f.seek(SeekFrom::Current(offset)).is_err() {
            return Err(UCFBError::IOError);
        }
    }
    Ok(())
}

/// Extract chunks from a file with the file pointer advanced to the start of the chunks
pub fn extract_chunks(file: &mut File) -> Result<Vec<Chunk>, UCFBError> {
    let mut current_chunk_header: ChunkHeader;
    let mut chunks: Vec<Chunk> = vec![];
    let mut temp_chunk_data: Vec<u8>;
    let mut buffer: Vec<u8> = vec![0; 8];
    // Parse out the chunks
    // read in these steps: Read header, read data, align on 4 bytes, repeat
    while match file.read(&mut buffer) {
        Ok(v) => v,
        Err(_) => return Err(UCFBError::FileTooSmall),
    } == buffer.len()
    {
        (_, current_chunk_header) = match parse_chunk_header(&mut buffer) {
            Ok(v) => (v.0.to_vec(), v.1),
            Err(e) => {
                println!("Error: {}", e);
                return Err(UCFBError::NotAUCFBFile);
            }
        };
        temp_chunk_data = vec![0; current_chunk_header.size.to_usize()];
        match file.read(&mut temp_chunk_data) {
            Err(_) => return Err(UCFBError::BadAlignment),
            _ => (), // Otherwise do nothing
        };
        chunks.push(Chunk {
            header: current_chunk_header,
            data: temp_chunk_data.to_vec(),
            deciphered_chunk: None,
        });
        // align by 4 bytes
        align_file_pointer(file)?;
    }
    Ok(chunks)
}

/// Extract chunks from a byte array of chunks
pub fn extract_chunks_bytearray(buffer: &mut Vec<u8>) -> Result<Vec<Chunk>, UCFBError> {
    let mut current_chunk_header: ChunkHeader;
    let mut chunks: Vec<Chunk> = vec![];
    let mut chunk_data: Vec<u8>;
    // Parse out the chunks
    // read in these steps: Read header, read data, align on 4 bytes, repeat
    while buffer.len() > 0 {
        (*buffer, current_chunk_header) = match parse_chunk_header(buffer) {
            Ok(v) => (v.0.to_vec(), v.1),
            Err(e) => {
                println!("Error: {}", e);
                return Err(UCFBError::NotAUCFBFile);
            }
        };
        chunk_data = buffer
            .drain(0..current_chunk_header.size.to_usize())
            .collect();
        chunks.push(Chunk {
            header: current_chunk_header,
            data: chunk_data,
            deciphered_chunk: None,
        });
        // align by 4 bytes
        match buffer.len() % 4 {
            0 | 4 => {}
            _ => {
                buffer.drain(0..(buffer.len() % 4));
                ()
            }
        }
    }
    Ok(chunks)
}

impl UCFBFile {
    /// Create a new Script object from a file
    pub fn new(file_name: String) -> Result<Self, UCFBError> {
        let mut le_file = match File::open(file_name) {
            Ok(v) => v,
            Err(_) => return Err(UCFBError::IOError),
        };
        let mut buffer: Vec<u8> = vec![0; 8];
        let header: UCFBHeader;
        if le_file.read(&mut buffer).is_err() {
            return Err(UCFBError::IOError);
        }
        (_, header) = match parse_header(&mut buffer) {
            Ok(v) => (v.0.to_vec(), v.1),
            Err(_) => return Err(UCFBError::NotAUCFBFile),
        };
        if header.size < 8 {
            return Err(UCFBError::FileTooSmall);
        } else if header.size == 8 {
            // The file is empty
            return Ok(UCFBFile {
                header: header,
                chunks: vec![],
            });
        }
        if le_file.metadata().unwrap().len() - 8 != header.size.into() {
            return Err(UCFBError::WrongHeaderSize);
        }

        Ok(UCFBFile {
            header: header,
            chunks: extract_chunks(&mut le_file)?,
        })
    }
    /// Try to figure out what the data stored in the chunks is and parse if possible
    pub fn visit_chunks(&mut self) {
        for chunk in &mut self.chunks {
            chunk.deciphered_chunk = match chunk.header.name.as_str() {
                // TODO: error handling
                "scr_" => Some(DecipheredChunk::Script(
                    Script::from_chunk(chunk.clone()).unwrap(),
                )),
                "\x60\x70\x1F\x2F" => Some(DecipheredChunk::Movie(
                    Movie::from_chunk(chunk.clone()).unwrap(),
                )),
                "ucfb" => Some(DecipheredChunk::UCFB(UCFBFile {
                    header: UCFBHeader {
                        size: chunk.header.size,
                    },
                    chunks: extract_chunks_bytearray(&mut chunk.data).unwrap(),
                })),
                _ => {
                    println!("Unknown chunk type: {}", chunk.header.name);
                    None
                } // TODO: make the output controllable by the program
            }
        }
    }
}
