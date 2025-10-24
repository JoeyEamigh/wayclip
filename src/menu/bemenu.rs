#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::{clipboard, config::error::Error};
use std::ffi::CString;
use std::mem;
use tracing::trace;

use super::Menu;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct BeMenu {
  clipboard: clipboard::WrappedClipboard,
}

impl Menu for BeMenu {
  fn new(clipboard: clipboard::WrappedClipboard) -> Result<Box<Self>, Error> {
    trace!("Initializing BeMenu");
    if !unsafe { bm_init() } {
      return Err(Error::BMenu("Failed to initialize bmenu".into()));
    };

    Ok(Box::new(Self { clipboard }))
  }

  fn show(&self) -> Result<Option<(String, usize)>, Error> {
    trace!("Starting BeMenu show");
    let menu = unsafe { bm_menu_new(std::ptr::null()) };
    self.handle_config(menu);
    self.add_items(menu);

    unsafe {
      let mut unicode = 0u32;
      let unicode_ptr: *mut u32 = &mut unicode;
      let mut status: bm_run_result;

      trace!("Entering BeMenu event loop");
      loop {
        trace!("Rendering menu");
        bm_menu_render(menu);
        let key = bm_menu_poll_key(menu, unicode_ptr);
        trace!("Polled key: {:?}", key);
        let pointer = bm_menu_poll_pointer(menu);
        trace!("Polled pointer: {:?}", pointer);
        let touch = bm_menu_poll_touch(menu);
        trace!("Polled touch: {:?}", touch);

        status = bm_menu_run_with_events(menu, key, pointer, touch, unicode);
        trace!("Run status: {:?}", status);
        if status != bm_run_result_BM_RUN_RESULT_RUNNING {
          break;
        }
      }
      trace!("Exited BeMenu event loop");

      if status == bm_run_result_BM_RUN_RESULT_SELECTED {
        let selected = *bm_menu_get_selected_items(menu, std::ptr::null_mut());
        let selected_idx = bm_item_get_userdata(selected) as *mut usize;
        let text = std::ffi::CStr::from_ptr(bm_item_get_text(selected)).to_str().unwrap();
        let index = if selected_idx.is_null() {
          0usize
        } else {
          let value = *selected_idx;
          bm_item_set_userdata(selected, std::ptr::null_mut());
          libc::free(selected_idx.cast());
          value
        };
        let owned = (text.to_string(), index);

        bm_menu_free(menu);
        return Ok(Some(owned));
      } else if status == bm_run_result_BM_RUN_RESULT_CANCEL {
        bm_menu_free(menu);
        return Ok(None);
      }

      bm_menu_free(menu);
    };

    Ok(None)
  }
}

impl BeMenu {
  fn handle_config(&self, menu: *mut bm_menu) {
    trace!("Handling BeMenu config");
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

  fn add_items(&self, menu: *mut bm_menu) {
    self.remove_items(menu);

    trace!("adding items to menu");
    for (idx, item) in (self.clipboard.read().unwrap().hist).iter().rev().enumerate() {
      // trace!("Processing item at index {}", idx);
      match &item.data {
        clipboard::ItemData::Text(data) => unsafe {
          self.add_text_item(menu, data, idx);
        },
        clipboard::ItemData::Image(_) => {}
      }
    }
  }

  fn remove_items(&self, menu: *mut bm_menu) {
    trace!("removing items from menu");

    unsafe { bm_menu_free_items(menu) }
  }

  unsafe fn add_text_item(&self, menu: *mut bm_menu, text_item: &clipboard::TextItem, idx: usize) {
    let c_string = match CString::new(text_item.text.clone()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let text_ptr = c_string.into_raw();
    let item_ptr = bm_item_new(text_ptr);
    if item_ptr.is_null() {
      let _ = CString::from_raw(text_ptr);
      return;
    }

    let idx_ptr = libc::malloc(mem::size_of::<usize>()) as *mut usize;
    if idx_ptr.is_null() {
      bm_item_free(item_ptr);
      let _ = CString::from_raw(text_ptr);
      return;
    }

    idx_ptr.write(idx);
    bm_item_set_userdata(item_ptr, idx_ptr.cast());

    if !bm_menu_add_item(menu, item_ptr) {
      bm_item_set_userdata(item_ptr, std::ptr::null_mut());
      libc::free(idx_ptr.cast());
      bm_item_free(item_ptr);
      let _ = CString::from_raw(text_ptr);
    }
  }
}
