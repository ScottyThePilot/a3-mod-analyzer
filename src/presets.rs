use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{Error, Contextualize};

use std::io::{BufRead, BufReader};
use std::fs::{self, File};
use std::path::PathBuf;
use std::collections::HashMap;



pub fn get_presets() -> Result<HashMap<String, Preset>, Error> {
  let path = crate::launcher_dir()?.join("Presets");
  let mut presets = HashMap::new();
  for entry in fs::read_dir(&path).context(&path)? {
    let entry = entry.context(&path)?;
    let entry_path = entry.path();
    if entry.file_type().context(&entry_path)?.is_file() {
      let file = File::open(&entry_path).context(&entry_path)?;
      let name = entry_path.file_stem().and_then(|s| s.to_str())
        .ok_or_else(|| Error::InvalidPresetPath(entry_path.clone()))?;
      let preset = Preset::from_reader(BufReader::new(file), name).context(name)?;
      presets.insert(name.to_owned(), preset);
    };
  };

  Ok(presets)
}

#[derive(Debug, Serialize)]
pub struct Preset {
  pub name: String,
  pub last_update: DateTime<FixedOffset>,
  pub mods: Vec<u64>
}

impl Preset {
  fn from_preset_repr(preset_repr: PresetRepr, name: String) -> Self {
    let PresetRepr {
      last_update: LastUpdateRepr { value: last_update },
      published_ids: PublishedIdsRepr { ids: mods }
    } = preset_repr;
    let mods = mods.into_iter()
      .filter_map(|id| id.value.into_steam())
      .collect();
    Preset { name, last_update, mods }
  }

  fn from_reader<R: BufRead>(reader: R, name: impl Into<String>) -> Result<Self, quick_xml::de::DeError> {
    let preset_repr: PresetRepr = quick_xml::de::from_reader(reader)?;
    Ok(Preset::from_preset_repr(preset_repr, name.into()))
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "addons-presets")]
struct PresetRepr {
  #[serde(rename = "last-update")]
  last_update: LastUpdateRepr,
  #[serde(rename = "published-ids")]
  published_ids: PublishedIdsRepr
}

#[derive(Debug, Deserialize)]
#[serde(rename = "last-update")]
struct LastUpdateRepr {
  #[serde(rename = "$value")]
  #[serde(deserialize_with = "deserialize_date_time_str")]
  value: DateTime<FixedOffset>
}

#[derive(Debug, Deserialize)]
#[serde(rename = "published-ids")]
struct PublishedIdsRepr {
  #[serde(rename = "id")]
  ids: Vec<PublishedIdRepr>
}

#[derive(Debug, Deserialize)]
#[serde(rename = "id")]
struct PublishedIdRepr {
  #[serde(rename = "$value")]
  #[serde(deserialize_with = "deserialize_steam_id_str")]
  value: PresetAddon
}

fn deserialize_steam_id_str<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PresetAddon, D::Error> {
  fn error<E: serde::de::Error>(value: &str) -> E {
    E::invalid_value(
      serde::de::Unexpected::Str(value),
      &"a string, prefixed with \"steam:\" followed by an integer, or a string, prefixed with \"local:\" followed by a path"
    )
  }

  let string = String::deserialize(deserializer)?;
  if let Some(string) = string.strip_prefix("steam:") {
    let id = string.parse::<u64>().map_err(|_| error(string))?;
    Ok(PresetAddon::Steam(id))
  } else if let Some(string) = string.strip_prefix("local:") {
    Ok(PresetAddon::Local(PathBuf::from(string)))
  } else {
    Err(error(&string))
  }
}

fn deserialize_date_time_str<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error> {
  let string = String::deserialize(deserializer)?;
  DateTime::parse_from_rfc3339(&string).map_err(<D::Error as serde::de::Error>::custom)
}

#[derive(Debug)]
enum PresetAddon {
  Steam(u64),
  Local(PathBuf)
}

impl PresetAddon {
  fn into_steam(self) -> Option<u64> {
    match self {
      PresetAddon::Steam(id) => Some(id),
      PresetAddon::Local(_) => None
    }
  }
}
