//! # Library for interacting with `NetworkManager` via D-Bus.
//!
//! The code found in sub-modules was generated w/ `zbus-xmlgen` then modified.
mod manager;
mod settings;

use manager::NetworkManagerProxyBlocking;
use serde::{Deserialize, Serialize};
use settings::SettingsProxyBlocking;
use std::collections::HashMap;
use uuid::Uuid;
use zbus::zvariant::{SerializeDict, Type};
use zbus::{
    blocking::Connection,
    zvariant::{ObjectPath, OwnedObjectPath},
};

/// Wrapper for known error cases.
#[derive(Debug, PartialEq)]
pub enum Error {
    ParseError(ParseError),
    Dbus(zbus::Error),
}

#[derive(Debug, Type, SerializeDict, Deserialize, Default)]
#[zvariant(signature = "a{sv}")]
struct C {
    #[zvariant(rename = "type")]
    kind: String,
    uuid: String,
    id: String,
}

#[derive(Debug, Type, SerializeDict, Deserialize, Default)]
#[zvariant(signature = "a{sv}")]
struct WiFi {
    ssid: Vec<u8>,
    mode: String,
}

#[derive(Debug, Type, SerializeDict, Deserialize, Default)]
#[zvariant(signature = "a{sv}")]
struct Security {
    #[zvariant(rename = "key-mgmt")]
    key_mgmt: String,
    #[zvariant(rename = "auth-alg")]
    auth_alg: String,
    psk: String,
}

#[derive(Debug, Type, SerializeDict, Deserialize, Default)]
#[zvariant(signature = "a{sv}")]
struct Ipv4 {
    method: String,
}

#[derive(Debug, Type, SerializeDict, Deserialize, Default)]
#[zvariant(signature = "a{sv}")]
struct Ipv6 {
    method: String,
}

#[derive(Debug, Type, Serialize, Deserialize, Default)]
#[zvariant(signature = "a{sa{sv}}")]
pub struct ConectionSettings {
    connection: C,
    #[serde(rename = "802-11-wireless")]
    wifi: WiFi,
    #[serde(rename = "802-11-wireless-security")]
    security: Security,
    ipv4: Ipv4,
    ipv6: Ipv6,
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl From<zbus::Error> for Error {
    fn from(value: zbus::Error) -> Self {
        Self::Dbus(value)
    }
}

/// Holds DBus connection and client proxies needed for interaction w/ `NetworkManager`.
pub struct NetworkManagerConnection {
    _connection: Connection,
    settings_proxy: SettingsProxyBlocking<'static>,
    nm_proxy: NetworkManagerProxyBlocking<'static>,
}

impl NetworkManagerConnection {
    pub fn new() -> Self {
        let connection = Connection::system().expect("should initialize dbus connection");
        let settings_proxy = SettingsProxyBlocking::new(&connection)
            .expect("should create NM Settings client proxy");
        let nm_proxy = NetworkManagerProxyBlocking::new(&connection)
            .expect("should create NM NetworkManager client proxy");

        Self {
            _connection: connection,
            settings_proxy,
            nm_proxy,
        }
    }

    fn _add_and_activate(
        &self,
        config: &ConectionSettings,
        iface: &str,
    ) -> Result<OwnedObjectPath, Error> {
        let connection = self.add(config)?;
        self.activate(&connection, iface)
    }

    /// Add a connection through `SettingsProxyBlocking`
    pub fn add(&self, config: &ConectionSettings) -> Result<OwnedObjectPath, Error> {
        self.settings_proxy
            .add_connection(config)
            .map_err(Error::Dbus)
    }
    /// Activate a connection through `NetworkManager`
    pub fn activate(
        &self,
        connection: &OwnedObjectPath,
        iface: &str,
    ) -> Result<OwnedObjectPath, Error> {
        let device = self.get_device_by_iface(iface)?;
        self.nm_proxy
            .activate_connection(connection, &device, &ObjectPath::from_str_unchecked("/"))
            .map_err(Error::Dbus)
    }

    pub fn get_device_by_iface(&self, iface: &str) -> Result<OwnedObjectPath, zbus::Error> {
        self.nm_proxy.get_device_by_ip_iface(iface)
    }

    /// Delete connection object.
    pub fn _delete(&self, path: &OwnedObjectPath) -> Result<zbus::message::Body, zbus::Error> {
        // Uses low-level API b/c dev system doesn't appear to have the
        // `Settings.Connection` service. Maybe I have an old systemd?
        let body = self
            ._connection
            .call_method(
                Some("org.freedesktop.NetworkManager"),
                path,
                Some("org.freedesktop.NetworkManager.Settings.Connection"),
                "Delete",
                &(),
            )?
            .body();
        Ok(body)
    }
    #[cfg(test)]
    /// Get the `SettingsProxyBlocking`. Useful for tests.
    fn _settings(&self) -> &SettingsProxyBlocking {
        &self.settings_proxy
    }
    #[cfg(test)]
    /// Get the `Connection`. Useful for tests.
    fn _connection(&self) -> &Connection {
        &self._connection
    }
}

#[cfg(test)]
impl ConectionSettings {
    pub fn uuid(&self) -> &str {
        &self.connection.uuid
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError(&'static str);
// reference:
// https://github.com/zxing/zxing/wiki/Barcode-Contents#wi-fi-network-config-android-ios-11
//ex:  "WIFI:S:test-ssid;T:WPA;P:00000000000000000;H:false;;"
impl TryFrom<&str> for ConectionSettings {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let qrs = s.strip_prefix("WIFI:").unwrap_or(s);
        let map: HashMap<&str, &str> = qrs
            .split(';')
            .filter_map(|element| element.split_once(":"))
            .collect();

        let c = Self {
            connection: C {
                kind: "802-11-wireless".to_string(),
                uuid: Uuid::new_v4().to_string(),
                id: map
                    .get("S")
                    .ok_or(ParseError("SSID not found"))?
                    .to_string(),
            },
            wifi: WiFi {
                ssid: map
                    .get("S")
                    .ok_or(ParseError("SSID not found"))?
                    .as_bytes()
                    .to_vec(),

                mode: "infrastructure".to_string(),
            },
            security: Security {
                key_mgmt: "wpa-psk".into(),
                auth_alg: "open".into(),
                psk: map
                    .get("P")
                    .ok_or(ParseError("Passphrase not found"))?
                    .to_string(),
            },
            ipv4: Ipv4 {
                method: "auto".into(),
            },
            ipv6: Ipv6 {
                method: "ignore".into(),
            },
        };
        Ok(c)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_signatures() {
        assert_eq!(ConectionSettings::SIGNATURE, "a{sa{sv}}");
    }

    #[test]
    fn test_parse() {
        let qr_str = "WIFI:S:test-ssid;T:WPA;P:00000000000000000;H:false;;";
        let mut c = ConectionSettings::try_from(qr_str).unwrap();
        c.connection.uuid = Uuid::new_v4().to_string();

        assert_eq!(c.connection.uuid, c.connection.uuid);
        assert_eq!("test-ssid".as_bytes(), &c.wifi.ssid);
        assert_eq!("00000000000000000", &c.security.psk);
    }

    #[test]
    #[ignore] // doesn't work on CI
    fn test_connection() {
        let qr_str = "WIFI:S:XXX-test-ssid;T:WPA;P:00000000000000000;H:false;;";
        let settings = ConectionSettings::try_from(qr_str).unwrap();
        let uuid = settings.uuid();

        let nm = NetworkManagerConnection::new();
        let _object_path = nm.add(&settings);

        // If we can get the connection then it was added.
        let object_path = nm._settings().get_connection_by_uuid(uuid).unwrap();

        // Cleanup the test connection.
        let _reply_body = nm._delete(&object_path);
    }

    #[test]
    #[ignore] // doesn't work on CI
    fn test_get_device_by_iface() {
        let nm = NetworkManagerConnection::new();
        let _err = nm.get_device_by_iface("blah0").unwrap_err();
        let _device = nm.get_device_by_iface("lo").unwrap();
    }
}
