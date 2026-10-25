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

#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.transcode, None);
    assert_eq!(config.user_hash, None);
    assert_eq!(config.timeout, Some(30));
    assert_eq!(config.connection_timeout, Some(10));
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

#[test]
fn test_default_transcode_config() {
    let transcode_config = TranscodeConfig::default();

    assert_eq!(transcode_config.format, Default::default());
    assert_eq!(transcode_config.quality, default_quality());
    assert_eq!(transcode_config.png, None);
    assert_eq!(transcode_config.jpg, None);
    assert_eq!(transcode_config.webp, None);
    assert_eq!(transcode_config.clamp, None);
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

#[test]
fn test_effective_quality() {
    let mut transcode_config = TranscodeConfig {
        format: Format::Png,
        quality: 80,
        jpg: None,
        webp: Some(WebpTranscodeConfig {
            quality: Some(69),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_eq!(transcode_config.effective_quality(), None);

    transcode_config.format = Format::Jpg;

    assert_eq!(transcode_config.effective_quality(), Some(80));

    transcode_config.format = Format::Webp;

    assert_eq!(transcode_config.effective_quality(), Some(69));
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

#[test]
fn test_png_compression_type() {
    assert_eq!(CompressionType::from(PngCompressionType::Fast), CompressionType::Fast);
    assert_eq!(CompressionType::from(PngCompressionType::Best), CompressionType::Best);
    assert_eq!(CompressionType::from(PngCompressionType::Uncompressed), CompressionType::Uncompressed);
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

#[test]
fn test_png_filter_type() {
    assert_eq!(FilterType::from(PngFilterType::NoFilter), FilterType::NoFilter);
    assert_eq!(FilterType::from(PngFilterType::Sub), FilterType::Sub);
    assert_eq!(FilterType::from(PngFilterType::Up), FilterType::Up);
    assert_eq!(FilterType::from(PngFilterType::Avg), FilterType::Avg);
    assert_eq!(FilterType::from(PngFilterType::Paeth), FilterType::Paeth);
    assert_eq!(FilterType::from(PngFilterType::Adaptive), FilterType::Adaptive);
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
