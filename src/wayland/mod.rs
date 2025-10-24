use std::{
  io::Read,
  os::{fd::BorrowedFd, unix::io::AsRawFd},
};

use tracing::{debug, trace};
use wayland_client::{
  event_created_child,
  protocol::{
    wl_registry,
    wl_seat::{self, WlSeat},
  },
  Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};

use wayland_protocols::ext::data_control::v1::client::{
  ext_data_control_device_v1::{self, ExtDataControlDeviceV1},
  ext_data_control_manager_v1::ExtDataControlManagerV1,
  ext_data_control_offer_v1::ExtDataControlOfferV1,
};

use crate::{
  clipboard::{self, WrappedClipboard},
  communication, input,
};

#[derive(Clone, Debug)]
struct WaylandState {
  clipboard: WrappedClipboard,

  seat: Option<WlSeat>,
  manager: Option<ExtDataControlManagerV1>,
  device: Option<ExtDataControlDeviceV1>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
  fn event(
    state: &mut Self,
    registry: &wl_registry::WlRegistry,
    event: wl_registry::Event,
    _: &(),
    _: &Connection,
    qh: &QueueHandle<WaylandState>,
  ) {
    if let wl_registry::Event::Global {
      name,
      interface,
      version: _version,
    } = event
    {
      // println!("[{}] {} (v{})", name, interface, _version);
      match &interface[..] {
        "wl_seat" => {
          registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
        }
        "ext_data_control_manager_v1" => {
          state.manager = Some(registry.bind::<ExtDataControlManagerV1, _, _>(name, 1, qh, ()));
        }
        _ => {}
      }
    }
  }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
  fn event(state: &mut Self, seat: &wl_seat::WlSeat, _: wl_seat::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
    if state.seat.is_some() {
      return;
    };

    state.seat = Some(seat.clone());
  }
}

impl Dispatch<ExtDataControlManagerV1, ()> for WaylandState {
  fn event(
    _: &mut Self,
    _: &ExtDataControlManagerV1,
    _: <ExtDataControlManagerV1 as Proxy>::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
  }
}

// impl Dispatch<ExtDataControlSourceV1, ()> for WaylandState {
//   fn event(
//     _: &mut Self,
//     _: &ExtDataControlSourceV1,
//     _: <ExtDataControlSourceV1 as Proxy>::Event,
//     _: &(),
//     _: &Connection,
//     _: &QueueHandle<Self>,
//   ) {
//     panic!("source event");
//   }
// }

impl Dispatch<ExtDataControlDeviceV1, ()> for WaylandState {
  fn event(
    state: &mut Self,
    _: &ExtDataControlDeviceV1,
    event: <ExtDataControlDeviceV1 as Proxy>::Event,
    _: &(),
    conn: &Connection,
    _: &QueueHandle<Self>,
  ) {
    match event {
      ext_data_control_device_v1::Event::DataOffer { id } => {
        let id = id.id();
        trace!("data offer id: {:?}", id);
        state.clipboard.write().unwrap().new_offer(id);
      }

      ext_data_control_device_v1::Event::Selection { id } if id.is_some() => {
        let id = id.unwrap().id();
        trace!("selection id: {:?}", id);
        let item = get_item(conn, state);
        if let Some(item) = item {
          state.clipboard.write().unwrap().commit(item);
        }
      }
      _ => {}
    }
  }

  event_created_child!(
    WaylandState,
    ExtDataControlDeviceV1,
    [ext_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ExtDataControlOfferV1, ())]
  );
}

fn get_item(conn: &Connection, state: &WaylandState) -> Option<clipboard::Item> {
  let borrow = state.clipboard.read().unwrap();
  let live = if let Some(live) = &borrow.live {
    live
  } else {
    return None;
  };

  let offer = live.offer.as_ref().unwrap();

  if let Some(mime_type) = live.mime_types.iter().find(|mime_type| mime_type.starts_with("image/")) {
    debug!("image mime type found: {:?}", mime_type);

    if !state.clipboard.read().unwrap().get_config().general.allow_images {
      return None;
    }

    let (mut read, write) = os_pipe::pipe().expect("fuck shit");
    offer.receive((*mime_type).clone(), unsafe {
      BorrowedFd::borrow_raw(write.as_raw_fd())
    });
    drop(write);

    conn.roundtrip().unwrap();

    let mut buffer = vec![];
    read.read_to_end(&mut buffer).unwrap();

    debug!("image buffer size: {:?} bytes", buffer.len());

    trace!("wayland data transferred in: {:?}", live.instant.elapsed());

    if let Some(file_type) = infer::get(&buffer) {
      debug!("file type: {:?} confirmed", file_type);
      let item = clipboard::Item {
        id: live.id.to_string(),
        data: clipboard::ItemData::Image(clipboard::ImageItem {
          image: buffer,
          mime: mime_type.to_string(),
        }),
      };

      return Some(item);
    }
  }

  let preferred_text_mime = state.clipboard.read().unwrap().preferred_text_mime();
  let (mut read, write) = os_pipe::pipe().expect("fuck shit");

  offer.receive(preferred_text_mime.clone(), unsafe {
    BorrowedFd::borrow_raw(write.as_raw_fd())
  });
  drop(write);

  conn.roundtrip().unwrap();

  let mut text = String::new();
  read.read_to_string(&mut text).unwrap();

  debug!("text buffer size: {:?} bytes", text.clone().len());

  trace!("wayland data transferred in: {:?}", live.instant.elapsed());

  if text.trim().is_empty() {
    return None;
  }

  let item = clipboard::Item {
    id: live.id.to_string(),
    data: clipboard::ItemData::Text(clipboard::TextItem {
      text,
      mime: preferred_text_mime,
    }),
  };

  Some(item)
}

impl Dispatch<ExtDataControlOfferV1, ()> for WaylandState {
  fn event(
    state: &mut Self,
    offer: &ExtDataControlOfferV1,
    event: <ExtDataControlOfferV1 as Proxy>::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    // println!("offer: {:?}", offer);
    let mut borrow = state.clipboard.write().unwrap();
    let live = if let Some(live) = &mut borrow.live {
      live
    } else {
      debug!("no in progress");
      return;
    };

    let mime_type = match event {
      wayland_protocols::ext::data_control::v1::client::ext_data_control_offer_v1::Event::Offer { mime_type } => {
        mime_type
      }
      _ => return,
    };

    live.handle_offer(offer, mime_type);
  }
}

impl WaylandState {
  fn new(clipboard: WrappedClipboard) -> (Self, EventQueue<Self>) {
    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();
    let mut queue = conn.new_event_queue();
    let qh: QueueHandle<WaylandState> = queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut state = WaylandState {
      clipboard,

      seat: None,
      manager: None,
      device: None,
    };

    // double roundtrip needed for seat to be set
    queue.roundtrip(&mut state).unwrap();
    queue.roundtrip(&mut state).unwrap();

    trace!("wayland seat: {:?}", state.seat);
    trace!("wayland manager: {:?}", state.manager);

    let seat = state.seat.clone().unwrap();
    state.device = Some(state.manager.as_ref().unwrap().get_data_device(&seat, &qh, ()));

    (state, queue)
  }
}

pub fn watch_clipboard(
  clipboard: WrappedClipboard,
  menu_message_receiver: std::sync::mpsc::Receiver<communication::MPSCMessage>,
) {
  trace!("Initializing Wayland state and queue");
  let (mut state, mut queue) = WaylandState::new(clipboard.clone());
  let mut dev = input::UDevice::new();

  trace!("Spawning menu message handling thread");
  std::thread::spawn(move || loop {
    trace!("Waiting for menu message");
    let (message, index) = menu_message_receiver.recv().unwrap();
    trace!("Received menu message: message={}, index={}", message, index);
    dev.copy(message, clipboard.read().unwrap().get_config().data.mime.clone());
    trace!("Performed copy operation");
    dev.paste();
    trace!("Performed paste operation");

    clipboard.write().unwrap().pasted_idx(index);
    trace!("Updated clipboard pasted index");
  });

  trace!("Starting main event loop");
  loop {
    trace!("Blocking dispatch start");
    queue.blocking_dispatch(&mut state).unwrap();
    trace!("Blocking dispatch completed");
  }
}
