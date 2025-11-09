use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Base64 encoding error: {0}")]
    Base64EncodeError(String),
    #[error("Base64 decoding error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub type SerializationResult<T> = Result<T, SerializationError>;

/// JSON serialization utilities
pub struct JsonUtils;

impl JsonUtils {
    /// Serialize any serializable type to JSON string
    pub fn to_string<T: Serialize>(value: &T) -> SerializationResult<String> {
        serde_json::to_string(value).map_err(SerializationError::from)
    }

    /// Serialize to pretty-printed JSON
    pub fn to_string_pretty<T: Serialize>(value: &T) -> SerializationResult<String> {
        serde_json::to_string_pretty(value).map_err(SerializationError::from)
    }

    /// Deserialize JSON string to type
    pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> SerializationResult<T> {
        serde_json::from_str(s).map_err(SerializationError::from)
    }

    /// Serialize to JSON Value
    pub fn to_value<T: Serialize>(value: &T) -> SerializationResult<Value> {
        serde_json::to_value(value).map_err(SerializationError::from)
    }

    /// Deserialize from JSON Value
    pub fn from_value<T: for<'de> Deserialize<'de>>(value: Value) -> SerializationResult<T> {
        serde_json::from_value(value).map_err(SerializationError::from)
    }

    /// Merge two JSON objects
    pub fn merge_objects(base: &mut Map<String, Value>, override_with: Map<String, Value>) {
        for (key, value) in override_with {
            base.insert(key, value);
        }
    }

    /// Extract field from JSON Value
    pub fn extract_field(value: &Value, field_name: &str) -> SerializationResult<Value> {
        value
            .get(field_name)
            .cloned()
            .ok_or_else(|| SerializationError::MissingField(field_name.to_string()))
    }

    /// Extract string field
    pub fn extract_string(value: &Value, field_name: &str) -> SerializationResult<String> {
        let field = Self::extract_field(value, field_name)?;
        field
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| SerializationError::InvalidFormat(format!(
                "Field '{}' is not a string", field_name
            )))
    }

    /// Extract number field as u64
    pub fn extract_u64(value: &Value, field_name: &str) -> SerializationResult<u64> {
        let field = Self::extract_field(value, field_name)?;
        field
            .as_u64()
            .ok_or_else(|| SerializationError::InvalidFormat(format!(
                "Field '{}' is not a valid u64", field_name
            )))
    }

    /// Extract boolean field
    pub fn extract_bool(value: &Value, field_name: &str) -> SerializationResult<bool> {
        let field = Self::extract_field(value, field_name)?;
        field
            .as_bool()
            .ok_or_else(|| SerializationError::InvalidFormat(format!(
                "Field '{}' is not a boolean", field_name
            )))
    }

    /// Check if JSON value has a specific field
    pub fn has_field(value: &Value, field_name: &str) -> bool {
        value.get(field_name).is_some()
    }

    /// Sanitize JSON by removing sensitive fields
    pub fn sanitize_json(value: &mut Value, sensitive_fields: &[&str]) {
        if let Some(obj) = value.as_object_mut() {
            for field in sensitive_fields {
                if obj.contains_key(*field) {
                    obj.insert(field.to_string(), Value::String("[REDACTED]".to_string()));
                }
            }
        }
    }
}

/// Base64 encoding/decoding utilities
pub struct Base64Utils;

impl Base64Utils {
    /// Encode bytes to base64 string
    pub fn encode(data: &[u8]) -> String {
        BASE64.encode(data)
    }

    /// Decode base64 string to bytes
    pub fn decode(encoded: &str) -> SerializationResult<Vec<u8>> {
        BASE64.decode(encoded).map_err(SerializationError::from)
    }

    /// Encode string to base64
    pub fn encode_string(s: &str) -> String {
        Self::encode(s.as_bytes())
    }

    /// Decode base64 to string
    pub fn decode_string(encoded: &str) -> SerializationResult<String> {
        let bytes = Self::decode(encoded)?;
        String::from_utf8(bytes).map_err(SerializationError::from)
    }

    /// URL-safe base64 encoding
    pub fn encode_url_safe(data: &[u8]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
    }

    /// URL-safe base64 decoding
    pub fn decode_url_safe(encoded: &str) -> SerializationResult<Vec<u8>> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(SerializationError::from)
    }
}

/// Hex encoding/decoding utilities
pub struct HexUtils;

impl HexUtils {
    /// Encode bytes to hex string
    pub fn encode(data: &[u8]) -> String {
        hex::encode(data)
    }

    /// Decode hex string to bytes
    pub fn decode(encoded: &str) -> SerializationResult<Vec<u8>> {
        hex::decode(encoded).map_err(|e| {
            SerializationError::InvalidFormat(format!("Invalid hex string: {}", e))
        })
    }

    /// Encode bytes to hex with 0x prefix
    pub fn encode_with_prefix(data: &[u8]) -> String {
        format!("0x{}", Self::encode(data))
    }

    /// Decode hex string (with or without 0x prefix)
    pub fn decode_flexible(encoded: &str) -> SerializationResult<Vec<u8>> {
        let hex_str = encoded.strip_prefix("0x").unwrap_or(encoded);
        Self::decode(hex_str)
    }

    /// Check if string is valid hex
    pub fn is_valid_hex(s: &str) -> bool {
        let hex_str = s.strip_prefix("0x").unwrap_or(s);
        hex_str.chars().all(|c| c.is_ascii_hexdigit())
    }
}

/// Message Pack serialization utilities (for efficient binary serialization)
pub struct MsgPackUtils;

impl MsgPackUtils {
    /// Serialize to MessagePack bytes
    pub fn to_bytes<T: Serialize>(value: &T) -> SerializationResult<Vec<u8>> {
        rmp_serde::to_vec(value).map_err(|e| {
            SerializationError::InvalidFormat(format!("MessagePack serialization error: {}", e))
        })
    }

    /// Deserialize from MessagePack bytes
    pub fn from_bytes<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> SerializationResult<T> {
        rmp_serde::from_slice(bytes).map_err(|e| {
            SerializationError::InvalidFormat(format!("MessagePack deserialization error: {}", e))
        })
    }

    /// Serialize to MessagePack and then base64 encode
    pub fn to_base64<T: Serialize>(value: &T) -> SerializationResult<String> {
        let bytes = Self::to_bytes(value)?;
        Ok(Base64Utils::encode(&bytes))
    }

    /// Decode base64 and then deserialize from MessagePack
    pub fn from_base64<'a, T: for<'de> Deserialize<'de>>(encoded: &'a str) -> SerializationResult<T> {
        let bytes = Base64Utils::decode(encoded)?;
        Self::from_bytes(&bytes)
    }
}

/// Query string serialization utilities
pub struct QueryStringUtils;

impl QueryStringUtils {
    /// Serialize struct to query string
    pub fn to_string<T: Serialize>(value: &T) -> SerializationResult<String> {
        serde_urlencoded::to_string(value).map_err(|e| {
            SerializationError::InvalidFormat(format!("Query string serialization error: {}", e))
        })
    }

    /// Deserialize query string to struct
    pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> SerializationResult<T> {
        serde_urlencoded::from_str(s).map_err(|e| {
            SerializationError::InvalidFormat(format!("Query string deserialization error: {}", e))
        })
    }

    /// Parse query string to HashMap
    pub fn to_map(query: &str) -> HashMap<String, String> {
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect()
    }

    /// Build query string from HashMap
    pub fn from_map(params: &HashMap<String, String>) -> String {
        if params.is_empty() {
            return String::new();
        }

        let pairs: Vec<String> = params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    urlencoding::encode(k),
                    urlencoding::encode(v)
                )
            })
            .collect();

        format!("?{}", pairs.join("&"))
    }
}

/// Data compression utilities
pub struct CompressionUtils;

impl CompressionUtils {
    /// Compress data using gzip
    pub fn compress_gzip(data: &[u8]) -> SerializationResult<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| {
            SerializationError::InvalidFormat(format!("Compression error: {}", e))
        })?;

        encoder.finish().map_err(|e| {
            SerializationError::InvalidFormat(format!("Compression finalization error: {}", e))
        })
    }

    /// Decompress gzip data
    pub fn decompress_gzip(compressed: &[u8]) -> SerializationResult<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();

        decoder.read_to_end(&mut decompressed).map_err(|e| {
            SerializationError::InvalidFormat(format!("Decompression error: {}", e))
        })?;

        Ok(decompressed)
    }

    /// Compress JSON to gzip
    pub fn compress_json<T: Serialize>(value: &T) -> SerializationResult<Vec<u8>> {
        let json = JsonUtils::to_string(value)?;
        Self::compress_gzip(json.as_bytes())
    }

    /// Decompress gzip to JSON
    pub fn decompress_json<T: for<'de> Deserialize<'de>>(compressed: &[u8]) -> SerializationResult<T> {
        let decompressed = Self::decompress_gzip(compressed)?;
        let json_str = String::from_utf8(decompressed)?;
        JsonUtils::from_str(&json_str)
    }
}

/// Serialization format detector
pub struct FormatDetector;

impl FormatDetector {
    /// Detect if data is JSON
    pub fn is_json(data: &[u8]) -> bool {
        serde_json::from_slice::<Value>(data).is_ok()
    }

    /// Detect if string is base64
    pub fn is_base64(s: &str) -> bool {
        Base64Utils::decode(s).is_ok()
    }

    /// Detect if string is hex
    pub fn is_hex(s: &str) -> bool {
        HexUtils::is_valid_hex(s)
    }

    /// Detect data format
    pub fn detect_format(data: &[u8]) -> &'static str {
        // Check for gzip magic number
        if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b {
            return "gzip";
        }

        // Check for JSON
        if Self::is_json(data) {
            return "json";
        }

        // Check for MessagePack (starts with specific byte patterns)
        if !data.is_empty() {
            let first_byte = data[0];
            if (0x80..=0x8f).contains(&first_byte)  // fixmap
                || (0x90..=0x9f).contains(&first_byte)  // fixarray
                || (0xa0..=0xbf).contains(&first_byte)  // fixstr
            {
                return "messagepack";
            }
        }

        "binary"
    }
}

/// Helper struct for building JSON objects
pub struct JsonBuilder {
    map: Map<String, Value>,
}

impl JsonBuilder {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
        }
    }

    pub fn insert<V: Into<Value>>(mut self, key: &str, value: V) -> Self {
        self.map.insert(key.to_string(), value.into());
        self
    }

    pub fn insert_if_some<V: Into<Value>>(mut self, key: &str, value: Option<V>) -> Self {
        if let Some(v) = value {
            self.map.insert(key.to_string(), v.into());
        }
        self
    }

    pub fn build(self) -> Value {
        Value::Object(self.map)
    }
}

impl Default for JsonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        name: String,
        age: u32,
    }

    #[test]
    fn test_json_serialization() {
        let test_data = TestStruct {
            name: "Alice".to_string(),
            age: 30,
        };

        let json_str = JsonUtils::to_string(&test_data).unwrap();
        assert!(json_str.contains("Alice"));
        assert!(json_str.contains("30"));

        let deserialized: TestStruct = JsonUtils::from_str(&json_str).unwrap();
        assert_eq!(deserialized, test_data);
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"Hello, Nexus-Security!";
        let encoded = Base64Utils::encode(data);
        let decoded = Base64Utils::decode(&encoded).unwrap();

        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hex_encoding() {
        let data = b"test data";
        let encoded = HexUtils::encode(data);
        let decoded = HexUtils::decode(&encoded).unwrap();

        assert_eq!(decoded, data);

        assert!(HexUtils::is_valid_hex(&encoded));
        assert!(HexUtils::is_valid_hex("0xabcdef123"));
        assert!(!HexUtils::is_valid_hex("not hex"));
    }

    #[test]
    fn test_json_builder() {
        let json = JsonBuilder::new()
            .insert("name", "Bob")
            .insert("age", 25)
            .insert_if_some("email", Some("bob@example.com"))
            .insert_if_some("phone", None::<String>)
            .build();

        assert_eq!(JsonUtils::extract_string(&json, "name").unwrap(), "Bob");
        assert_eq!(JsonUtils::extract_u64(&json, "age").unwrap(), 25);
        assert!(JsonUtils::has_field(&json, "email"));
        assert!(!JsonUtils::has_field(&json, "phone"));
    }

    #[test]
    fn test_sanitize_json() {
        let mut json = serde_json::json!({
            "username": "alice",
            "password": "secret123",
            "api_key": "key123"
        });

        JsonUtils::sanitize_json(&mut json, &["password", "api_key"]);

        assert_eq!(JsonUtils::extract_string(&json, "username").unwrap(), "alice");
        assert_eq!(JsonUtils::extract_string(&json, "password").unwrap(), "[REDACTED]");
        assert_eq!(JsonUtils::extract_string(&json, "api_key").unwrap(), "[REDACTED]");
    }

    #[test]
    fn test_msgpack_serialization() {
        let test_data = TestStruct {
            name: "Charlie".to_string(),
            age: 35,
        };

        let bytes = MsgPackUtils::to_bytes(&test_data).unwrap();
        let deserialized: TestStruct = MsgPackUtils::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized, test_data);
    }

    #[test]
    fn test_compression() {
        let data = b"This is some test data that should compress well because it has repetition. repetition. repetition.";

        let compressed = CompressionUtils::compress_gzip(data).unwrap();
        let decompressed = CompressionUtils::decompress_gzip(&compressed).unwrap();

        assert_eq!(decompressed, data);
        assert!(compressed.len() < data.len()); // Should be compressed
    }

    #[test]
    fn test_format_detection() {
        let json_data = b"{\"key\": \"value\"}";
        assert_eq!(FormatDetector::detect_format(json_data), "json");

        let compressed = CompressionUtils::compress_gzip(b"test").unwrap();
        assert_eq!(FormatDetector::detect_format(&compressed), "gzip");
    }

    #[test]
    fn test_query_string() {
        let mut params = HashMap::new();
        params.insert("page".to_string(), "1".to_string());
        params.insert("limit".to_string(), "20".to_string());

        let query = QueryStringUtils::from_map(&params);
        assert!(query.contains("page=1"));
        assert!(query.contains("limit=20"));

        let parsed = QueryStringUtils::to_map(query.trim_start_matches('?'));
        assert_eq!(parsed.get("page"), Some(&"1".to_string()));
        assert_eq!(parsed.get("limit"), Some(&"20".to_string()));
    }
}
