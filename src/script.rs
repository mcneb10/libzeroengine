use crate::ucfb::*;
use lunify::{unify, Format, InstructionLayout, OperandType, Settings};

/// This object represents the addme.script file
/// 'Tis a wrapper above `ucfb` that also decompiles the lua code
#[derive(Debug, Clone)]
pub struct Script {
    /// The name of the script
    pub name: String,
    /// I don't know
    pub info: u8,
    /// Lua bytecode
    pub body: Vec<u8>,
}

/// Errors returned by Script
#[derive(Debug, Clone, Copy)]
pub enum ScriptError {
    /// Error during parsing chunk
    ChunkParseError,
    /// Chunk doesn't have script magic
    NotAScript,
}

impl Script {
    /// Deserialize script from chunk
    pub fn from_chunk(chunk: Chunk) -> Result<Self, ScriptError> {
        if chunk.header.name != "scr_" {
            return Err(ScriptError::NotAScript);
        }
        let mut data = chunk.data.clone();
        let subchunks: Vec<Chunk> = match extract_chunks_bytearray(&mut data) {
            Ok(v) => v,
            Err(_) => return Err(ScriptError::ChunkParseError),
        };
        let mut body = subchunks.get(2).unwrap().data.clone();
        // There is a trailing null byte after the data that must be removed
        body.pop();
        Ok(Script {
            // Remove all null bytes after extracting name
            name: String::from_utf8(subchunks.get(0).unwrap().data.clone())
                .unwrap()
                .replace('\0', ""),
            info: *subchunks.get(1).unwrap().data.get(0).unwrap(),
            body: body,
        })
    }
    /// Convert the lua 5.0 bytecode to lua 5.1 so it can be decompiled
    pub fn get_lua_51_bytecode_from_50(&self) -> Vec<u8> {
        let bytecode: &[u8] = &self.body;
        let fmt = Format {
            endianness: lunify::Endianness::Little,
            size_t_width: lunify::BitWidth::Bit64,
            ..Format::default()
        };
        let bytecode_settings: Settings = lunify::Settings {
            lua50: lunify::lua50::Settings {
                stack_limit: 128,
                fields_per_flush: 32,
                binary_signature: "\x1bLua",
                layout: InstructionLayout::from_specification([
                    OperandType::Opcode(6),
                    OperandType::C(9),
                    OperandType::B(9),
                    OperandType::A(8),
                ])
                .unwrap(),
            },
            lua51: Default::default(),
            output: Default::default(),
        };
        // TODO: error handling
        // TODO: actually made it work by reimplementing this function but injecting lineinfo
        unify(bytecode, &fmt, &bytecode_settings).unwrap()
    }
    /// Decompile the bytecode, internally converting to lua 5.1 first
    pub fn decompile_bytecode(&self) {
        let bytecode = self.get_lua_51_bytecode_from_50();
        // TODO: decompile the new bytecode
    }
}
