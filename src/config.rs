use anyhow::{Result, bail};
use image::codecs::png::{CompressionType, FilterType};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub encode: EncodeConfig,
    pub resize: ResizeConfig,
    pub catbox: CatboxConfig,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
#[serde(default, deny_unknown_fields)]
pub struct EncodeConfig {
    pub enabled: bool,
    pub format: Format,
    #[serde(default = "default_quality")]
    pub quality: u8,
    pub png: Option<PngEncodeConfig>,
    pub jpg: Option<JpgEncodeConfig>,
    pub webp: Option<WebpEncodeConfig>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Png,
    #[default]
    #[serde(alias = "jpeg")]
    Jpg,
    Webp,
}

#[derive(Deserialize, Debug, Default, Copy, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct PngEncodeConfig {
    pub compression: PngCompressionType,
    pub filter: PngFilterType,
}

#[derive(Deserialize, Debug, Default, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PngCompressionType {
    Fast,
    #[default]
    Best,
    Uncompressed,
}

#[derive(Deserialize, Debug, Default, Copy, Clone)]
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

#[derive(Deserialize, Debug, Default, Copy, Clone)]
#[serde(default)]
#[serde(default, deny_unknown_fields)]
pub struct JpgEncodeConfig {
    pub quality: u8,
}

#[derive(Deserialize, Debug, Default, Copy, Clone)]
#[serde(default)]
#[serde(default, deny_unknown_fields)]
pub struct WebpEncodeConfig {
    #[serde(default = "default_quality")]
    pub quality: u8,
    pub lossless: Option<bool>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct ResizeConfig {
    pub enabled: bool,
    pub max_resolution: Option<MaxResolution>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct MaxResolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct CatboxConfig {
    pub user_hash: Option<String>,
}

fn default_quality() -> u8 {
    90
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

impl Config {
    pub fn validate(self) -> Result<Self> {
        if self.encode.quality > 100 {
            bail!("Quality cannot be greater than 100.");
        }

        Ok(self)
    }
}
