use std::sync::{Arc, RwLock};

use crate::config::{data::Config, file::FileHelper};
use serde::{Deserialize, Serialize};
use tracing::debug;
use wayland_client::backend::ObjectId;

#[cfg(debug_assertions)]
use tracing::trace;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextItem {
  pub text: String,
  pub mime: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ImageItem {
  pub image: Vec<u8>,
  pub mime: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ItemData {
  Text(TextItem),
  Image(ImageItem),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Item {
  pub id: String,
  pub data: ItemData,
}

#[derive(Clone, Debug)]
pub struct LiveClipboard {
  pub id: ObjectId,
  pub offer:
    Option<wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1>,
  pub mime_types: Vec<String>,
  pub data: Vec<u8>,
  pub instant: std::time::Instant,
}

impl LiveClipboard {
  pub fn new(id: ObjectId) -> Self {
    LiveClipboard {
      id,
      offer: None,
      mime_types: vec![],
      data: vec![],
      instant: std::time::Instant::now(),
    }
  }

  pub fn handle_offer(
    &mut self,
    offer: &wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
    mime_type: String,
  ) {
    self.mime_types.push(mime_type);

    if self.offer.is_none() {
      self.offer = Some(offer.clone());
    }
  }
}

#[derive(Clone, Debug)]
pub struct Clipboard {
  pub live: Option<LiveClipboard>,
  pub hist: Vec<Item>,

  // private
  config: Config,
  helper: FileHelper,
}

pub type WrappedClipboard = Arc<RwLock<Clipboard>>;

impl Clipboard {
  pub fn init(config: Config, helper: FileHelper) -> WrappedClipboard {
    let mut cb = Clipboard {
      live: None,
      hist: vec![],

      // private
      config,
      helper,
    };

    cb.restore();

    Arc::new(RwLock::new(cb))
  }

  pub fn new_offer(&mut self, id: ObjectId) {
    self.live = Some(LiveClipboard::new(id));
  }

  pub fn commit(&mut self, data: Item) {
    if let Some(last) = self.hist.last() {
      match (last.clone().data, data.clone().data) {
        (ItemData::Text(text), ItemData::Text(new_text)) if text.text == new_text.text => return,
        (ItemData::Image(image), ItemData::Image(new_image)) if image.image == new_image.image => return,
        _ => {}
      }
    }

    self.hist.push(data);
    if self.hist.len() > self.config.general.max_history && self.config.general.max_history > 0 {
      self.hist.remove(0);
    }

    #[cfg(debug_assertions)]
    if let Some(live) = &mut self.live {
      trace!("wayland data in: {:?}", live.instant.elapsed())
    };

    self.live = None;
    self.save();
  }

  pub fn preferred_text_mime(&self) -> String {
    self.config.data.mime.to_string()
  }

  pub fn get_config(&self) -> Config {
    self.config.clone()
  }

  fn save(&self) {
    #[cfg(debug_assertions)]
    let timer = std::time::Instant::now();

    let savable = self.hist.clone();

    debug!("persisting {:?} clipboard items", savable.len());
    self.helper.persist_clipboard(savable);

    #[cfg(debug_assertions)]
    trace!("persisted clipboard in {:?}", timer.elapsed());
  }

  fn restore(&mut self) {
    #[cfg(debug_assertions)]
    let timer = std::time::Instant::now();

    let existing = self.helper.retrieve_clipboard();

    if let Some(existing) = existing {
      self.hist = existing;
    }

    debug!("restored {:?} clipboard items", self.hist.len());

    #[cfg(debug_assertions)]
    trace!("restored clipboard in {:?}", timer.elapsed());
  }
}
