mod bemenu;

use crate::{clipboard, config::error::Error};

pub trait Menu {
  fn new(clipboard: clipboard::WrappedClipboard) -> Result<Box<Self>, Error>
  where
    Self: Sized;
  fn show(&mut self) -> Result<Option<(String, usize)>, Error>;
}

pub type WrappedMenu = Box<dyn Menu>;

pub fn init(clipboard: clipboard::WrappedClipboard) -> Result<WrappedMenu, Error> {
  if let Ok(menu) = bemenu::BeMenu::new(clipboard) {
    Ok(menu)
  } else {
    Err(Error::NoMenu)
  }
}
