use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;

/// The state of a Homie device according to the Homie
/// [device lifecycle](https://homieiot.github.io/specification/#device-lifecycle).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    /// The state of the device is not yet known to the controller because device discovery is still
    /// underway.
    Unknown,
    /// The device is connected to the MQTT broker but is not yet ready to operate.
    Init,
    /// The device is connected and operational.
    Ready,
    /// The device has cleanly disconnected from the MQTT broker.
    Disconnected,
    /// The device is currently sleeping.
    Sleeping,
    /// The device was uncleanly disconnected from the MQTT broker. This could happen due to a
    /// network issue, power failure or some other unexpected failure.
    Lost,
    /// The device is connected to the MQTT broker but something is wrong and it may require human
    /// intervention.
    Alert,
}

impl State {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Init => "init",
            Self::Ready => "ready",
            Self::Disconnected => "disconnected",
            Self::Sleeping => "sleeping",
            Self::Lost => "lost",
            Self::Alert => "alert",
        }
    }
}

/// An error which can be returned when parsing a `State` from a string, if the string does not
/// match a valid Homie
/// [device lifecycle](https://homieiot.github.io/specification/#device-lifecycle) state.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("Invalid state '{0}'")]
pub struct ParseStateError(String);

impl FromStr for State {
    type Err = ParseStateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init" => Ok(Self::Init),
            "ready" => Ok(Self::Ready),
            "disconnected" => Ok(Self::Disconnected),
            "sleeping" => Ok(Self::Sleeping),
            "lost" => Ok(Self::Lost),
            "alert" => Ok(Self::Alert),
            _ => Err(ParseStateError(s.to_owned())),
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The data type of a Homie property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Datatype {
    Integer,
    Float,
    Boolean,
    String,
    Enum,
    Color,
}

/// An error which can be returned when parsing a `Datatype` from a string, if the string does not
/// match a valid Homie `$datatype` attribute.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("Invalid datatype '{0}'")]
pub struct ParseDatatypeError(String);

impl FromStr for Datatype {
    type Err = ParseDatatypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "integer" => Ok(Self::Integer),
            "float" => Ok(Self::Float),
            "boolean" => Ok(Self::Boolean),
            "string" => Ok(Self::String),
            "enum" => Ok(Self::Enum),
            "color" => Ok(Self::Color),
            _ => Err(ParseDatatypeError(s.to_owned())),
        }
    }
}

/// A [property](https://homieiot.github.io/specification/#properties) of a Homie node.
///
/// The `id`, `name` and `datatype` are required, but might not be available immediately when the
/// property is first discovered. The other attributes are optional.
#[derive(Clone, Debug)]
pub struct Property {
    /// The subtopic ID of the property. This is unique per node, and should follow the Homie
    /// [ID format](https://homieiot.github.io/specification/#topic-ids).
    pub id: String,

    /// The human-readable name of the property. This is a required attribute, but might not be
    /// available as soon as the property is first discovered.
    pub name: Option<String>,

    /// The data type of the property. This is a required attribute, but might not be available as
    /// soon as the property is first discovered.
    pub datatype: Option<Datatype>,

    /// Whether the property can be set by the Homie controller. This should be true for properties
    /// like the brightness or power state of a light, and false for things like the temperature
    /// reading of a sensor. It is false by default.
    pub settable: bool,

    /// Whether the property value is retained by the MQTT broker. This is true by default.
    pub retained: bool,

    /// The unit of the property, if any. This may be one of the
    /// [recommended units](https://homieiot.github.io/specification/#property-attributes), or any
    /// other custom unit.
    pub unit: Option<String>,

    /// The format of the property, if any. This should be specified if the datatype is `Enum` or
    /// `Color`, and may be specified if the datatype is `Integer` or `Float`.
    pub format: Option<String>,

    /// The current value of the property, if known. This may change frequently.
    pub value: Option<String>,
}

impl Property {
    /// Create a new property with the given ID.
    ///
    /// # Arguments
    /// * `id`: The subtopic ID for the property. This must be unique per device, and follow the
    ///   Homie [ID format](https://homieiot.github.io/specification/#topic-ids).
    pub(crate) fn new(id: &str) -> Property {
        Property {
            id: id.to_owned(),
            name: None,
            datatype: None,
            settable: false,
            retained: true,
            unit: None,
            format: None,
            value: None,
        }
    }

    /// Returns whether all the required
    /// [attributes](https://homieiot.github.io/specification/#property-attributes) of the property
    /// are filled in.
    pub fn has_required_attributes(&self) -> bool {
        self.name.is_some() && self.datatype.is_some()
    }
}

/// A [node](https://homieiot.github.io/specification/#nodes) of a Homie device.
///
/// All attributes are required, but might not be available immediately when the node is first
/// discovered.
#[derive(Clone, Debug)]
pub struct Node {
    /// The subtopic ID of the node. This is unique per device, and should follow the Homie
    /// [ID format](https://homieiot.github.io/specification/#topic-ids).
    pub id: String,

    /// The human-readable name of the node. This is a required attribute, but might not be
    /// available as soon as the node is first discovered.
    pub name: Option<String>,

    /// The type of the node. This is an arbitrary string. It is a required attribute, but might not
    /// be available as soon as the node is first discovered.
    pub node_type: Option<String>,

    /// The properties of the node, keyed by their IDs. There should be at least one.
    pub properties: HashMap<String, Property>,
}

impl Node {
    /// Create a new node with the given ID.
    ///
    /// # Arguments
    /// * `id`: The subtopic ID for the node. This must be unique per device, and follow the Homie
    ///   [ID format](https://homieiot.github.io/specification/#topic-ids).
    pub(crate) fn new(id: &str) -> Node {
        Node {
            id: id.to_owned(),
            name: None,
            node_type: None,
            properties: HashMap::new(),
        }
    }

    /// Returns whether all the required
    /// [attributes](https://homieiot.github.io/specification/#node-attributes) of the node and its
    /// properties are filled in.
    pub fn has_required_attributes(&self) -> bool {
        self.name.is_some()
            && self.node_type.is_some()
            && !self.properties.is_empty()
            && self
                .properties
                .values()
                .all(|property| property.has_required_attributes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Extension {
    pub id: String,
    pub version: String,
    pub homie_versions: Vec<String>,
}

/// An error which can be returned when parsing an `Extension` from a string.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("Invalid extension '{0}'")]
pub struct ParseExtensionError(String);

impl FromStr for Extension {
    type Err = ParseExtensionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(':').collect();
        if let [id, version, homie_versions] = parts.as_slice() {
            if let Some(homie_versions) = homie_versions.strip_prefix("[") {
                if let Some(homie_versions) = homie_versions.strip_suffix("]") {
                    return Ok(Extension {
                        id: (*id).to_owned(),
                        version: (*version).to_owned(),
                        homie_versions: homie_versions.split(';').map(|p| p.to_owned()).collect(),
                    });
                }
            }
        }
        Err(ParseExtensionError(s.to_owned()))
    }
}

/// A Homie [device](https://homieiot.github.io/specification/#devices) which has been discovered.
///
/// The `id`, `homie_version`, `name` and `state` are required, but might not be available
/// immediately when the device is first discovered. The `implementation` is optional.
#[derive(Clone, Debug)]
pub struct Device {
    /// The subtopic ID of the device. This is unique per Homie base topic, and should follow the
    /// Homie [ID format](https://homieiot.github.io/specification/#topic-ids).
    pub id: String,

    /// The version of the Homie convention which the device implements.
    pub homie_version: String,

    /// The human-readable name of the device. This is a required attribute, but might not be
    /// available as soon as the device is first discovered.
    pub name: Option<String>,

    /// The current state of the device according to the Homie
    /// [device lifecycle](https://homieiot.github.io/specification/#device-lifecycle).
    pub state: State,

    /// An identifier for the Homie implementation which the device uses.
    pub implementation: Option<String>,

    /// The nodes of the device, keyed by their IDs.
    pub nodes: HashMap<String, Node>,

    /// The Homie extensions implemented by the device.
    pub extensions: Vec<Extension>,

    /// The IP address of the device on the local network.
    pub local_ip: Option<String>,

    /// The MAC address of the device's network interface.
    pub mac: Option<String>,

    /// The name of the firmware running on the device.
    pub firmware_name: Option<String>,

    /// The version of the firware running on the device.
    pub firmware_version: Option<String>,

    /// The interval at which the device refreshes its stats.
    pub stats_interval: Option<Duration>,

    /// The amount of time since the device booted.
    pub stats_uptime: Option<Duration>,

    /// The device's signal strength in %.
    pub stats_signal: Option<i64>,

    /// The device's CPU temperature in °C.
    pub stats_cputemp: Option<f64>,

    /// The device's CPU load in %, averaged across all CPUs over the last `stats_interval`.
    pub stats_cpuload: Option<i64>,

    /// The device's battery level in %.
    pub stats_battery: Option<i64>,

    /// The device's free heap space in bytes.
    pub stats_freeheap: Option<u64>,

    /// The device's power supply voltage in volts.
    pub stats_supply: Option<f64>,
}

impl Device {
    pub(crate) fn new(id: &str, homie_version: &str) -> Device {
        Device {
            id: id.to_owned(),
            homie_version: homie_version.to_owned(),
            name: None,
            state: State::Unknown,
            implementation: None,
            nodes: HashMap::new(),
            extensions: Vec::default(),
            local_ip: None,
            mac: None,
            firmware_name: None,
            firmware_version: None,
            stats_interval: None,
            stats_uptime: None,
            stats_signal: None,
            stats_cputemp: None,
            stats_cpuload: None,
            stats_battery: None,
            stats_freeheap: None,
            stats_supply: None,
        }
    }

    /// Returns whether all the required
    /// [attributes](https://homieiot.github.io/specification/#device-attributes) of the device and
    /// all its nodes and properties are filled in.
    pub fn has_required_attributes(&self) -> bool {
        self.name.is_some()
            && self.state != State::Unknown
            && self
                .nodes
                .values()
                .all(|node| node.has_required_attributes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_parse_succeeds() {
        let legacy_stats: Extension = "org.homie.legacy-stats:0.1.1:[4.x]".parse().unwrap();
        assert_eq!(legacy_stats.id, "org.homie.legacy-stats");
        assert_eq!(legacy_stats.version, "0.1.1");
        assert_eq!(legacy_stats.homie_versions, &["4.x"]);

        let meta: Extension = "eu.epnw.meta:1.1.0:[3.0.1;4.x]".parse().unwrap();
        assert_eq!(meta.id, "eu.epnw.meta");
        assert_eq!(meta.version, "1.1.0");
        assert_eq!(meta.homie_versions, &["3.0.1", "4.x"]);

        let minimal: Extension = "a:0:[]".parse().unwrap();
        assert_eq!(minimal.id, "a");
        assert_eq!(minimal.version, "0");
        assert_eq!(minimal.homie_versions, &[""]);
    }

    #[test]
    fn extension_parse_fails() {
        assert_eq!(
            "".parse::<Extension>(),
            Err(ParseExtensionError("".to_owned()))
        );
        assert_eq!(
            "test.blah:1.2.3".parse::<Extension>(),
            Err(ParseExtensionError("test.blah:1.2.3".to_owned()))
        );
        assert_eq!(
            "test.blah:1.2.3:4.x".parse::<Extension>(),
            Err(ParseExtensionError("test.blah:1.2.3:4.x".to_owned()))
        );
    }
}