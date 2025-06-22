#!/bin/bash

# Sets up wpa_supplicant to control wlan0 without NetworkManager interference.
# Useful to test connected to an Android device with something like:
#   cargo run --target x86_64-unknown-linux-gnu --example dbus
# While running the android app.

set -x;

cd "$(dirname "$0")"

# WPA_SUPPLICANT="wpa_supplicant"
# WPA_SUPPLICANT="../../hostap/wpa_supplicant/wpa_supplicant -ddd" # For really verbose logging.
# WPA_SUPPLICANT="../../hostap/wpa_supplicant/wpa_supplicant"

sudo systemctl stop wpa_supplicant
sudo systemctl stop NetworkManager
sudo killall wpa_supplicant
sudo killall dbus-daemon

sudo $WPA_SUPPLICANT -i wlan0 -c simple.conf -u 2>&1
