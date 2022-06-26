use dbus::arg::{Variant};
use dbus::Path as ObjectPath;
use dbus::{blocking::{Connection, BlockingSender}, Message};
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use dbus::arg::messageitem::MessageItem;
use std::time::Duration;
use std::collections::HashMap;

use std::path::Path;
use std::thread::sleep;

use crate::BlurzError;

use crate::bluetooth_device::BluetoothDevice;
use crate::bluetooth_session::BluetoothSession;

const OBEX_BUS: &str = "org.bluez.obex";
const OBEX_PATH: &str = "/org/bluez/obex";
const OBJECT_PUSH_INTERFACE: &str = "org.bluez.obex.ObjectPush1";
const CLIENT_INTERFACE: &str = "org.bluez.obex.Client1";
const TRANSFER_INTERFACE: &str = "org.bluez.obex.Transfer1";

pub enum SessionTarget {
    Ftp,
    Map,
    Opp,
    Pbap,
    Sync_,
}

impl SessionTarget {
    fn as_str(&self) -> &str {
        match self {
            SessionTarget::Ftp => "ftp",
            SessionTarget::Map => "map",
            SessionTarget::Opp => "opp",
            SessionTarget::Pbap => "pbap",
            SessionTarget::Sync_ => "sync",
        }
    }
}

pub enum TransferState {
    Queued,
    Active,
    Complete,
    Suspended,
    Error,
}

impl TransferState {
    fn as_str(&self) -> &str {
        match self {
            TransferState::Queued => "queued",
            TransferState::Active => "active",
            TransferState::Complete => "complete",
            TransferState::Suspended => "suspended",
            TransferState::Error => "error",
        }
    }
}

pub fn open_bus_connection() -> Result<Connection, BlurzError> {
    let c = Connection::new_session()?;
    Ok(c)
}

pub struct BluetoothOBEXSession<'a> {
    session: &'a BluetoothSession,
    object_path: String,
}

impl<'a> BluetoothOBEXSession<'a> {
    // https://git.kernel.org/pub/scm/bluetooth/bluez.git/tree/doc/obex-api.txt#n12
    pub fn new(
        session: &'a BluetoothSession,
        device: &BluetoothDevice,
    ) -> Result<BluetoothOBEXSession<'a>, BlurzError> {
        let device_address: String = device.get_address()?;
        let mut map = HashMap::new();
        map.insert("Target", Variant(SessionTarget::Opp.as_str()));
        let m = Message::new_method_call(OBEX_BUS, OBEX_PATH, CLIENT_INTERFACE, "CreateSession")
            .map_err(|err| BlurzError::UnkownError(err))?
            .append2(device_address, map);

        let r = session
            .get_connection()
            .send_with_reply_and_block(m, std::time::Duration::from_millis(1000))?;
        let session_path: ObjectPath = r.read1()?;
        let session_str: String = session_path.parse().map_err(|_| BlurzError::UnkownError("Could not parse path".to_owned()))?;
        let obex_session = BluetoothOBEXSession {
            session,
            object_path: session_str,
        };
        Ok(obex_session)
    }

    // https://git.kernel.org/pub/scm/bluetooth/bluez.git/tree/doc/obex-api.txt#n35
    pub fn remove_session(&self) -> Result<(), BlurzError> {
        let object_path = ObjectPath::new(&self.object_path)
            .map_err(|err| BlurzError::UnkownError(err))?;
        let m = Message::new_method_call(OBEX_BUS, OBEX_PATH, CLIENT_INTERFACE, "RemoveSession")
            .map_err(|err| BlurzError::UnkownError(err))?
            .append1(object_path);
        let _r = self
            .session
            .get_connection()
            .send_with_reply_and_block(m, std::time::Duration::from_millis(1000))?;
        Ok(())
    }
}

pub struct BluetoothOBEXTransfer<'a> {
    session: &'a BluetoothOBEXSession<'a>,
    object_path: String,
    _name: String,
}

impl<'a> BluetoothOBEXTransfer<'a> {
    // https://git.kernel.org/pub/scm/bluetooth/bluez.git/tree/doc/obex-api.txt#n169
    pub fn send_file(
        session: &'a BluetoothOBEXSession,
        file_path: &str,
    ) -> Result<BluetoothOBEXTransfer<'a>, BlurzError> {
        let session_path: String = session.object_path.clone();
        let m =
            Message::new_method_call(OBEX_BUS, session_path, OBJECT_PUSH_INTERFACE, "SendFile")
                .map_err(|err| BlurzError::UnkownError(err))?
                .append1(file_path);
        let r = session
            .session
            .get_connection()
            .send_with_reply_and_block(m, std::time::Duration::from_millis(1000))?;
        let transfer_path: ObjectPath = r.read1()?;
        let transfer_str: String = transfer_path.parse().map_err(|_| BlurzError::UnkownError("Could not parse path".to_owned()))?;

        let file_name: String = match Path::new(file_path).file_name() {
            Some(value) => value.to_string_lossy().to_string(),
            None => file_path.to_string(),
        };

        let obex_transfer = BluetoothOBEXTransfer {
            session,
            object_path: transfer_str,
            _name: file_name,
        };
        Ok(obex_transfer)
    }

    // https://git.kernel.org/pub/scm/bluetooth/bluez.git/tree/doc/obex-api.txt#n115
    pub fn status(&self) -> Result<String, BlurzError> {
        let transfer_path = self.object_path.clone();

        //let p = c.with_proxy(SERVICE_NAME, object_path, std::time::Duration::from_millis(1000));
        //let metadata: MessageItem = p.get(interface, prop)?;

        let p = &self.session.session.get_connection().with_proxy(OBEX_BUS, transfer_path, std::time::Duration::from_millis(1000));
        let status: MessageItem = p.get(TRANSFER_INTERFACE, "Status")?;
        match status.inner::<&str>() {
            Ok(value) => Ok(value.to_string()),
            Err(_) => Err(BlurzError::FailedToGetStatus),
        }
    }

    pub fn wait_until_transfer_completed(&self) -> Result<(), BlurzError> {
        sleep(Duration::from_millis(500));
        let mut transfer_status: String = self.status()?;

        while transfer_status != TransferState::Complete.as_str() {
            sleep(Duration::from_millis(500));
            transfer_status = match self.status() {
                Ok(value) => {
                    if value == TransferState::Error.as_str() {
                        break;
                    } else {
                        value
                    }
                }
                Err(_) => break,
            }
        }
        Ok(())
    }
}
