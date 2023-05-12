use self::{data::Config, file::FileHelper};

pub mod cli;
mod consts;
pub mod data;
pub mod error;
pub mod file;
pub mod install;
mod resources;

pub fn init(mut helper: FileHelper) -> (Config, FileHelper) {
  let config = Config::load(helper.clone());
  helper.init_cocoon(&config);

  (config, helper)
}

pub fn init_helper() -> FileHelper {
  FileHelper::new()
}
