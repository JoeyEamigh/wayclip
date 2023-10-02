#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::{clipboard, config::error::Error};
use std::ffi::CString;

use super::Menu;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct BeMenu {
  clipboard: clipboard::WrappedClipboard,

  menu: *mut bm_menu,
  items: Vec<(*mut bm_item, *mut usize)>,
}

impl Menu for BeMenu {
  fn new(clipboard: clipboard::WrappedClipboard) -> Result<Box<Self>, Error> {
    if !unsafe { bm_init() } {
      return Err(Error::BMenu("Failed to initialize bmenu".into()));
    };

    Ok(Box::new(Self {
      clipboard,

      menu: unsafe { bm_menu_new(std::ptr::null()) },
      items: Vec::new(),
    }))
  }

  fn show(&mut self) -> Result<Option<(String, usize)>, Error> {
    let menu = self.menu;
    self.handle_config(menu);
    self.add_items(menu);

    unsafe {
      let unicode: *mut u32 = &mut 0;
      let mut status: bm_run_result;

      loop {
        bm_menu_render(menu);
        let key = bm_menu_poll_key(menu, unicode);
        let pointer = bm_menu_poll_pointer(menu);
        let touch = bm_menu_poll_touch(menu);

        status = bm_menu_run_with_events(menu, key, pointer, touch, *unicode);
        if status != bm_run_result_BM_RUN_RESULT_RUNNING {
          break;
        }
      }

      if status == bm_run_result_BM_RUN_RESULT_SELECTED {
        let selected = *bm_menu_get_selected_items(menu, std::ptr::null_mut());
        let selected_idx = bm_item_get_userdata(selected) as *mut i32;
        let text = std::ffi::CStr::from_ptr(bm_item_get_text(selected)).to_str().unwrap();

        return Ok(Some((text.to_string(), *Box::from_raw(selected_idx) as usize)));
      } else if status == bm_run_result_BM_RUN_RESULT_CANCEL {
        return Ok(None);
      }
    };

    Ok(None)
  }
}

impl BeMenu {
  fn handle_config(&self, menu: *mut bm_menu) {
    let cb = self.clipboard.clone();
    let config = cb.read().unwrap().get_config().bemenu;

    unsafe {
      bm_menu_set_filter_mode(menu, bm_filter_mode_BM_FILTER_MODE_DMENU_CASE_INSENSITIVE);
      bm_menu_set_lines(menu, config.lines);
      bm_menu_set_title(menu, std::ffi::CStr::as_ptr(&CString::new(config.title).unwrap()));
      bm_menu_set_font(menu, std::ffi::CStr::as_ptr(&CString::new(config.font).unwrap()));
      bm_menu_grab_keyboard(menu, config.grab_focus);
      bm_menu_set_monitor(menu, config.monitor);
      bm_menu_set_spacing(menu, true);
    }
  }

  fn add_items(&mut self, menu: *mut bm_menu) {
    for (idx, item) in (self.clipboard.read().unwrap().hist).iter().rev().enumerate() {
      match &item.data {
        clipboard::ItemData::Text(data) => unsafe {
          let (item, idx) = self.create_text_item(data, idx);
          bm_menu_add_item(menu, item);
          self.items.push((item, idx));
        },
        clipboard::ItemData::Image(_) => {}
      }
    }
  }

  fn create_text_item(&self, item: &clipboard::TextItem, idx: usize) -> (*mut bm_item, *mut usize) {
    let item = CString::new(item.text.clone()).unwrap();
    unsafe {
      let item = bm_item_new(std::ffi::CStr::as_ptr(&item));
      let idx = Box::into_raw(Box::new(idx));

      bm_item_set_userdata(item, idx as *mut _);

      (item, idx)
    }
  }
}

impl Drop for BeMenu {
  fn drop(&mut self) {
    unsafe {
      bm_menu_free(self.menu);

      for (_, idx) in self.items.iter() {
        drop(Box::from_raw(*idx));
      }
    }
  }
}
