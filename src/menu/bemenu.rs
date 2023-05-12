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
  items: Vec<*mut bm_item>,
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

  fn show(&mut self) -> Result<Option<String>, Error> {
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
        let text = std::ffi::CStr::from_ptr(bm_item_get_text(selected)).to_str().unwrap();

        return Ok(Some(text.to_string()));
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
    for item in (self.clipboard.read().unwrap().hist).iter().rev() {
      match &item.data {
        clipboard::ItemData::Text(data) => unsafe {
          let item = self.create_text_item(data);
          bm_menu_add_item(menu, item);
          self.items.push(item);
        },
        clipboard::ItemData::Image(_) => {}
      }
    }
  }

  fn create_text_item(&self, item: &clipboard::TextItem) -> *mut bm_item {
    let item = CString::new(item.text.clone()).unwrap();
    unsafe { bm_item_new(std::ffi::CStr::as_ptr(&item)) }
  }
}

impl Drop for BeMenu {
  fn drop(&mut self) {
    unsafe {
      // for item in self.items.iter() {
      //   bm_item_free(*item);
      // }
      bm_menu_free(self.menu);
    }
  }
}
