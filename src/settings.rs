use serde::Deserialize;
use serde_yml::Value;
use std::{collections::HashMap, error, fmt};
use strum_macros::EnumString;

#[derive(Debug)]
pub enum Error {
    EncodeFormatQualityError(u8),
    ResizeMaxResolutionZeroError([u32; 2]),
    ResizeMaxResolutionPowerError([u32; 2]),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EncodeFormatQualityError(t) => {
                write!(f, "encode_format_quality is out-of-bounds ({})", t)
            }
            Error::ResizeMaxResolutionZeroError(t) => {
                write!(
                    f,
                    "resize_max_resolution values must be larger than 0 ({}, {})",
                    t[0], t[1]
                )
            }
            Error::ResizeMaxResolutionPowerError(t) => {
                write!(
                    f,
                    "both values of resize_max_resolution must be a power-of-two ({}, {})",
                    t[0], t[1]
                )
            }
        }
    }
}

#[derive(Deserialize, Default, Debug, PartialEq)]
pub struct Settings {
    #[serde(default = "default_enable_litterbox")]
    pub enable_litterbox: bool,
    #[serde(default)]
    pub litterbox_expire_time: ExpireTime,
    #[serde(default = "default_enable_encode")]
    pub enable_encode: bool,
    #[serde(default)]
    pub encode_format: EncodeFormat,
    #[serde(default = "default_encode_format_quality")]
    pub encode_format_quality: u8,
    #[serde(default)]
    pub enable_resize: bool,
    #[serde(default = "default_resize_max_resolution")]
    pub resize_max_resolution: [u32; 2],
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(flatten)]
    pub unexpected: HashMap<String, Value>,
}

fn default_enable_litterbox() -> bool {
    true
}

fn default_enable_encode() -> bool {
    false
}

fn default_encode_format_quality() -> u8 {
    80
}

fn default_resize_max_resolution() -> [u32; 2] {
    [1024, 1024]
}

fn default_user_agent() -> String {
    "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0".into()
}

impl Settings {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            enable_litterbox: default_enable_litterbox(),
            litterbox_expire_time: ExpireTime::default(),
            enable_encode: default_enable_encode(),
            encode_format: EncodeFormat::default(),
            encode_format_quality: default_encode_format_quality(),
            enable_resize: false,
            resize_max_resolution: default_resize_max_resolution(),
            user_agent: default_user_agent(),
            unexpected: HashMap::new(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, serde_yml::Error> {
        serde_yml::from_str(s)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.encode_format_quality > 100 || self.encode_format_quality < 1 {
            return Err(Error::EncodeFormatQualityError(self.encode_format_quality));
        }

        if !(self.resize_max_resolution[0] > 0) || !(self.resize_max_resolution[1] > 0) {
            return Err(Error::ResizeMaxResolutionZeroError(
                self.resize_max_resolution,
            ));
        }

        if !self.resize_max_resolution[0].is_power_of_two()
            || !self.resize_max_resolution[1].is_power_of_two()
        {
            return Err(Error::ResizeMaxResolutionPowerError(
                self.resize_max_resolution,
            ));
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
    #[strum(serialize = "WEBP", ascii_case_insensitive)]
    WEBP,
}

impl Default for EncodeFormat {
    fn default() -> Self {
        Self::JPG
    }
}

#[derive(EnumString, strum_macros::Display, Deserialize, Debug, PartialEq)]
pub enum ExpireTime {
    #[serde(rename = "1h")]
    #[strum(serialize = "1h", ascii_case_insensitive)]
    ONE,
    #[serde(rename = "12h")]
    #[strum(serialize = "12h", ascii_case_insensitive)]
    TWELVE,
    #[serde(rename = "24h")]
    #[strum(serialize = "24h", ascii_case_insensitive)]
    TWENTYFOUR,
    #[serde(rename = "72h")]
    #[strum(serialize = "72h", ascii_case_insensitive)]
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

    use std::str::FromStr;

    #[test]
    fn test_encode_format() {
        assert_eq!(EncodeFormat::from_str("jPg"), Ok(EncodeFormat::JPG));
        assert_eq!(EncodeFormat::from_str("jpg").unwrap(), EncodeFormat::JPG);
        assert_eq!(EncodeFormat::from_str("jPEG").unwrap(), EncodeFormat::JPG);
        assert_eq!(EncodeFormat::from_str("Png").unwrap(), EncodeFormat::PNG);
        assert_eq!(EncodeFormat::from_str("WebP").unwrap(), EncodeFormat::WEBP);
        assert!(EncodeFormat::from_str("bad").is_err());

        assert_eq!(EncodeFormat::JPG.to_string(), "JPEG");
        assert_eq!(EncodeFormat::PNG.to_string(), "PNG");
        assert_eq!(EncodeFormat::WEBP.to_string(), "WEBP");

        assert_eq!(format!("{:?}", EncodeFormat::JPG), "JPG");

        assert_eq!(EncodeFormat::JPG, EncodeFormat::JPG);
        assert_ne!(EncodeFormat::JPG, EncodeFormat::PNG);
    }

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
    fn test_validation(){
        let ok_case1 = r#"
        enable_litterbox: true
        litterbox_expire_time: "24h"
        enable_encode: false
        encode_format: "JPG"
        encode_format_quality: 80
        enable_resize: true
        resize_max_resolution:
            - 1024
            - 1024
        user_agent: "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0"
        "#;

        let settings = Settings::from_str(&ok_case1).unwrap();

        assert!(settings.validate().is_ok());
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

        assert!(settings.validate().is_err());

        let err_case2 = r#"
        encode_format_quality: 0
        "#;

        let settings = Settings::from_str(&err_case2).unwrap();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_resize_max_resolution() {
        let ok_case1 = r#"
        resize_max_resolution:
            - 512
            - 512
        "#;

        let settings = Settings::from_str(&ok_case1).unwrap();
        assert_eq!(settings.resize_max_resolution, [512, 512]);

        let err_case1 = r#"
        resize_max_resolution:
            - 0
            - 1024
        "#;

        let settings = Settings::from_str(&err_case1).unwrap();

        assert!(settings.validate().is_err());

        let err_case2 = r#"
        resize_max_resolution:
            - 1024
            - 0
        "#;

        let settings = Settings::from_str(&err_case2).unwrap();

        assert!(settings.validate().is_err());

        let err_case3 = r#"
        resize_max_resolution:
            - 0
            - 0
        "#;

        let settings = Settings::from_str(&err_case3).unwrap();

        assert!(settings.validate().is_err());

        let err_case4 = r#"
        resize_max_resolution:
            - 513
            - 512
        "#;

        let settings = Settings::from_str(&err_case4).unwrap();

        assert!(settings.validate().is_err());

        let err_case5 = r#"
        resize_max_resolution:
            - 512
            - 513
        "#;

        let settings = Settings::from_str(&err_case5).unwrap();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_error_displays() {
        assert_eq!(
            Error::EncodeFormatQualityError(255).to_string(),
            "encode_format_quality is out-of-bounds (255)"
        );

        assert_eq!(
            Error::ResizeMaxResolutionZeroError([0, 0]).to_string(),
            "resize_max_resolution values must be larger than 0 (0, 0)"
        );

        assert_eq!(
            Error::ResizeMaxResolutionPowerError([3, 5]).to_string(),
            "both values of resize_max_resolution must be a power-of-two (3, 5)"
        );
    }
}
