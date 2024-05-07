use botox::crypto::gen_random;
/// Animus Magic is the internal redis IPC system for internal communications between the bot and the server
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const WILDCARD_CLUSTER_ID: u16 = u16::MAX; // top means wildcard/all clusters

#[derive(Clone)]
pub struct AnimusMessageMetadata {
    pub from: AnimusTarget,
    pub to: AnimusTarget,
    pub cluster_id_from: u16,
    pub cluster_id_to: u16,
    pub op: AnimusOp,
    pub command_id: String,
    pub payload_offset: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
pub enum AnimusTarget {
    #[default]
    Bot,
    Jobserver,
    Webserver,
    Infra,
    Wildcard,
}

impl AnimusTarget {
    pub fn to_byte(self) -> u8 {
        match self {
            AnimusTarget::Bot => 0x0,
            AnimusTarget::Jobserver => 0x1,
            AnimusTarget::Webserver => 0x2,
            AnimusTarget::Infra => 0x3,
            AnimusTarget::Wildcard => u8::MAX,
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x0 => Some(AnimusTarget::Bot),
            0x1 => Some(AnimusTarget::Jobserver),
            0x2 => Some(AnimusTarget::Webserver),
            0x3 => Some(AnimusTarget::Infra),
            u8::MAX => Some(AnimusTarget::Wildcard),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Default, Copy, Clone)]
#[non_exhaustive]
pub enum AnimusOp {
    #[default]
    Request,
    Response,
    Error,
    Probe,
}

impl AnimusOp {
    pub fn to_byte(&self) -> u8 {
        match self {
            AnimusOp::Request => 0x0,
            AnimusOp::Response => 0x1,
            AnimusOp::Error => 0x2,
            AnimusOp::Probe => 0x3,
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x0 => Some(AnimusOp::Request),
            0x1 => Some(AnimusOp::Response),
            0x2 => Some(AnimusOp::Error),
            0x3 => Some(AnimusOp::Probe),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AnimusErrorResponse {
    pub message: String,
    pub context: String,
}

// Impl From trait for converting a display to an AnimusErrorResponse
impl<T: core::fmt::Display> From<T> for AnimusErrorResponse {
    fn from(e: T) -> Self {
        Self {
            message: "Failed to create response".to_string(),
            context: e.to_string(),
        }
    }
}

/// Wrapper to parse a struct T to bytes
pub fn serialize_data<T: Serialize>(payload: &T) -> Result<Vec<u8>, crate::Error> {
    let bytes = serde_cbor::to_vec(payload)?;
    Ok(bytes)
}

/// Wrapper to parse a payload to a struct T
pub fn deserialize_data<T: for<'a> Deserialize<'a>>(payload: &[u8]) -> Result<T, crate::Error> {
    let msg = serde_cbor::from_slice::<T>(payload)?;
    Ok(msg)
}

#[allow(dead_code)]
pub fn new_command_id() -> String {
    gen_random(16)
}

#[allow(dead_code)]
pub fn default_request_timeout() -> Duration {
    Duration::from_secs(10)
}

/// Creates a payload
pub fn create_payload(
    cmd_id: &str,
    from: AnimusTarget,
    cluster_id_from: u16, // From which cluster the message is coming from
    cluster_id_to: u16,   // To which cluster the message is is going to
    to: AnimusTarget,
    op: AnimusOp,
    data: Vec<u8>,
) -> Vec<u8> {
    let mut payload = Vec::new();

    // Push from as 1 u8
    payload.push(from.to_byte());

    // Push to as 1 u8
    payload.push(to.to_byte());

    // Push cluster id from and to as 2 u8s
    let cluster_id = cluster_id_from.to_be_bytes();

    for byte in cluster_id {
        payload.push(byte);
    }

    let cluster_id = cluster_id_to.to_be_bytes();

    for byte in cluster_id {
        payload.push(byte);
    }

    // Push the op byte
    payload.push(op.to_byte());

    // Push the command id
    for byte in cmd_id.as_bytes() {
        payload.push(*byte);
    }

    // Push seperator of '/'
    payload.push(0x2f);

    for byte in data {
        payload.push(byte);
    }

    payload
}

// Parses the metadata of a payload
pub fn get_payload_meta(payload: &[u8]) -> Result<AnimusMessageMetadata, crate::Error> {
    const FROM_BYTE: usize = 0;
    const TO_BYTE: usize = FROM_BYTE + 1;
    const CLUSTER_ID_FROM_BYTE: usize = TO_BYTE + 1;
    const CLUSTER_ID_TO_BYTE: usize = CLUSTER_ID_FROM_BYTE + 2;
    const OP_BYTE: usize = CLUSTER_ID_TO_BYTE + 2;

    // Take out from
    let from = AnimusTarget::from_byte(payload[FROM_BYTE]).ok_or("Invalid from byte")?;

    // Take out scope
    let to = AnimusTarget::from_byte(payload[TO_BYTE]).ok_or("Invalid type byte")?;

    // Take out cluster id
    let cluster_id_from = u16::from_be_bytes([
        payload[CLUSTER_ID_FROM_BYTE],
        payload[CLUSTER_ID_FROM_BYTE + 1],
    ]);
    let cluster_id_to =
        u16::from_be_bytes([payload[CLUSTER_ID_TO_BYTE], payload[CLUSTER_ID_TO_BYTE + 1]]);

    let op = AnimusOp::from_byte(payload[OP_BYTE]).ok_or("Invalid op byte")?;

    let mut cmd_id = String::new();

    let mut i = OP_BYTE + 1;
    loop {
        if payload[i] == 0x2f {
            break;
        }

        cmd_id.push(payload[i] as char);

        i += 1;
    }

    Ok(AnimusMessageMetadata {
        from,
        to,
        cluster_id_from,
        cluster_id_to,
        op,
        command_id: cmd_id,
        payload_offset: i + 1,
    })
}
