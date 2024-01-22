use crate::ucfb::*;
/// Object that reperesents an in-game cutscene
#[derive(Debug, Clone)]
pub struct Movie {
    /// A list of bink cutscene files
    pub bink_files: Vec<Vec<u8>>,
}

// TODOs: .bik to regular video conversion with ffmpeg

/// Errors returned by Movie
#[derive(Debug, Clone, Copy)]
pub enum MovieError {
    /// Error during parsing chunk
    ChunkParseError,
    /// Movie doesn't have correct magic
    NotAMovie,
    /// Movie is corrupted
    CorruptedMovie,
}

impl Movie {
    /// Deserialize movie from chunk
    pub fn from_chunk(chunk: Chunk) -> Result<Self, MovieError> {
        if chunk.header.name != "\x60\x70\x1F\x2F" {
            return Err(MovieError::NotAMovie);
        }
        let mut data = chunk.data.clone();
        // Jar Jar
        let mut binks: Vec<Vec<u8>> = vec![];
        // The structure of these files is odd
        // There seems to be a bink file at offsets that are multiples of 0x800 from the beginning of the MVS file
        // 0x7F0 offset from the beginning of the chunk
        // 0x2F0 of it is padding and 0x500 is some header
        let mut offset = 0;
        data.drain(0..0x7F0);
        while data.len() > offset {
            // Try to find the file
            // Should land on it immediately the first time
            let sig = match data.get(offset..offset + 3) {
                Some(v) => v,
                None => break,
            };
            if "BIK".as_bytes() != sig {
                offset += 4;
                continue;
            }
            // Extract the bik
            let bik_size = u32::from_le_bytes(match data.get(offset + 4..offset + 8) {
                Some(v) => v.try_into().unwrap(),
                None => continue,
            }) + 8;
            binks.push(
                data.get(offset..(offset + (bik_size as usize) + 7))
                    .ok_or(MovieError::CorruptedMovie)?
                    .to_vec(),
            );
            offset += (bik_size as usize) + 8;
        }
        Ok(Movie { bink_files: binks })
    }
}
