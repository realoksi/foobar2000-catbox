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

#[derive(Deserialize, Debug, Clone)]
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

impl Default for EncodeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            format: Default::default(),
            quality: default_quality(),
            png: None,
            jpg: None,
            webp: None,
        }
    }
}

impl EncodeConfig {
    pub fn quality(&self) -> Option<u8> {
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

#[derive(Deserialize, Debug, Default, Copy, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct PngEncodeConfig {
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

#[derive(Deserialize, Debug, Default, Copy, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct JpgEncodeConfig {
    pub quality: Option<u8>,
}

#[derive(Deserialize, Debug, Default, Copy, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct WebpEncodeConfig {
    pub quality: Option<u8>,
    pub lossless: bool,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(try_from = "RawResizeConfig")]
pub enum ResizeConfig {
    #[default]
    Disabled,
    Enabled {
        max_resolution: MaxResolution,
        preserve_aspect_ratio: bool,
    },
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct MaxResolution {
    pub width: u32,
    pub height: u32,
}

impl MaxResolution {
    fn validate(&self) -> Result<()> {
        if self.width > 0 && self.height > 0 {
            Ok(())
        } else {
            bail!("Max resolution width and height must be greater than 0.")
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct RawResizeConfig {
    pub enabled: bool,
    pub max_resolution: Option<MaxResolution>,
    pub preserve_aspect_ratio: bool,
}

impl TryFrom<RawResizeConfig> for ResizeConfig {
    type Error = anyhow::Error;

    fn try_from(value: RawResizeConfig) -> Result<Self> {
        match (
            value.enabled,
            value.max_resolution,
            value.preserve_aspect_ratio,
        ) {
            (false, max_resolution, _) => {
                if let Some(max_resolution) = max_resolution {
                    max_resolution.validate()?;
                }
                Ok(Self::Disabled)
            }
            (true, Some(max_resolution), preserve_aspect_ratio) => {
                max_resolution.validate()?;
                Ok(Self::Enabled {
                    max_resolution,
                    preserve_aspect_ratio,
                })
            }
            (true, None, _) => {
                bail!("A maximum resolution is required while resize is enabled.")
            }
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct CatboxConfig {
    pub user_hash: Option<String>,
}

fn default_quality() -> u8 {
    80
}

fn check_quality(name: &str, quality: u8, errors: &mut Vec<String>) {
    if quality > 100 {
        errors.push(format!("{} quality cannot be greater than 100", name));
    }
}

impl Config {
    // TODO: move config validation into a raw config try_from, so it can be checked implicitly
    pub fn validate(self) -> Result<Self> {
        let mut errors = vec![];

        check_quality("encode", self.encode.quality, &mut errors);

        if let Some(jpg_encode_config) = self.encode.jpg
            && let Some(q) = jpg_encode_config.quality
        {
            check_quality("jpg", q, &mut errors);
        }

        if let Some(webp_encode_config) = self.encode.webp
            && let Some(q) = webp_encode_config.quality
        {
            check_quality("webp", q, &mut errors);
        }

        if errors.is_empty() {
            Ok(self)
        } else {
            bail!(errors.join("\n"))
        }
    }
}

#[test]
fn default_config_test() -> Result<()> {
    Config::default().validate()?;

    Ok(())
}

#[test]
fn sample_config_test() -> Result<()> {
    const TOML: &str = include_str!("../config.toml");

    let config = toml::from_str::<Config>(TOML)?.validate()?;

    assert!(config.encode.enabled);
    assert_eq!(config.encode.format, Format::Png);
    assert!(config.encode.png.is_some());
    assert!(config.encode.jpg.is_none());
    assert!(config.encode.webp.is_none());
    assert!(config.catbox.user_hash.is_none());

    let png_config = config.encode.png.unwrap();

    assert_eq!(png_config.compression, PngCompressionType::Best);
    assert_eq!(png_config.filter, PngFilterType::NoFilter);

    let resize_config = config.resize;

    match resize_config {
        ResizeConfig::Disabled => {
            panic!("Resize should not be disabled.")
        }
        ResizeConfig::Enabled {
            max_resolution,
            preserve_aspect_ratio,
        } => {
            assert_eq!(max_resolution.height, 1024);
            assert_eq!(max_resolution.width, 1024);
            assert!(preserve_aspect_ratio);
        }
    }

    Ok(())
}
