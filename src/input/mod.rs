use evdev::{
  uinput::{VirtualDevice, VirtualDeviceBuilder},
  AttributeSet, EventType, InputEvent, Key,
};

pub struct UDevice {
  device: VirtualDevice,
}

impl UDevice {
  pub fn new() -> Self {
    let mut keys = AttributeSet::<Key>::new();
    keys.insert(Key::KEY_PASTE);

    let device = VirtualDeviceBuilder::new()
      .unwrap()
      .name("wayclip")
      .with_keys(&keys)
      .unwrap()
      .build()
      .unwrap();

    Self { device }
  }

  pub fn paste(&mut self) {
    let type_ = EventType::KEY;
    let code = Key::KEY_PASTE.code();

    let down_event = InputEvent::new(type_, code, 1);
    self.device.emit(&[down_event]).unwrap();
    let up_event = InputEvent::new(type_, code, 0);
    self.device.emit(&[up_event]).unwrap();
  }

  pub fn copy(&self, text: String, mime: String) {
    let opts = wl_clipboard_rs::copy::Options::new();
    opts
      .copy(
        wl_clipboard_rs::copy::Source::Bytes(text.into_bytes().into()),
        wl_clipboard_rs::copy::MimeType::Specific(mime),
      )
      .unwrap();
  }
}
