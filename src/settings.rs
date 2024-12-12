use std::collections::HashMap;
use serde::Deserialize;
use serde_yml::Value;
use strum_macros::EnumString;

#[derive(Debug, PartialEq)]
pub enum Error {
    EncodeFormatQualityOutOfRange(u8),
    UnexpectedKeys,
}

#[derive(Deserialize, Default, Debug, PartialEq)]
pub struct Settings {
    #[serde(default = "default_enable_litterbox")]
    pub enable_litterbox: bool,
    #[serde(default)]
    pub litterbox_expire_time: ExpireTime,
    #[serde(default)]
    pub enable_encode: bool,
    #[serde(default)]
    pub encode_format: EncodeFormat,
    #[serde(default = "default_encode_format_quality")]
    pub encode_format_quality: u8,
    #[serde(default)]
    pub enable_resize: bool,
    #[serde(default)]
    pub resize_max_resolution: Option<[u16; 2]>,
    #[serde(flatten)]
    pub unexpected: HashMap<String, Value>,
}

fn default_enable_litterbox() -> bool {
    true
}

fn default_encode_format_quality() -> u8 {
    80
}

impl Settings {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            enable_litterbox: true,
            litterbox_expire_time: ExpireTime::default(),
            enable_encode: false,
            encode_format: EncodeFormat::default(),
            encode_format_quality: 80,
            enable_resize: false,
            resize_max_resolution: None,
            unexpected: HashMap::new(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, serde_yml::Error> {
        serde_yml::from_str(s)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.encode_format_quality > 100 || self.encode_format_quality < 1 {
            return Err(Error::EncodeFormatQualityOutOfRange(self.encode_format_quality));
        }

        if !self.unexpected.is_empty() {
            return Err(Error::UnexpectedKeys);
        }

        Ok(())
    }
}

#[derive(EnumString, strum_macros::Display, Deserialize, Debug, PartialEq)]
pub enum EncodeFormat {
    #[strum(serialize = "JPEG", serialize = "JPG", ascii_case_insensitive)]
    JPG,
    #[strum(serialize = "PNG", ascii_case_insensitive)]
    PNG,
}

impl Default for EncodeFormat {
    fn default() -> Self {
        Self::JPG
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum ExpireTime {
    #[serde(rename = "1h")]
    ONE,
    #[serde(rename = "12h")]
    TWELVE,
    #[serde(rename = "24h")]
    TWENTYFOUR,
    #[serde(rename = "72h")]
    SEVENTYTWO,
}

impl Default for ExpireTime {
    fn default() -> Self {
        Self::TWENTYFOUR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let ok_case1 = r#"
        "#;

        let settings = Settings::from_str(ok_case1).unwrap();
        assert_eq!(Settings::new(), settings);

        let err_case1 = r#"
            enable_litterbox: "true"
        "#;

        assert!(Settings::from_str(err_case1).is_err());
    }

    #[test]
    fn test_encode_format_quality_validation() {
        let ok_case1 = r#"
        encode_format_quality: 100
        "#;

        let settings = Settings::from_str(&ok_case1).unwrap();
        assert_eq!(settings.encode_format_quality, 100);

        let err_case1 = r#"
        encode_format_quality: 110
        "#;

        let settings = Settings::from_str(&err_case1).unwrap();
        assert_eq!(
            settings.validate(),
            Err(Error::EncodeFormatQualityOutOfRange(110))
        );

        let err_case2 = r#"
        encode_format_quality: 0
        "#;

        let settings = Settings::from_str(&err_case2).unwrap();
        assert_eq!(
            settings.validate(),
            Err(Error::EncodeFormatQualityOutOfRange(0))
        );
    }
}
