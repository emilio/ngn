# NGN (Not Google Nearby)

This is a Rust library developed as part of my CS degree, in order to make P2P
communication easier on commercial hardware.

It's still pretty bare bones and only has a Linux and Android Wifi Direct
back-end. The ideal goal would be to have some sort of mesh network that you
can route things through, but that's not easy due to platform limitations
mainly...

Things that I'd eventually like to do:

 * non-manual tests (yeah, I know).

 * Better / different neighbor address discovery (but that's hard to do across
   platforms).

 * Bluetooth (or other transport layers) support. This would allow us to bypass
   the silly Android limitation of one physical WiFi Direct group per device
   too.

 * Support for other platforms (Windows and macOS / iOS, though the later two
   don't support WiFi Direct).

 * Improved crypto performance and auditing.

There are also a lot of issues with the setup right now, but that's not always
the library's fault:

 * Poor interaction with network manager:
   https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/1804

 * Poor interaction with some DHCP set-ups.

 * On Linux you need access to wpa_supplicant via dbus which is usually not
   granted by default (e.g. on arch you need to add a rule to
   `/usr/share/dbus-1/system.d/wpa_supplicant.conf`).

See also `random-notes.md` for well, random notes.
