use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
  /// [DEFAULT]; starts the clipboard monitor
  Start,
  /// activates the clipboard menu dropdown (for use in a keybinding)
  Toggle,
  /// dumps the clipboard contents to stdout
  Dump,
  /// install
  Install,
}
