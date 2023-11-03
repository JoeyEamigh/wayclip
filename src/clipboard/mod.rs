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

    if self.config.data.dedupe {
      #[cfg(debug_assertions)]
      let timer = std::time::Instant::now();

      let idx = self
        .hist
        .iter()
        .position(|item| match (item.clone().data, data.clone().data) {
          (ItemData::Text(text), ItemData::Text(new_text)) if text.text == new_text.text => true,
          (ItemData::Image(image), ItemData::Image(new_image)) if image.image == new_image.image => true,
          _ => false,
        });

      if let Some(idx) = idx {
        debug!("found duplicate clipboard item - removing");
        self.hist.remove(idx);
      }

      #[cfg(debug_assertions)]
      trace!("(copy function) deduped clipboard in {:?}", timer.elapsed());
    }

    self.hist.push(data);
    if self.hist.len() > self.config.general.max_history && self.config.general.max_history > 0 {
      self.hist.remove(0);
    }

    self.live = None;
    self.save();
  }

  pub fn preferred_text_mime(&self) -> String {
    self.config.data.mime.to_string()
  }

  pub fn get_config(&self) -> Config {
    self.config.clone()
  }

  pub fn dump(&self) {
    println!("{:#?}", self.hist);
  }

  pub fn clear(&mut self) {
    self.hist.clear();
    self.save()
  }

  /// handle a clipboard paste event by moving the selected index to the end
  pub fn pasted_idx(&mut self, idx: usize) {
    if idx >= self.hist.len() {
      return;
    }

    // remove the item from history reversed
    let item = self.hist.remove(self.hist.len() - idx - 1);
    self.hist.push(item);
    self.save();
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

    if let Some(mut existing) = existing {
      if !self.config.general.allow_images {
        #[cfg(debug_assertions)]
        let timer = std::time::Instant::now();

        existing.retain(|item| match item.data.clone() {
          ItemData::Text(_) => true,
          ItemData::Image(_) => false,
        });

        #[cfg(debug_assertions)]
        trace!("(restore function) image removal took {:?}", timer.elapsed());
      }

      if !self.config.data.dedupe {
        self.hist = existing;
      } else {
        use itertools::Itertools;

        #[cfg(debug_assertions)]
        let timer = std::time::Instant::now();

        self.hist = existing
          .into_iter()
          .unique_by(|item| match item.data.clone() {
            ItemData::Text(text) => text.text,
            // no good way to dedupe images with unique_by bc of clones - hashing maybe but that's a lot of work
            ItemData::Image(_) => item.id.clone(),
          })
          .collect();

        #[cfg(debug_assertions)]
        trace!("(restore function) dedupe took {:?}", timer.elapsed());
      }
    }

    debug!("restored {:?} clipboard items", self.hist.len());

    #[cfg(debug_assertions)]
    trace!("restored clipboard in {:?}", timer.elapsed());
  }
}
