pub use bluetooth_adapter::BluetoothAdapter;
pub use bluetooth_device::BluetoothDevice;
pub use bluetooth_discovery_session::BluetoothDiscoverySession;
pub use bluetooth_event::BluetoothEvent;
pub use bluetooth_gatt_characteristic::BluetoothGATTCharacteristic;
pub use bluetooth_gatt_descriptor::BluetoothGATTDescriptor;
pub use bluetooth_gatt_service::BluetoothGATTService;
pub use bluetooth_obex::BluetoothOBEXSession;
pub use bluetooth_session::BluetoothSession;

pub mod bluetooth_adapter;
pub mod bluetooth_device;
pub mod bluetooth_discovery_session;
pub mod bluetooth_event;
pub mod bluetooth_gatt_characteristic;
pub mod bluetooth_gatt_descriptor;
pub mod bluetooth_gatt_service;
pub mod bluetooth_obex;
pub mod bluetooth_session;
mod bluetooth_utils;



#[derive(thiserror::Error, Debug)]
pub enum BlurzError {

    #[error(transparent)]
    DbusError {
        #[from]
        source: dbus::Error
    },

    #[error("An unkown error has ocurred: {0}")]
    UnkownError(String),

    #[error("Function {0} not implemented")]
    NotImplemented(String),

    #[error("Bluetooth adapter not found")]
    AdapterNotFound,

    #[error("No device found")]
    NoDeviceFound,

    #[error("Deprecated, please us {0}")]
    DeprecatedFeature(String),
}