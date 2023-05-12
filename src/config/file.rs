use cocoon::MiniCocoon;
use std::{fmt, fs, path::PathBuf};

use crate::clipboard::Item;

use super::{
  consts::{APP_NAME, CONFIG_FILE, SYSTEMD_FILE},
  data::Config,
  resources::Resource,
};

pub fn generate_cocoon(seed: [u8; 32], key: Option<String>) -> MiniCocoon {
  let key = match key {
    Some(key) => {
      if key.is_empty() {
        machine_uid::get().unwrap()
      } else {
        key
      }
    }
    None => machine_uid::get().unwrap(),
  };

  MiniCocoon::from_password(key.as_bytes(), &seed)
}

#[derive(Default)]
pub struct FileHelper {
  pub config_dir: PathBuf,
  pub cache_dir: PathBuf,
  pub log_dir: PathBuf,
  pub systemd_dir: PathBuf,

  // privates
  cocoon: Option<MiniCocoon>,
}

impl Clone for FileHelper {
  fn clone(&self) -> Self {
    FileHelper {
      config_dir: self.config_dir.clone(),
      cache_dir: self.cache_dir.clone(),
      log_dir: self.log_dir.clone(),
      systemd_dir: self.systemd_dir.clone(),

      // privates
      cocoon: None,
    }
  }
}

impl fmt::Debug for FileHelper {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("FileHelper")
      .field("config_dir", &self.config_dir)
      .field("cache_dir", &self.cache_dir)
      .field("log_dir", &self.log_dir)
      .finish()
  }
}

impl FileHelper {
  pub fn new() -> Self {
    let config_dir = init_dir(dirs::config_dir().unwrap().join(APP_NAME));
    let cache_dir = init_dir(dirs::cache_dir().unwrap().join(APP_NAME));
    let log_dir = init_dir(dirs::data_dir().unwrap().join(APP_NAME));
    let systemd_dir = init_dir(dirs::config_dir().unwrap().join("systemd/user"));

    FileHelper {
      config_dir,
      cache_dir,
      log_dir,
      systemd_dir,

      // privates
      cocoon: None,
    }
  }

  pub fn init_config(&self) -> PathBuf {
    if !self.config_dir.join(CONFIG_FILE).exists() {
      let default = Resource::get(CONFIG_FILE).unwrap();
      fs::write(self.config_dir.join(CONFIG_FILE), default.data).unwrap();
    }

    self.config_dir.join(CONFIG_FILE)
  }

  pub fn install_systemd_file(&self) -> PathBuf {
    let file = self.systemd_dir.join(SYSTEMD_FILE);

    if !file.exists() {
      let default = Resource::get(SYSTEMD_FILE).unwrap();
      fs::write(file.clone(), default.data).unwrap();
    }

    file
  }

  pub fn persist_clipboard(&self, clipboard: Vec<Item>) {
    let encoded = bincode::serialize(&clipboard).unwrap();
    let cocoon = self.cocoon.as_ref().unwrap();

    let mut writer = self.get_clipboard_file();
    cocoon.dump(encoded, &mut writer).unwrap();
  }

  pub fn retrieve_clipboard(&self) -> Option<Vec<Item>> {
    if !self.cache_dir.join("clipboard.bin").is_file() {
      return None;
    }

    let mut reader = self.get_clipboard_file();
    let cocoon = self.cocoon.as_ref().unwrap();

    let decrypted = cocoon.parse(&mut reader).unwrap();
    bincode::deserialize::<Vec<Item>>(&decrypted).ok()
  }

  pub fn init_cocoon(&mut self, config: &Config) {
    let seed = self.get_seed();
    let cocoon = generate_cocoon(seed, config.encryption.key.clone());

    self.cocoon = Some(cocoon);
  }

  fn get_clipboard_file(&self) -> fs::File {
    let path = self.cache_dir.join("clipboard.bin");

    if path.is_file() {
      return fs::OpenOptions::new().read(true).write(true).open(path).unwrap();
    }

    fs::File::create(path).unwrap()
  }

  fn get_seed_file(&self) -> fs::File {
    let path = self.config_dir.join("seed.bin");

    if path.is_file() {
      return fs::OpenOptions::new().read(true).write(true).open(path).unwrap();
    }

    fs::File::create(path).unwrap()
  }

  fn get_seed(&self) -> [u8; 32] {
    use std::io::{Read, Write};

    let mut handle = self.get_seed_file();
    let mut seed = [0u8; 32];

    let success = handle.read_exact(&mut seed);

    if success.is_err() {
      use rand::Rng;
      seed = rand::thread_rng().gen::<[u8; 32]>();

      handle.write_all(&seed).unwrap();
    }

    seed
  }
}

fn init_dir(dir: PathBuf) -> PathBuf {
  if dir.is_dir() {
    return dir;
  }

  fs::create_dir_all(dir.as_path()).unwrap();

  dir
}
