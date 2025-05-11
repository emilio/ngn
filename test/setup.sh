#!/bin/bash

set -x;

cd "$(dirname "$0")"

DEBUGGER=""
DEBUGGER="rr"

WPA_SUPPLICANT="wpa_supplicant"
WPA_SUPPLICANT="../../hostap/wpa_supplicant/wpa_supplicant"

sudo systemctl stop wpa_supplicant
sudo systemctl stop NetworkManager
sudo killall wpa_supplicant
sudo killall dbus-daemon

sudo modprobe -r mac80211_hwsim
sudo modprobe mac80211_hwsim radios=3

TMP_DIR=$(mktemp -d)

echo "Test set-up starting, state dir: $TMP_DIR"

# FIXME: Assumes wlan0 is pre-existing. Arguably not amazing but...
# TODO: Add wlan3 etc once more stuff is working
IFACES="wlan1 wlan2 wlan3"

BUILD_TARGET=x86_64-unknown-linux-gnu
# BUILD_MODE="release"
BUILD_MODE="debug"
CARGO_FLAGS=""
if [ $BUILD_MODE = "release" ]; then
  CARGO_FLAGS="--release"
fi
CARGO_FLAGS+=" -Zbuild-std"
export RUSTFLAGS=-Zsanitizer=address
cargo build -vv --example dbus $CARGO_FLAGS --target=$BUILD_TARGET || exit 1

for iface in $IFACES; do
  IFACE_DIR="$TMP_DIR/$iface"
  mkdir -p "$IFACE_DIR/dbus/system-services"
  DBUS_CONF="$IFACE_DIR/dbus/system.conf"
  cat dbus-system-bus-mock.conf | sed "s#TMP_DIR#$IFACE_DIR/dbus#g" > "$DBUS_CONF"
  dbus-daemon --config-file "$DBUS_CONF" --print-address --fork > "$IFACE_DIR/dbus/log" 2>&1
  sleep .5 # TODO: Make this more reliable
  ADDRESS=$(cat "$IFACE_DIR/dbus/log" | head -1)
  echo "Address for $iface is $ADDRESS"
  sudo DBUS_SYSTEM_BUS_ADDRESS=$ADDRESS $DEBUGGER $WPA_SUPPLICANT -ddd -i $iface -c simple.conf -u 2>&1 | tee "$IFACE_DIR/wpa_supplicant.log" &
  sleep .5 # TODO: Make this more reliable
  DBUS_SYSTEM_BUS_ADDRESS=$ADDRESS nohup $DEBUGGER ../target/$BUILD_TARGET\/$BUILD_MODE/examples/dbus 2>&1 | tee "$IFACE_DIR/client.log" &
  sleep .5 # TODO: Make this more reliable
done

echo "Test set-up started, state dir: $TMP_DIR"
