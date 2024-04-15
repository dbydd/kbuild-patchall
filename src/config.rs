use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};

/// This is a struct will be deserialized from the given filename.
///
/// version indicates the version of the byteos.
/// config indicates the configuration of the byteos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ByteOSConfig {
    pub version: Option<String>,
    // crates: Option<String>,
    // mocules: Option<String>,
    /// Config list for byteos. This field will be converted to rust cfg.
    // configs: Option<HashMap<String, String>>
    pub bin: Option<HashMap<String, BinaryConfig>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryConfig {
    pub target: String,
    pub run: Option<String>,
    pub configs: HashMap<String, String>,
}

impl ByteOSConfig {
    pub fn get_bin_config(&self, bin: &str) -> Option<BinaryConfig> {
        match self.bin {
            Some(ref configs) => configs.get(bin).cloned(),
            None => None,
        }
    }
}

pub fn read_toml(path: &str) -> Result<ByteOSConfig, String> {
    let fcontent = fs::read_to_string(path).map_err(|x| x.to_string())?;
    let byteos_config: ByteOSConfig = toml::from_str(&fcontent).map_err(|x| x.to_string())?;
    Ok(byteos_config)
}
