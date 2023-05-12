#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(drain_filter)]

use clap::Parser;

mod clipboard;
mod communication;
mod config;
mod input;
mod menu;
mod wayland;

// #[tokio::main]
fn main() {
  let helper = config::init_helper();
  let _guard = init_logger(helper.log_dir.clone());

  let cli = config::cli::Cli::parse();

  match &cli.command {
    Some(config::cli::Commands::Toggle) => toggle(),
    Some(config::cli::Commands::Install) => config::install::install(helper),
    _ => run(helper),
  }
}

fn run(helper: config::file::FileHelper) {
  let (config, helper) = config::init(helper);

  // bemenu -> wayland
  let (tx, rx) = std::sync::mpsc::channel::<String>();

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

// dev logger
#[cfg(debug_assertions)]
fn init_logger(_log_dir: std::path::PathBuf) -> &'static str {
  tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();

  "guard"
}

// "prod" logger
#[cfg(not(debug_assertions))]
fn init_logger(log_dir: std::path::PathBuf) -> tracing_appender::non_blocking::WorkerGuard {
  let file_appender = tracing_appender::rolling::daily(log_dir, "wayclip.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  tracing_subscriber::fmt().with_writer(non_blocking).init();
  guard
}
