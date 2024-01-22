//! A parser for the various binary filetypes and containers used by Pandemic's ZeroEngine
//!
//! ZeroEngine was used by Pandemic to make games such as the original Star Wars Battlefront games
//!
//! [explanation of supported file formats]
//! [code examples]

#![deny(missing_docs)]
/// Module representing audio data
pub mod audio_data;
/// Module representing a level
pub mod lvl;
/// Module representing a in-game cutscene (mvs)
pub mod mvs;
/// Module representing all game object property chunks
pub mod prop;
/// Module representing a lua script from ZeroEngine
pub mod script;
/// Module representing a texture
pub mod tex;
/// Module representing a ucfb file from ZeroEngine
pub mod ucfb;
