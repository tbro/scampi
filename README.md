# QR WiFi NM Connection Helper

Adds a network connection to [NetworkManager](https://www.networkmanager.dev/)
from scanned QR code. It uses the attached webcam to scan the QR code, and
passes the configuration to network manager via DBUS.

Note that by default it only prints out the scanned data. Provide `-w` option to add the connection to `NetworkManager`.

## Usage

	cargo run -- --help

## Caveats

  * Only works with **wpa-psk** for now.
  * No UI.
