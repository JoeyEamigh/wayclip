# Changelog

All notable changes to this project will be documented in this file.

## [0.4.1] - 2023-11-19

Fixing a double free in the drop for BeMenu and minorly reworking logging.

## [0.4.0] - 2023-11-3

This release merges PR #2 from @Vilhelm-Ian, which adds the ability to clear the clipboard.

## [0.3.0] - 2023-10-2

This release fixes a memory leak. There is still another memory leak around bemenu's use of cairo but still working on figuring it out.

## [0.2.1] - 2023-06-4

This release fixes a bug where wayclip would rearrange the clipboard after paste from the wrong side of the vector. It also forces paste mimetype to what is set in config.toml. This should fix intermittent pasting bugs. Also bumps rust nightly to 1.72. Run `cargo clean` if you are getting compilation errors, as sparse-by-default may break `cargo check` and rust-analyzer.

## [0.2.0] - 2023-05-14

This release adds dedupe support, disables image copying by default (since it
makes the file large and really should be stored separately), and improves error messages.

- Added: Dedupe support
- Added: config file entry to disable image copying
- Added: Better error messages
- Fixed: logging verbosity

## [0.1.0] - 2023-05-13

First release! See README for features.
