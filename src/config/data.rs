use std::path::PathBuf;

use super::{consts::*, file::FileHelper};
use figment::{
  providers::{Format, Toml},
  Figment,
};
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug, Default)]
pub struct Config {
  #[serde(default)]
  pub general: General,
  #[serde(default)]
  pub data: Data,
  #[serde(default)]
  pub encryption: Encryption,
  #[serde(default)]
  pub bemenu: BeMenuConfig,

  // private
  #[serde(skip)]
  figment: Figment,
  #[serde(skip)]
  helper: FileHelper,
  #[serde(skip)]
  path: PathBuf,
}

impl Config {
  pub fn load(helper: FileHelper) -> Self {
    let path = helper.init_config();
    let figment = Figment::new().join(Toml::file(path.clone()));
    let config = figment.extract::<Config>();

    let mut config = match config {
      Ok(config) => config,
      Err(_) => Config::default(),
    };
    config.figment = figment;
    config.helper = helper;
    config.path = path;

    config
  }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct General {
  #[serde(default)]
  pub max_history: usize,
  #[serde(default)]
  pub menu: String,
}

impl Default for General {
  fn default() -> Self {
    General {
      max_history: MAX_HISTORY,
      menu: MENU.to_string(),
    }
  }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Data {
  #[serde(default)]
  pub mime: String,
  #[serde(default)]
  pub dedupe: bool,
}

impl Default for Data {
  fn default() -> Self {
    Data {
      mime: MIME.to_string(),
      dedupe: DEDUPE,
    }
  }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Encryption {
  #[serde(default)]
  pub encrypt: bool,
  #[serde(default)]
  pub key: Option<String>,
}

impl Default for Encryption {
  fn default() -> Self {
    Encryption {
      encrypt: ENCRYPT,
      key: None,
    }
  }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BeMenuConfig {
  #[serde(default)]
  pub font: String,
  #[serde(default)]
  pub title: String,
  #[serde(default)]
  pub lines: u32,
  #[serde(default)]
  pub grab_focus: bool,
  #[serde(default)]
  pub monitor: i32,
}

impl Default for BeMenuConfig {
  fn default() -> Self {
    BeMenuConfig {
      font: FONT.to_string(),
      title: TITLE.to_string(),
      lines: LINES,
      grab_focus: GRAB_FOCUS,
      monitor: MONITOR,
    }
  }
}
