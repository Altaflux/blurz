use crate::bluetooth_session::BluetoothSession;
use crate::BlurzError;
use dbus::arg::Variant;
use dbus::arg::messageitem::{MessageItem, MessageItemDict};
use dbus::Message;
use dbus::{
    arg::Arg,
    blocking::BlockingSender,
};
use std::time::Duration;

static ADAPTER_INTERFACE: &'static str = "org.bluez.Adapter1";
static SERVICE_NAME: &'static str = "org.bluez";

pub struct BluetoothDiscoverySession<'a> {
    adapter: String,
    session: &'a BluetoothSession,
}

impl<'a> BluetoothDiscoverySession<'a> {
    pub fn create_session(
        session: &'a BluetoothSession,
        adapter: String,
    ) -> Result<BluetoothDiscoverySession, BlurzError> {
        Ok(BluetoothDiscoverySession::new(session, adapter))
    }

    fn new(session: &'a BluetoothSession, adapter: String) -> BluetoothDiscoverySession<'a> {
        BluetoothDiscoverySession {
            adapter: adapter,
            session: session,
        }
    }

    fn call_method(&self, method: &str, param: Option<[MessageItem; 1]>) -> Result<(), BlurzError> {
        let mut m =
            Message::new_method_call(SERVICE_NAME, &self.adapter, ADAPTER_INTERFACE, method)
                .map_err(|err| BlurzError::UnkownError(err))?;
        match param {
            Some(p) => m.append_items(&p),
            None => (),
        };

        self.session
            .get_connection()
            .send_with_reply_and_block(m, Duration::from_millis(1000))?;
        Ok(())
    }

    pub fn start_discovery(&self) -> Result<(), BlurzError> {
        self.call_method("StartDiscovery", None)
    }

    pub fn stop_discovery(&self) -> Result<(), BlurzError> {
        self.call_method("StopDiscovery", None)
    }

    pub fn set_discovery_filter(
        &self,
        uuids: Vec<String>,
        rssi: Option<i16>,
        pathloss: Option<u16>,
    ) -> Result<(), BlurzError> {
        let uuids = {
            let mut res: Vec<MessageItem> = Vec::new();
            for u in uuids {
                res.push(u.into());
            }
            res
        };

        let mut m:Vec<(MessageItem, MessageItem)> = vec![(
            MessageItem::from(Box::new("UUIDs".into())),
            MessageItem::Variant(Box::new(
                MessageItem::new_array(uuids).unwrap(),
            )),
        )];

        if let Some(rssi) = rssi {
            m.push((
                MessageItem::from( Box::new("RSSI".into())),
                MessageItem::Variant(Box::new(rssi.into())),
            ))
        }

        if let Some(pathloss) = pathloss {
            m.push((
                MessageItem::from(Box::new("Pathloss".into())),
                MessageItem::Variant(Box::new(pathloss.into())),
            ))
        }

        self.call_method(
            "SetDiscoveryFilter",
            Some([MessageItem::Dict(
                MessageItemDict::new(
                    m
                    ,
                    <String as Arg>::signature(),
                    <Variant<u16> as Arg>::signature(),
                )
                .unwrap(),
            )]),
        )
    }

}
