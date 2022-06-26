use std::error::Error;

use blurz::bluetooth_event::BluetoothEvent;
use blurz::bluetooth_session::BluetoothSession as Session;

fn test5() -> Result<(), Box<dyn Error>> {
    let session = &Session::create_session(Some("/org/bluez/hci0")).unwrap();
    loop {
        session.incoming(1000, |message| {
            println!("{:?}", BluetoothEvent::from(message))
        })?;
    }
}

fn main() {
    match test5() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
