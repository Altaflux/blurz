use dbus::{blocking::{Connection, BlockingSender}, Message, arg::{Arg, Append}};
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use dbus::arg::messageitem::MessageItem;
use crate::BlurzError;

static ADAPTER_INTERFACE: &'static str = "org.bluez.Adapter1";
static DEVICE_INTERFACE: &'static str = "org.bluez.Device1";
static SERVICE_INTERFACE: &'static str = "org.bluez.GattService1";
static CHARACTERISTIC_INTERFACE: &'static str = "org.bluez.GattCharacteristic1";
static DESCRIPTOR_INTERFACE: &'static str = "org.bluez.GattDescriptor1";
static SERVICE_NAME: &'static str = "org.bluez";

fn get_managed_objects(c: &Connection) -> Result<Vec<MessageItem>, BlurzError> {
    let m = Message::new_method_call(
        SERVICE_NAME,
        "/",
        "org.freedesktop.DBus.ObjectManager",
        "GetManagedObjects"
    ).map_err(|err| BlurzError::UnkownError(err))?;
    
    let r = c.send_with_reply_and_block(m, std::time::Duration::from_millis(1000))?;
    Ok(r.get_items())
}

pub fn get_adapters(c: &Connection) -> Result<Vec<String>, BlurzError> {
    let mut adapters: Vec<String> = Vec::new();
    let objects: Vec<MessageItem> = get_managed_objects(&c)?;
    let z: &[(MessageItem, MessageItem)] = objects.get(0).unwrap().inner().unwrap();
    for (path, interfaces) in z {
        for (i, _) in interfaces.inner::<&[(MessageItem, MessageItem)]>().unwrap() {
            let name: &str = i.inner().unwrap();
            if name == ADAPTER_INTERFACE {
                let p: &str = path.inner().unwrap();
                adapters.push(String::from(p));
            }
        }
    }
    println!("{:?}", adapters);
    Ok(adapters)
}

pub fn list_devices(c: &Connection, adapter_path: &String) -> Result<Vec<String>, BlurzError> {
    list_item(c, DEVICE_INTERFACE, adapter_path, "Adapter")
}

pub fn list_services(c: &Connection, device_path: &String) -> Result<Vec<String>, BlurzError> {
    list_item(c, SERVICE_INTERFACE, device_path, "Device")
}

pub fn list_characteristics(
    c: &Connection,
    device_path: &String,
) -> Result<Vec<String>, BlurzError> {
    list_item(c, CHARACTERISTIC_INTERFACE, device_path, "Service")
}

pub fn list_descriptors(c: &Connection, device_path: &String) -> Result<Vec<String>, BlurzError> {
    list_item(c, DESCRIPTOR_INTERFACE, device_path, "Characteristic")
}

fn list_item(
    c: &Connection,
    item_interface: &str,
    item_path: &str,
    item_property: &str,
) -> Result<Vec<String>, BlurzError> {
    let mut v: Vec<String> = Vec::new();
    let objects: Vec<MessageItem> = get_managed_objects(&c)?;
    let z: &[(MessageItem, MessageItem)] = objects.get(0).unwrap().inner().unwrap();
    for (path, interfaces) in z {
        for (i, _) in interfaces.inner::<&[(MessageItem, MessageItem)]>().unwrap() {
            let name: &str = i.inner().unwrap();
            if name == item_interface {
                let objpath: &str = path.inner().unwrap();
                let prop = (get_property(c, item_interface, objpath, item_property))?;
                let prop_path = prop.inner::<&str>().unwrap();
                if prop_path == item_path {
                    v.push(String::from(objpath));
                }
            }
        }
    }
    Ok(v)
}

pub fn get_property(
    c: &Connection,
    interface: &str,
    object_path: &str,
    prop: &str,
) -> Result<MessageItem, BlurzError> {
    let p = c.with_proxy(SERVICE_NAME, object_path, std::time::Duration::from_millis(1000));
    let metadata: MessageItem = p.get(interface, prop)?;
    Ok(metadata)
}

pub fn set_property<T: Arg + Append>(
    c: &Connection,
    interface: &str,
    object_path: &str,
    prop: &str,
    value: T,
    timeout_ms: i32,
) -> Result<(), BlurzError>
where
    T: Into<MessageItem>,
{
    let p = c.with_proxy(SERVICE_NAME, object_path, std::time::Duration::from_millis(timeout_ms.try_into().unwrap()));
    p.set(interface, prop, dbus::arg::Variant(value))?;
    Ok(())
}

pub fn call_method(
    c: &Connection,
    interface: &str,
    object_path: &str,
    method: &str,
    param: Option<&[MessageItem]>,
    timeout_ms: i32,
) -> Result<(), BlurzError> {
    let mut m = Message::new_method_call(
        SERVICE_NAME,
        object_path,
        interface,
        method
    ).map_err(|err| BlurzError::UnkownError(err))?;
    match param {
        Some(p) => m.append_items(p),
        None => (),
    };
    c.send_with_reply_and_block(m,std::time::Duration::from_millis(timeout_ms.try_into().unwrap()))?;
    Ok(())
}
