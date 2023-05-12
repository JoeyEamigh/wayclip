mod bemenu;

use crate::{clipboard, config::error::Error};

pub trait Menu {
  fn new(clipboard: clipboard::WrappedClipboard) -> Result<Box<Self>, Error>
  where
    Self: Sized;
  fn show(&mut self) -> Result<Option<String>, Error>;
}

pub type WrappedMenu = Box<dyn Menu>;

// pub fn init(clipboard: clipboard::WrappedClipboard) -> Result<WrappedMenu, Error> {
//   #[cfg(feature = "bemenu")]
//   if has_bemenu() {
//     return Ok(bemenu::BeMenu::new(clipboard)?);
//   }

//   Err(Error::NoMenu)
// }

// pub fn has_bemenu() -> bool {
//   #[cfg(not(feature = "bemenu"))]
//   return false;

//   pkg_config::Config::new()
//     .atleast_version("0.6.0")
//     .probe("bemenu")
//     .is_ok()
// }

pub fn init(clipboard: clipboard::WrappedClipboard) -> Result<WrappedMenu, Error> {
  if let Ok(menu) = bemenu::BeMenu::new(clipboard) {
    Ok(menu)
  } else {
    Err(Error::NoMenu)
  }
}
