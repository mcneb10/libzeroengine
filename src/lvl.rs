use crate::ucfb::*;
/// Object that reperesents a level
#[derive(Debug, Clone)]
pub struct Level {
    /// Chunks contained in level
    pub chunks: Vec<Chunk>,
}

/// Errors returned by Movie
#[derive(Debug)]
pub enum LevelError {
    /// Invalid Magic
    NotALevel,
    /// Chunk Read failure
    ChunkError(UCFBError),
}

impl Level {
    /// Deserialize level from chunk
    pub fn from_chunk(chunk: Chunk) -> Result<Self, LevelError> {
        if chunk.header.name != "lvl_" {
            return Err(LevelError::NotALevel);
        }
        let mut data = chunk.data.clone();
        // Remove stuff we don't need
        data.drain(0..8);
        let level_chunks = match extract_chunks_bytearray(&mut data) {
            Ok(v) => v,
            Err(e) => return Err(LevelError::ChunkError(e)),
        };
        Ok(Level {
            chunks: level_chunks,
        })
    }
}
