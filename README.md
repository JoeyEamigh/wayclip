# wayclip: an opinionated Wayland clipboard manager (for kde and sway at the moment)

wayclip is a clipboard manager for Wayland compositors. It is written in Rust and monitors the clipboard by interfacing with zwlr_data_control_manager_v1. this means it works on KWin and Sway for the moment.

## Features

- text clipboard history
- selection of history items with bemenu
- history persistence
- history item limit
- history encryption

## Dependencies

wayclip depends on the following in version 1:

- bemenu

## Installation

wayclip needs access to the `input` user group to paste since the wayland virtual keyboard protocol has spotty support (and i use kde).

to add your user to the input group run:

```bash
sudo gpasswd -a $USER input
```

then log out and back in, or reboot.

### From source

```bash
git clone https://github.com/JoeyEamigh/wayclip.git
cd wayclip

cargo build --release
./install.sh // installs to /usr/local/bin and requires sudo

wayclip install // installs systemd file
```

### Arch Linux AUR

```bash
paru -S wayclip-manager-git
yay -S wayclip-manager-git

wayclip install // installs systemd file
```

### Cargo

```bash
cargo install wayclip

wayclip install // installs systemd file
```

## Config

the config file for wayclip will be created after first run and lives at `~/.config/wayclip/config.toml`. most of the options work, but some are works in progress.

## Usage

wayclip is a daemon that monitors the clipboard. when you run `wayclip install`, it installs a user systemd file which can be enabled with `systemctl --user enable wayclip.service` and started with `systemctl --user start wayclip.service`.

since wayland has no working hotkeys system, you should use your compositor's hotkey system to start wayclip. for example, in kde 5.27, open the shortcuts setting panel, and click `add command`. type `wayclip toggle` in the prompt box, then bind it to your preferred shortcut.

## TODO (not sure how much of this i will actually do)

- [x] add an actual dedupe
- [ ] add support for multiple text mime-types at a time
- [ ] add support for images
- [ ] add support for files
- [ ] add support for other compositors and wayland protocols (ongoing)
- [ ] add support for other menu programs (dmenu, rofi, etc.)
