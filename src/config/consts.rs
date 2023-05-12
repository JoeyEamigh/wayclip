pub const APP_NAME: &str = "wayclip";
pub const CONFIG_FILE: &str = "config.toml";
pub const SYSTEMD_FILE: &str = "wayclip.service";

// config constants
// [general]
pub const MAX_HISTORY: usize = 0;
pub const MENU: &str = "bemenu";

// [data]
pub const MIME: &str = "text/plain";
pub const DEDUPE: bool = true;

// [encryption]
pub const ENCRYPT: bool = true;

// [bemenu]
pub const FONT: &str = "monospace 12";
pub const TITLE: &str = "search >";
pub const LINES: u32 = 15;
pub const GRAB_FOCUS: bool = true;
pub const MONITOR: i32 = -1;
