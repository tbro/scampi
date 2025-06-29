use argh::FromArgs;
use network_manager::{ConectionSettings, NetworkManagerConnection};
use qrcode_scanner::QRScanStream;
use std::{thread::sleep, time::Duration};
use zbus::zvariant::OwnedObjectPath;

mod network_manager;

#[derive(FromArgs)]
/// Command options.
struct Options {
    /// video device to scan (default = `/dev/video0`).
    #[argh(option, short = 'd', default = "String::from(\"/dev/video0\")")]
    video_device: String,
    /// number of times to try scanning the image.
    #[argh(option, short = 'r', default = "default_retries()")]
    retries: u16,
    /// activate any scanned WIFI connections to `NetworkManager`
    #[argh(switch, short = 'w')]
    wifi: bool,
    /// network interface to active connection
    #[argh(option, short = 'i', default = "String::from(\"wlan0\")")]
    interface: String,
}

fn default_retries() -> u16 {
    100
}

fn main() {
    let options: Options = argh::from_env();
    if let Some(res) = scan(options.retries, options.video_device) {
        println!("{res:#?}");

        if options.wifi {
            // TODO validate interface against known interfaces (dbus)
            println!("Activating Connection for {}", options.interface);
            let nm = NetworkManagerConnection::new();

            let _paths: Vec<OwnedObjectPath> = res
                .iter()
                .map(|s| {
                    ConectionSettings::try_from(s.as_str())
                        .and_then(|settings| nm.add(&settings))
                        .and_then(|connection| nm.activate(&connection, &options.interface))
                })
                .filter_map(|r| r.map_err(|e| eprintln!("{e:?}")).ok())
                .collect();
        }
    } else {
        eprintln!("Scan failed: exceeded timeout");
    }
}

/// Scan the device.
fn scan(retries: u16, device: String) -> Option<Vec<String>> {
    let mut scanner = QRScanStream::new(device).expect("should open device");
    for _ in 0..retries {
        let res = scanner.decode_next().expect("should decode image");
        if !res.is_empty() {
            return Some(res);
        }
        sleep(Duration::from_millis(200));
    }
    None
}
