use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Contextualize};

use std::io::{BufReader, Read};
use std::fs::File;
use std::collections::HashMap;
use std::path::PathBuf;



pub fn get_addons() -> Result<HashMap<u64, Addon>, Error> {
  let path = crate::launcher_dir()?.join("Steam.json");
  let file = File::open(&path).context(&path)?;
  Addon::from_reader(BufReader::new(file)).map_err(Error::AddonsParsingFailed)
}

#[derive(Debug, Serialize)]
pub struct Addon {
  pub id: u64,
  pub display_name: String,
  pub path: PathBuf,
  pub url: String,
  pub file_system_size: u64,
  pub file_system_space_required: u64,
  pub last_update: DateTime<Utc>,
  pub dependencies: Vec<u64>
}

impl Addon {
  fn from_extension_repr(extension: ExtensionRepr) -> Self {
    let ExtensionRepr {
      display_name,
      extension_path: path,
      file_system_space_required,
      url,
      storage_info: StorageInfoRepr {
        published_id: id,
        last_update,
        file_system_size
      },
      steam_dependencies: dependencies
    } = extension;

    Addon {
      id,
      display_name,
      path,
      url,
      file_system_size,
      file_system_space_required,
      last_update,
      dependencies
    }
  }

  fn from_reader<R: Read>(reader: R) -> serde_json::Result<HashMap<u64, Self>> {
    let steam: SteamExtensionsRepr = serde_json::from_reader(reader)?;
    let addons = steam.extensions.into_iter()
      .map(Self::from_extension_repr)
      .map(|addon| (addon.id, addon))
      .collect::<HashMap<u64, Self>>();
    Ok(addons)
  }
}



#[derive(Debug, Deserialize)]
struct SteamExtensionsRepr {
  #[serde(rename = "Extensions")]
  extensions: Vec<ExtensionRepr>
}

#[derive(Debug, Deserialize)]
struct ExtensionRepr {
  #[serde(rename = "DisplayName")]
  display_name: String,
  #[serde(rename = "ExtensionPath")]
  extension_path: PathBuf,
  #[serde(rename = "FileSystemSpaceRequired")]
  file_system_space_required: u64,
  #[serde(rename = "Url")]
  url: String,
  #[serde(rename = "StorageInfo")]
  storage_info: StorageInfoRepr,
  #[serde(rename = "SteamDependencies")]
  steam_dependencies: Vec<u64>
}

#[derive(Debug, Deserialize)]
struct StorageInfoRepr {
  #[serde(rename = "PublishedId")]
  published_id: u64,
  #[serde(rename = "LastUpdate")]
  #[serde(with = "chrono::serde::ts_seconds")]
  last_update: DateTime<Utc>,
  #[serde(rename = "FileSystemSize")]
  file_system_size: u64
}
