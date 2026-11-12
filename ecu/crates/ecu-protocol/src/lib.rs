//! Versioned OpenTech-ECU protocol.
//!
//! Two transports are defined:
//! - JSON envelopes for the browser/Tauri WebSocket link;
//! - a length-delimited binary frame with CRC32 for embedded/serial links.
//!
//! No message writes raw ECU memory. Calibration changes are explicit,
//! checksum-verified documents applied by the service with revision history.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec::Vec;

use ecu_types::TelemetryFrame;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAGIC: [u8; 2] = *b"OE";
pub const MAX_BINARY_PAYLOAD: usize = 64 * 1024;
pub const HEADER_LEN: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    pub protocol_version: u16,
    pub client_name: String,
    pub client_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloAck {
    pub accepted: bool,
    pub protocol_version: u16,
    pub server_version: String,
    pub capabilities: Vec<String>,
    pub reason: String,
}

pub fn negotiate(hello: &Hello, server_version: &str) -> HelloAck {
    if hello.protocol_version != PROTOCOL_VERSION {
        return HelloAck {
            accepted: false,
            protocol_version: PROTOCOL_VERSION,
            server_version: server_version.into(),
            capabilities: Vec::new(),
            reason: format!(
                "protocol version {} not supported; server uses {}",
                hello.protocol_version, PROTOCOL_VERSION
            ),
        };
    }
    HelloAck {
        accepted: true,
        protocol_version: PROTOCOL_VERSION,
        server_version: server_version.into(),
        capabilities: vec![
            "telemetry_json".into(),
            "binary_frames_crc32".into(),
            "calibration_snapshots".into(),
            "mixture_map_offline".into(),
            "stft_ltft".into(),
            "replay".into(),
        ],
        reason: String::new(),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    Hello(HelloAck),
    Telemetry {
        seq: u64,
        calibration_checksum: String,
        frame: TelemetryFrame,
    },
    Status {
        status: String,
        calibration_revision: u32,
        running: bool,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JsonEnvelope {
    pub protocol: u16,
    pub seq: u64,
    pub event: ServerEvent,
}

impl JsonEnvelope {
    pub fn new(seq: u64, event: ServerEvent) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            seq,
            event,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    TooShort,
    BadMagic,
    UnsupportedVersion(u16),
    PayloadTooLarge,
    CrcMismatch { expected: u32, actual: u32 },
    EmptyPayload,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::TooShort => f.write_str("frame shorter than header"),
            ProtocolError::BadMagic => f.write_str("bad magic"),
            ProtocolError::UnsupportedVersion(v) => write!(f, "unsupported protocol version {v}"),
            ProtocolError::PayloadTooLarge => f.write_str("payload too large"),
            ProtocolError::CrcMismatch { expected, actual } => {
                write!(
                    f,
                    "CRC mismatch expected {expected:08x} actual {actual:08x}"
                )
            }
            ProtocolError::EmptyPayload => f.write_str("empty payload"),
        }
    }
}

/// Binary frame layout:
/// magic(2) | version(2 LE) | flags(2 LE) | sequence(4 LE) | length(2 LE) | crc32(4 LE) | payload
pub fn encode_binary_frame(seq: u32, flags: u16, payload: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    if payload.is_empty() {
        return Err(ProtocolError::EmptyPayload);
    }
    if payload.len() > MAX_BINARY_PAYLOAD {
        return Err(ProtocolError::PayloadTooLarge);
    }
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&seq.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    let crc = crc32(payload);
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedFrame<'a> {
    pub version: u16,
    pub flags: u16,
    pub sequence: u32,
    pub payload: &'a [u8],
}

pub fn decode_binary_frame(bytes: &[u8]) -> Result<DecodedFrame<'_>, ProtocolError> {
    if bytes.len() < HEADER_LEN {
        return Err(ProtocolError::TooShort);
    }
    if bytes[0..2] != MAGIC {
        return Err(ProtocolError::BadMagic);
    }
    let version = u16::from_le_bytes([bytes[2], bytes[3]]);
    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(version));
    }
    let flags = u16::from_le_bytes([bytes[4], bytes[5]]);
    let sequence = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
    let len = u16::from_le_bytes([bytes[10], bytes[11]]) as usize;
    if len == 0 || len > MAX_BINARY_PAYLOAD {
        return Err(if len == 0 {
            ProtocolError::EmptyPayload
        } else {
            ProtocolError::PayloadTooLarge
        });
    }
    if bytes.len() != HEADER_LEN + len {
        return Err(ProtocolError::TooShort);
    }
    let expected = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    let payload = &bytes[HEADER_LEN..];
    let actual = crc32(payload);
    if expected != actual {
        return Err(ProtocolError::CrcMismatch { expected, actual });
    }
    Ok(DecodedFrame {
        version,
        flags,
        sequence,
        payload,
    })
}

/// CRC-32/ISO-HDLC (reflected polynomial 0xEDB88320), bitwise so no lookup
/// table or global state is required in firmware.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_vector() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn frame_round_trip() {
        let payload = br#"{"rpm":3000}"#;
        let bytes = encode_binary_frame(42, 1, payload).unwrap();
        let f = decode_binary_frame(&bytes).unwrap();
        assert_eq!(f.version, PROTOCOL_VERSION);
        assert_eq!(f.flags, 1);
        assert_eq!(f.sequence, 42);
        assert_eq!(f.payload, payload);
    }

    #[test]
    fn corrupted_payload_fails_crc() {
        let mut bytes = encode_binary_frame(1, 0, b"hello").unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;
        assert!(matches!(
            decode_binary_frame(&bytes),
            Err(ProtocolError::CrcMismatch { .. })
        ));
    }

    #[test]
    fn truncated_and_bad_magic_are_rejected() {
        assert_eq!(
            decode_binary_frame(b"OE").unwrap_err(),
            ProtocolError::TooShort
        );
        let mut bytes = encode_binary_frame(1, 0, b"x").unwrap();
        bytes[0] = b'X';
        assert_eq!(
            decode_binary_frame(&bytes).unwrap_err(),
            ProtocolError::BadMagic
        );
    }

    #[test]
    fn version_mismatch_is_rejected_explicitly() {
        let mut bytes = encode_binary_frame(1, 0, b"x").unwrap();
        bytes[2] = 99;
        bytes[3] = 0;
        assert_eq!(
            decode_binary_frame(&bytes).unwrap_err(),
            ProtocolError::UnsupportedVersion(99)
        );
    }

    #[test]
    fn handshake_rejects_wrong_version() {
        let h = Hello {
            protocol_version: 2,
            client_name: "test".into(),
            client_version: "1".into(),
        };
        let ack = negotiate(&h, "0.1.0");
        assert!(!ack.accepted);
        assert!(ack.reason.contains("not supported"));
    }

    #[test]
    fn json_telemetry_envelope_has_protocol_and_crc_covered_binary_equivalent() {
        let e = JsonEnvelope::new(
            1,
            ServerEvent::Telemetry {
                seq: 1,
                calibration_checksum: "abc".into(),
                frame: TelemetryFrame::default(),
            },
        );
        let json = e.to_json().unwrap();
        assert!(json.contains("\"protocol\":1"));
        let back: JsonEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(back.protocol, PROTOCOL_VERSION);
    }
}
