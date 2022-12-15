use chrono::{DateTime, FixedOffset, Utc};
use serde::Serialize;

use crate::addons::Addon;
use crate::presets::Preset;

use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct AddonAnalysis {
  pub id: u64,
  pub name: String,
  pub link: String,
  pub last_update: DateTime<Utc>,
  pub last_usage: Option<DateTime<FixedOffset>>,
  pub file_size: u64,
  pub preset_count: usize,
  pub dependency_count: usize,
  pub dependents_count: usize
}

pub fn perform_analysis(addons: &HashMap<u64, Addon>, presets: &HashMap<String, Preset>) -> Vec<AddonAnalysis> {
  let mut analysis = HashMap::new();

  for addon in addons.values() {
    analysis.insert(addon.id, AddonAnalysis {
      id: addon.id,
      name: addon.display_name.clone(),
      link: addon.url.clone(),
      last_update: addon.last_update,
      last_usage: None,
      file_size: addon.file_system_size,
      preset_count: 0,
      dependency_count: addon.dependencies.len(),
      dependents_count: 0
    });
  };

  for addon in addons.values() {
    for &dependency in addon.dependencies.iter() {
      if let Some(addon_analysis) = analysis.get_mut(&dependency) {
        addon_analysis.dependents_count += 1;
      };
    };
  };

  for preset in presets.values() {
    for &addon in preset.mods.iter() {
      if let Some(addon_analysis) = analysis.get_mut(&addon) {
        addon_analysis.preset_count += 1;
        addon_analysis.last_usage = Some(match addon_analysis.last_usage {
          Some(last_usage) => std::cmp::max(last_usage, preset.last_update),
          None => preset.last_update
        });
      };
    };
  };

  let mut analysis = analysis.into_values()
    .collect::<Vec<AddonAnalysis>>();
  analysis.sort_unstable_by_key(|addon_analysis| addon_analysis.name.to_ascii_lowercase());
  analysis
}
