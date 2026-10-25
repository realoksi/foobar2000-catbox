use image::codecs::png::{CompressionType, FilterType};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub transcode: Option<TranscodeConfig>,
    pub user_hash: Option<String>,
    pub timeout: Option<u32>,
    pub connection_timeout: Option<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            transcode: None,
            user_hash: None,
            timeout: Some(30),
            connection_timeout: Some(10),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct TranscodeConfig {
    pub format: Format,
    #[serde(default = "default_quality")]
    pub quality: u8,
    pub png: Option<PngTranscodeConfig>,
    pub jpg: Option<JpgTranscodeConfig>,
    pub webp: Option<WebpTranscodeConfig>,
    pub clamp: Option<ClampTranscodeConfig>,
}

impl Default for TranscodeConfig {
    fn default() -> Self {
        Self {
            format: Default::default(),
            quality: default_quality(),
            png: None,
            jpg: None,
            webp: None,
            clamp: None,
        }
    }
}

impl TranscodeConfig {
    pub fn effective_quality(&self) -> Option<u8> {
        match self.format {
            Format::Png => None,
            Format::Jpg => self.jpg.and_then(|j| j.quality).or(Some(self.quality)),
            Format::Webp => self.webp.and_then(|w| w.quality).or(Some(self.quality)),
        }
    }
}

#[derive(Deserialize, Debug, Default, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Png,
    #[default]
    #[serde(alias = "jpeg")]
    Jpg,
    Webp,
}

#[derive(Deserialize, Debug, Default, PartialEq, Copy, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct PngTranscodeConfig {
    pub compression: PngCompressionType,
    pub filter: PngFilterType,
}

/// A mirror of `image::codecs::png::CompressionType`, because the upstream type doesn't derive
/// `serde`, and is non-exhaustive. This mirror omits the upstream `Default` and `Level(u8)`
/// variants.
#[derive(Deserialize, Debug, Default, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PngCompressionType {
    #[default]
    Fast,
    Best,
    Uncompressed,
}

impl From<PngCompressionType> for CompressionType {
    fn from(value: PngCompressionType) -> CompressionType {
        match value {
            PngCompressionType::Fast => CompressionType::Fast,
            PngCompressionType::Best => CompressionType::Best,
            PngCompressionType::Uncompressed => CompressionType::Uncompressed,
        }
    }
}

/// A mirror of `image::codecs::png::FilterType`, because the upstream type doesn't derive
/// `serde`, and is non-exhaustive.
#[derive(Deserialize, Debug, Default, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PngFilterType {
    NoFilter,
    Sub,
    Up,
    Avg,
    Paeth,
    #[default]
    Adaptive,
}

impl From<PngFilterType> for FilterType {
    fn from(value: PngFilterType) -> FilterType {
        match value {
            PngFilterType::NoFilter => FilterType::NoFilter,
            PngFilterType::Sub => FilterType::Sub,
            PngFilterType::Up => FilterType::Up,
            PngFilterType::Avg => FilterType::Avg,
            PngFilterType::Paeth => FilterType::Paeth,
            PngFilterType::Adaptive => FilterType::Adaptive,
        }
    }
}

#[derive(Deserialize, Debug, Default, PartialEq, Copy, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct JpgTranscodeConfig {
    pub quality: Option<u8>,
}

#[derive(Deserialize, Debug, Default, PartialEq, Copy, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct WebpTranscodeConfig {
    pub quality: Option<u8>,
    pub lossless: bool,
}

#[derive(Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub struct ClampTranscodeConfig {
    pub max_resolution: MaxResolution,
}

#[derive(Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub struct MaxResolution {
    pub width: u32,
    pub height: u32,
}

fn default_quality() -> u8 {
    80
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    #[test]
    fn empty_config_test() -> Result<()> {
        const TOML: &str = r#""#;

        assert_eq!(toml::from_str::<Config>(TOML)?, Default::default());

        Ok(())
    }

    #[test]
    fn minimal_config_test() -> Result<()> {
        const TOML: &str = r#"
        [transcode]
        format = "webp"

        [transcode.webp]
        lossless = true
        "#;

        let config = toml::from_str::<Config>(TOML)?;

        let transcode_config = config.transcode.unwrap();

        assert_eq!(transcode_config.format, Format::Webp);

        let webp_config = transcode_config.webp.unwrap();

        assert!(webp_config.lossless);

        Ok(())
    }

    #[test]
    fn override_config_test() -> Result<()> {
        const TOML: &str = r#"
        [transcode]
        format = "jpg"
        quality = 60

        [transcode.jpg]
        quality = 80
        "#;

        let config = toml::from_str::<Config>(TOML)?;

        let transcode_config = config.transcode.unwrap();

        assert_eq!(transcode_config.format, Format::Jpg);
        assert_eq!(transcode_config.quality, 60);

        let jpg_config = transcode_config.jpg.unwrap();

        assert_eq!(jpg_config.quality, Some(80));

        assert_eq!(80, transcode_config.effective_quality().unwrap());

        Ok(())
    }

    #[test]
    fn sample_config_test() -> Result<()> {
        const TOML: &str = r#"
        user_hash = "fa06e93babb0163ad24951b25"
        timeout = 30
        connection_timeout = 10

        [transcode]
        format = "png"
        quality = 80

        [transcode.png]
        compression = "best"
        filter = "adaptive"

        [transcode.jpg]
        quality = 80

        [transcode.webp]
        lossless = true

        [transcode.clamp]
        max_resolution = { width = 1024, height = 1024 }
        "#;

        let config = toml::from_str::<Config>(TOML)?;

        let transcode_config = config.transcode.unwrap();

        assert_eq!(transcode_config.format, Format::Png);

        let jpg_config = transcode_config.jpg.unwrap();

        assert_eq!(jpg_config.quality, Some(80));

        let webp_config = transcode_config.webp.unwrap();

        assert_eq!(webp_config.quality, None);
        assert!(webp_config.lossless);

        assert_eq!(
            config.user_hash,
            Some("fa06e93babb0163ad24951b25".to_string())
        );

        let png_config = transcode_config.png.unwrap();

        assert_eq!(png_config.compression, PngCompressionType::Best);
        assert_eq!(png_config.filter, PngFilterType::Adaptive);

        let clamp_config = transcode_config.clamp.unwrap();

        assert_eq!(clamp_config.max_resolution.height, 1024);
        assert_eq!(clamp_config.max_resolution.width, 1024);

        println!("{:#?}", config);

        Ok(())
    }
}
