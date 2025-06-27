# scampi
*scampi* is a simple tool for Linux that allows you to use your web-cam to scan
QR codes. It can add WiFi connections encoded as QR codes directly to
[NetworkManager](https://www.networkmanager.dev/).

The name *desugars* to something like **s**can **cam** wi**f**i.

By default it only prints out the scanned data. Provide `-w` option to add the
connection to `NetworkManager`.

## Usage
To print the decoded QR code:

	cargo run

To scan a QR code and add the decoded WiFi connection to `NetworkManager`:

	cargo run -- -w -i wlan0

The default interface (`-i`) is `wlan0`, so if in fact your WiFi interface is
`wlan0`, you can omit the '-i` options.

## Help

	cargo run -- --help

## Caveats

  * Only works with **wpa-psk** for now.
  * No UI.
