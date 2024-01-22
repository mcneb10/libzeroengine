use std::{collections::HashMap, ffi::CStr};

use crate::prop::constants::*;
use crate::ucfb::*;

mod constants;

/// Possible property container types
#[derive(Debug, Clone)]
pub enum PropertyContainerTypes {
    /// Game Object
    GameObjectClass,
    /// Explosion Object
    ExplosionClass,
    /// Ordnance Object
    OrdnanceClass,
    /// Weapon Object
    WeaponClass,
}

/// Object that reperesents an in-game property container (odf)
#[derive(Debug, Clone)]
pub struct PropertyContainer {
    /// Property Container/Class Type
    pub r#type: PropertyContainerTypes,
    /// The properties
    pub properties: HashMap<String, String>,
    /// The class name
    pub name: String,
    /// One of these properties will be populated
    ///
    /// The class label
    pub class_label: Option<String>,
    /// The class parent
    pub class_parent: Option<String>,
}

/// Errors returned by prop
#[derive(Debug)]
pub enum PropertyError {
    /// Error during parsing chunk
    ChunkParseError(UCFBError),
    /// Property doesn't have correct magic
    NotAProperty,
    /// Property is corrupted
    CorruptedProperty,
}

impl PropertyContainer {
    /// Deserialize class from chunk
    pub fn from_chunk(chunk: Chunk) -> Result<Self, PropertyError> {
        let r#type: PropertyContainerTypes = match chunk.header.name.as_str() {
            "entc" => PropertyContainerTypes::GameObjectClass,
            "expc" => PropertyContainerTypes::ExplosionClass,
            "ordc" => PropertyContainerTypes::OrdnanceClass,
            "wpnc" => PropertyContainerTypes::WeaponClass,
            _ => return Err(PropertyError::NotAProperty),
        };
        let subchunks = extract_chunks_bytearray(&mut chunk.data.clone())
            .map_err(|e| PropertyError::ChunkParseError(e))?;

        // The chunk name doesn't really matter, as the chunks are always in a specific order
        // BASE chunk
        let base_chunk = match subchunks.get(0) {
            Some(v) => v,
            None => return Err(PropertyError::CorruptedProperty),
        };
        // TYPE chunk
        let type_chunk = match subchunks.get(1) {
            Some(v) => v,
            None => return Err(PropertyError::CorruptedProperty),
        };

        // The base class TODO: find if label or parent
        let base_class = CStr::from_bytes_until_nul(&base_chunk.data)
            .map_err(|_| PropertyError::CorruptedProperty)?
            .to_str()
            .map_err(|_| PropertyError::CorruptedProperty)?
            .to_string();
        // The name of the odf
        let type_class = CStr::from_bytes_until_nul(&type_chunk.data)
            .map_err(|_| PropertyError::CorruptedProperty)?
            .to_str()
            .map_err(|_| PropertyError::CorruptedProperty)?
            .to_string();

        let mut prop_index = 2;
        let mut prop_subchunk = subchunks
            .get(prop_index)
            .ok_or(PropertyError::CorruptedProperty)?;
        let mut properties: HashMap<String, String> = HashMap::new();
        while prop_subchunk.header.name == "PROP" {
            let (hash_index, value) = (
                prop_subchunk
                    .data
                    .get(0..4)
                    .ok_or(PropertyError::CorruptedProperty)?
                    .try_into()
                    .map_err(|_| PropertyError::CorruptedProperty)?,
                String::from_utf8(
                    prop_subchunk
                        .data
                        .get(4..)
                        .ok_or(PropertyError::CorruptedProperty)?
                        .try_into()
                        .map_err(|_| PropertyError::CorruptedProperty)?,
                )
                .map_err(|_| PropertyError::CorruptedProperty)?
                .replace("\0", ""),
            );
            properties.insert(
                match HASHVALUES.get(hash_index) {
                    Some(v) => v.to_string(),
                    None => return Err(PropertyError::CorruptedProperty),
                },
                value, //.as_str()
            );
            prop_index += 1;

            prop_subchunk = match subchunks.get(prop_index) {
                Some(v) => v,
                None => break,
            }
        }

        let lab = if CLASSLABELS.contains(&base_class.as_str()) {
            Some(base_class.clone())
        } else {
            None
        };

        let parent = match lab {
            Some(_) => None,
            None => Some(base_class),
        };

        Ok(PropertyContainer {
            r#type: r#type,
            properties: properties,
            name: type_class,
            class_label: lab,
            class_parent: parent,
        })
    }

    /// Get the ODF text representation of this object
    pub fn get_odf(&self) -> String {
        let mut result: String = format!(
            "[{:?}]\n\n{}\n",
            self.r#type,
            match self.class_label.clone() {
                Some(v) => format!("ClassLabel = {}", v),
                // TODO: better error handling
                None => format!("ClassParent = {}", self.class_parent.clone().unwrap()),
            }
        );

        self.properties.get("GeometryName").map(|v| {
            result = format!("{}\nGeometryName = {}\n", result, v);
        });

        result = format!("{}\n[Properties]\n\n", result);

        for (k, v) in self.properties.clone() {
            match k.as_str() {
                "GeometryName" => continue,
                _ => {
                    result = format!(
                        "{}\n{} = {}\n",
                        result,
                        k,
                        if v.parse::<u64>().is_ok() {
                            v
                        } else {
                            format!("\"{}\"", v)
                        }
                    );
                }
            }
        }

        result
    }
}
