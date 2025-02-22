#!/bin/bash

set -x;

cd "$(dirname "$0")"

sudo systemctl stop wpa_supplicant
sudo systemctl stop NetworkManager
sudo killall wpa_supplicant

sudo modprobe -r mac80211_hwsim
sudo modprobe mac80211_hwsim radios=3

# FIXME: Assumes wlan0 is pre-existing.
IFACES="wlan1 wlan2 wlan3"

for i in $IFACES; do
  # FIXME: Should have -u, but that fails because it tries to register the same dbus service name.
  sudo wpa_supplicant -i $i -c simple.conf &
done
