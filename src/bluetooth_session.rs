use dbus::{blocking::{Connection}, message::MatchRule, channel::MatchingReceiver, Message};
use crate::BlurzError;

static BLUEZ_MATCH: &'static str = "type='signal',sender='org.bluez'";


pub struct BluetoothSession {
    connection: Connection,
}

impl core::fmt::Debug for BluetoothSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BluetoothSession").finish()
    }
}

impl BluetoothSession {
    pub fn create_session(path: Option<&str>) -> Result<BluetoothSession, BlurzError> {
        let rule = {
            if let Some(path) = path {
                format!("{},path='{}'", BLUEZ_MATCH, path)
            } else {
                String::from(BLUEZ_MATCH)
            }
        };

        let c = Connection::new_system()?;
        
        c.add_match_no_cb(&rule)?;
        Ok(BluetoothSession::new(c))
    }

    fn new(connection: Connection) -> BluetoothSession {
        BluetoothSession {
            connection: connection,
        }
    }

    pub fn get_connection(&self) -> &Connection {
        &self.connection
    }


    pub fn incoming<T>(&self, timeout_ms: u32, receiver : T ) -> Result<(), BlurzError>
        where T: Fn(Message) + Send + 'static {
    
        let receiver_id = self.connection.start_receive(MatchRule::new(), Box::new(move |message: Message, _| {
            receiver(message);
            true
        }));
    
        self.connection.process(std::time::Duration::from_millis(timeout_ms.into()))?;
        self.connection.stop_receive(receiver_id);
        Ok(())
    }
}
