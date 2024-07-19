#![allow(unused)] //remove later
use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};

use super::{Digest, GetLayers, Layer as MainLayer, MediaType, Size, VALID_PLATFORMS};

#[derive(Deserialize, Serialize, Debug)]
pub struct ManifestDigestList {
    pub manifests: Vec<ManifestDigest>,
    #[serde(rename = "mediaType")]
    pub media_type: MediaType,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u8,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ManifestDigest {
    pub digest: Digest,
    #[serde(rename = "mediaType")]
    pub media_type: MediaType,
    pub platform: Platform,
    pub size: Size,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Platform {
    // Specifies the CPU architecture, for example amd64 or ppc64le.
    pub architecture: String,
    // Specifies the operating system, for example linux or windows.
    pub os: String,
    // Specifies the operating system version, for example 10.0.10586.
    #[serde(rename = "os.version")]
    pub os_version: Option<String>,
    // Specifies an array of strings, each listing a required OS feature (for example on Windows win32k).
    #[serde(rename = "os.features")]
    pub os_features: Option<Vec<String>>,
    // Specifies a variant of the CPU, for example v6 to specify a particular CPU variant of the ARM CPU.
    pub variant: Option<String>,
    // Specifies an array of strings, each listing a required CPU feature (for example sse4 or aes).
    pub features: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ImageManifestV2 {
    #[serde(rename = "mediaType")]
    pub media_type: MediaType,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u8,
    pub config: Config,
    pub layers: Vec<Layer>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(rename = "mediaType")]
    pub media_type: MediaType,
    pub size: Size,
    pub digest: Digest,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Layer {
    #[serde(rename = "mediaType")]
    pub media_type: MediaType,
    pub size: Size,
    pub digest: Digest,
}

impl ManifestDigestList {
    pub fn get_one(self) -> Option<ManifestDigest> {
        if self.manifests.is_empty() {
            return None;
        }
        self.manifests
            .into_iter()
            .filter_map(|manifest| {
                if manifest.platform.is_valid() {
                    Some(manifest)
                } else {
                    None
                }
            })
            .take(1)
            .next()
    }
}

impl Platform {
    fn is_valid(&self) -> bool {
        let platform = (self.os.as_str(), self.architecture.as_str());

        VALID_PLATFORMS.iter().any(|&plat| plat == platform)
    }
}

impl GetLayers for ImageManifestV2 {
    fn get_layers(self) -> Vec<super::Layer> {
        self.layers.into_iter().map(MainLayer::from).collect()
    }
}
