#![feature(if_let_guard)]

use clap::Parser;

mod clipboard;
mod communication;
mod config;
mod input;
mod menu;
mod wayland;

fn main() {
  let helper = config::init_helper();
  let _guard = init_logger(helper.log_dir.clone());

  let cli = config::cli::Cli::parse();

  match &cli.command {
    Some(config::cli::Commands::Toggle) => toggle(),
    Some(config::cli::Commands::Install) => config::install::install(helper),
    Some(config::cli::Commands::Dump) => dump(helper),
    Some(config::cli::Commands::Clear) => clear(helper),
    _ => run(helper),
  }
}

fn run(helper: config::file::FileHelper) {
  let (config, helper) = config::init(helper);

  // bemenu -> wayland
  let (tx, rx) = std::sync::mpsc::channel::<communication::MPSCMessage>();

  let clipboard = clipboard::Clipboard::init(config, helper);
  let t_clipboard = clipboard.clone();

  std::thread::spawn(move || {
    communication::SocketHandler::server().listen(clipboard, tx);
  });

  wayland::watch_clipboard(t_clipboard, rx);
}

fn toggle() {
  communication::SocketHandler::client().toggle();
}

fn dump(helper: config::file::FileHelper) {
  let (config, helper) = config::init(helper);

  let clipboard = clipboard::Clipboard::init(config, helper);
  clipboard.read().unwrap().dump();
}

fn clear(helper: config::file::FileHelper) {
  let (config, helper) = config::init(helper);

  let clipboard = clipboard::Clipboard::init(config, helper);
  clipboard.write().unwrap().clear();
}

fn init_logger(log_dir: std::path::PathBuf) -> tracing_appender::non_blocking::WorkerGuard {
  use tracing::metadata::LevelFilter;
  use tracing_subscriber::{
    filter::Directive, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
  };

  #[cfg(debug_assertions)]
  let file_appender = tracing_appender::rolling::daily(log_dir, "wayclip-debug.log");
  #[cfg(not(debug_assertions))]
  let file_appender = tracing_appender::rolling::daily(log_dir, "wayclip.log");

  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

  // directives for debug builds
  #[cfg(debug_assertions)]
  let default_directive = Directive::from(LevelFilter::TRACE);

  #[cfg(debug_assertions)]
  let filter_directives = if let Ok(filter) = std::env::var("RUST_LOG") {
    filter
  } else {
    "wayclip=trace".to_string()
  };

  // directives for release builds
  #[cfg(not(debug_assertions))]
  let default_directive = Directive::from(LevelFilter::INFO);

  #[cfg(not(debug_assertions))]
  let filter_directives = if let Ok(filter) = std::env::var("RUST_LOG") {
    filter
  } else {
    "wayclip=info".to_string()
  };

  let filter = EnvFilter::builder()
    .with_default_directive(default_directive)
    .parse_lossy(filter_directives);

  tracing_subscriber::registry()
    .with(fmt::layer())
    .with(fmt::layer().with_writer(non_blocking).with_filter(filter))
    .init();

  guard
}
