//! A parser for the various binary filetypes used by Pandemic's ZeroEngine
//!
//! ZeroEngine was used to make games such as the original Star Wars Battlefront games
//!
//! [explanation of supported file formats]
//! [code examples]

#![deny(missing_docs)]
/// Module representing a level
pub mod lvl;
/// Module representing a in-game cutscene (mvs)
pub mod mvs;
/// Module representing a lua script from ZeroEngine
pub mod script;
/// Module representing a texture
pub mod tex;
/// Module representing a ucfb file from ZeroEngine
pub mod ucfb;
