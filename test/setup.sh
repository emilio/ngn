#!/bin/bash

set -x;

cd "$(dirname "$0")"

sudo systemctl stop wpa_supplicant
sudo systemctl stop NetworkManager
sudo killall wpa_supplicant
sudo killall dbus-daemon

sudo modprobe -r mac80211_hwsim
sudo modprobe mac80211_hwsim radios=3

TMP_DIR=$(mktemp -d)

# FIXME: Assumes wlan0 is pre-existing. Arguably not amazing but...
# TODO: Add wlan3 etc once more stuff is working
IFACES="wlan1 wlan2"

for iface in $IFACES; do
  IFACE_DIR="$TMP_DIR/$iface"
  mkdir -p "$IFACE_DIR/dbus/system-services"
  DBUS_CONF="$IFACE_DIR/dbus/system.conf"
  cat dbus-system-bus-mock.conf | sed "s#TMP_DIR#$IFACE_DIR/dbus#g" > "$DBUS_CONF"
  dbus-daemon --config-file "$DBUS_CONF" --print-address --fork > "$IFACE_DIR/dbus/log" 2>&1
  sleep .5 # TODO: Make this more reliable
  ADDRESS=$(cat "$IFACE_DIR/dbus/log" | head -1)
  echo "Address for $iface is $ADDRESS"
  sudo DBUS_SYSTEM_BUS_ADDRESS=$ADDRESS wpa_supplicant -i $iface -c simple.conf -u >"$IFACE_DIR/wpa_supplicant.log" 2>&1 &
  sleep .5 # TODO: Make this more reliable
  DBUS_SYSTEM_BUS_ADDRESS=$ADDRESS cargo run --example dbus > "$IFACE_DIR/client.log" 2>&1 &
  sleep .5 # TODO: Make this more reliable
done

echo "Test set-up started, state dir: $TMP_DIR"
