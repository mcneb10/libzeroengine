use crate::ucfb::*;
/// Object that reperesents a chunk of audio data
/// INCOMPLETE
#[derive(Debug, Clone)]
pub struct AudioData {}

// According to Psych0fred himself,
// the audio data format is 16 Bit Stereo 48Khz PCM for music
// sounds could be a variety of formats

/// Errors returned by AudioData
#[derive(Debug, Clone, Copy)]
pub enum AudioDataError {
    /// Chunk does not store audio data
    NotAAudioDataChunk,
}

impl AudioData {
    /// Deserialize audio data from chunk
    pub fn from_chunk(chunk: Chunk) -> Result<Self, AudioDataError> {
        if chunk.header.name.as_bytes() != b"\x5C\xD9\xA0\x23" {
            return Err(AudioDataError::NotAAudioDataChunk);
        }
        Ok(AudioData {})
    }
}
