# TODO more digging

 * wpa_supplicant has a mesh mode:
   * https://trac.gateworks.com/wiki/wireless/wifi/mesh
   * https://github.com/MayfieldRoboticsPublic/wpa_supplicant/blob/master/wpa_supplicant/mesh.c

# Testing WifiP2P via dbus

Gotchas:

 * Make sure the interface is not managed via networkmanager. Can be done via something like `nmcli device set wlo2 managed no`.
 * If you get dbus access errors when using wpa_supplicant's dbus interface, see which permissions are used for the messages. For example if you're using `dbus-broker`, you might need to edit something like `/usr/share/dbus-1/system.d/wpa_supplicant.conf` to grant your user (or the `wheel` group) the relevant permissions.
 * There's no chance of having multiple p2p "contexts" / interfaces per device per spec, however there can be multiple groups: https://lists.infradead.org/pipermail/hostap/2015-September/033754.html

# Other links to keep track of

 * https://github.com/dbus2/zbus/issues/1180
