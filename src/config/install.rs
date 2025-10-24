use crate::config::consts::SYSTEMD_FILE;

use super::file::FileHelper;

pub fn install(helper: FileHelper) {
  let path = helper.install_systemd_file();

  println!("Installed systemd file to: {}", path.display());
  print!("Would you like to enable it now? [y/N]: ");

  use std::io::Write;
  std::io::stdout().flush().unwrap();

  let mut input = String::new();
  std::io::stdin().read_line(&mut input).unwrap();

  if input.trim().to_lowercase() == "y" {
    std::process::Command::new("systemctl")
      .arg("--user")
      .arg("enable")
      .arg(SYSTEMD_FILE)
      .arg("--now")
      .spawn()
      .unwrap()
      .wait()
      .unwrap();
    println!("\nEnabling and starting {}", SYSTEMD_FILE);
  } else {
    println!(
      "\nYou can enable it later by running: systemctl --user enable {}",
      SYSTEMD_FILE
    );
  }
}
