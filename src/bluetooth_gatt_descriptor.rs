use crate::bluetooth_session::BluetoothSession;
use crate::bluetooth_utils;
use crate::BlurzError;

use dbus::arg::messageitem::{MessageItem, MessageItemDict};
use dbus::arg::Variant;

use dbus::{
    arg::{Arg},
    blocking::{BlockingSender, Connection},
    Message,
};
use std::time::Duration;
static SERVICE_NAME: &'static str = "org.bluez";
static GATT_DESCRIPTOR_INTERFACE: &'static str = "org.bluez.GattDescriptor1";

#[derive(Clone, Debug)]
pub struct BluetoothGATTDescriptor<'a> {
    object_path: String,
    session: &'a BluetoothSession,
}

impl<'a> BluetoothGATTDescriptor<'a> {
    pub fn new(session: &'a BluetoothSession, object_path: String) -> BluetoothGATTDescriptor {
        BluetoothGATTDescriptor {
            object_path: object_path,
            session: session,
        }
    }

    pub fn get_id(&self) -> String {
        self.object_path.clone()
    }

    fn get_property(&self, prop: &str) -> Result<MessageItem, BlurzError> {
        bluetooth_utils::get_property(
            self.session.get_connection(),
            GATT_DESCRIPTOR_INTERFACE,
            &self.object_path,
            prop,
        )
    }

    fn call_method(
        &self,
        method: &str,
        param: Option<&[MessageItem]>,
        timeout_ms: i32,
    ) -> Result<(), BlurzError> {
        bluetooth_utils::call_method(
            self.session.get_connection(),
            GATT_DESCRIPTOR_INTERFACE,
            &self.object_path,
            method,
            param,
            timeout_ms,
        )
    }

    /*
     * Properties
     */

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n198
    pub fn get_uuid(&self) -> Result<String, BlurzError> {
        let uuid = self.get_property("UUID")?;
        Ok(String::from(uuid.inner::<&str>().unwrap()))
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n202
    pub fn get_characteristic(&self) -> Result<String, BlurzError> {
        let service = self.get_property("Characteristic")?;
        Ok(String::from(service.inner::<&str>().unwrap()))
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n207
    pub fn get_value(&self) -> Result<Vec<u8>, BlurzError> {
        let value = self.get_property("Value")?;
        let z: &[MessageItem] = value.inner().unwrap();
        let mut v: Vec<u8> = Vec::new();
        for y in z {
            v.push(y.inner::<u8>().unwrap());
        }
        Ok(v)
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n213
    pub fn get_flags(&self) -> Result<Vec<String>, BlurzError> {
        let flags = self.get_property("Flags")?;
        let z: &[MessageItem] = flags.inner().unwrap();
        let mut v: Vec<String> = Vec::new();
        for y in z {
            v.push(String::from(y.inner::<&str>().unwrap()));
        }
        Ok(v)
    }

    /*
     * Methods
     */


    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n174
    pub fn read_value(&self, offset: Option<u16>) -> Result<Vec<u8>, BlurzError> {
        let c = Connection::new_system()?;
        let mut m = Message::new_method_call(
            SERVICE_NAME,
            &self.object_path,
            GATT_DESCRIPTOR_INTERFACE,
            "ReadValue",
        )
        .map_err(|err| BlurzError::UnkownError(err))?;
        m.append_items(&[MessageItem::Dict(
            MessageItemDict::new(
                match offset {
                    Some(o) => vec![(
                        MessageItem::from(Box::new("offset".into())),
                        MessageItem::Variant(Box::new(o.into())),
                    )],
                    None => vec![],
                },
                <String as Arg>::signature(),
                <Variant<u8> as Arg>::signature(),
            )
            .unwrap(),
        )]);
        let reply = c.send_with_reply_and_block(m, Duration::from_millis(1000))?;
        let items: MessageItem = reply.get1().unwrap();
        let z: &[MessageItem] = items.inner().unwrap();
        let mut v: Vec<u8> = Vec::new();
        for i in z {
            v.push(i.inner::<u8>().unwrap());
        }
        Ok(v)
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n186
    pub fn write_value(&self, values: Vec<u8>, offset: Option<u16>) -> Result<(), BlurzError> {
        let args = {
            let mut res: Vec<MessageItem> = Vec::new();
            for v in values {
                res.push(v.into());
            }
            res
        };
        self.call_method(
            "WriteValue",
            Some(&[
                MessageItem::new_array(args).unwrap(),
                MessageItem::Dict(
                    MessageItemDict::new(
                        match offset {
                            Some(o) => vec![(
                                MessageItem::from(Box::new("offset".into())),
                                MessageItem::Variant(Box::new(o.into())),
                            )],
                            None => vec![],
                        },
                        <String as Arg>::signature(),
                        <Variant<u8> as Arg>::signature(),
                    )
                    .map_err(|_| BlurzError::UnkownError("".to_owned()))?,
                ),
            ]),
            1000,
        )
    }
}
