#!/bin/bash

PACKAGE=wayclip

cargo build --release

systemctl --user stop wayclip.service
sudo cp target/release/$PACKAGE /usr/local/bin/$PACKAGE
