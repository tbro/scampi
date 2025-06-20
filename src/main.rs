use qrcode_scanner::QRScanStream;
use std::{thread::sleep, time::Duration};

const MAX_RETRIES: u16 = 100;

fn main() {
    // TODO take dev from from command line and default to "/dev/video"
    let mut scanner = QRScanStream::new("/dev/video0".to_string()).unwrap();
    let mut retries = 0;

    let res = loop {
        let res = scanner.decode_next().unwrap();
        if !res.is_empty() {
            break res;
        }
        if retries > MAX_RETRIES {
            panic!("exceeded max retries ({})", MAX_RETRIES);
        }
        retries += 1;
        sleep(Duration::from_millis(200));
    };

    println!("{:#?}", res);
}
