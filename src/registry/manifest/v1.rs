#![allow(unused)]
use serde::{Deserialize, Serialize};

use super::{GetLayers, Layer as MainLayer};

pub type BlobSum = String;

#[derive(Deserialize, Serialize, Debug)]
pub struct ImageManifestV1 {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u8,
    pub name: String,
    pub architecture: String,
    #[serde(rename = "fsLayers")]
    pub fs_layers: Vec<FsLayer>,
    pub history: Vec<HistoryEntry>,
    pub signatures: Vec<Signature>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FsLayer {
    #[serde(rename = "blobSum")]
    pub blob_sum: BlobSum,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HistoryEntry {
    #[serde(rename = "v1Compatibility")]
    pub v1_compatibility: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Signature {
    pub header: Header,
    pub signature: String,
    pub protected: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Header {
    pub jwk: Jwk,
    pub alg: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Jwk {
    pub crv: String,
    pub kid: String,
    pub kty: String,
    pub x: String,
    pub y: String,
}

impl GetLayers for ImageManifestV1 {
    fn get_layers(self) -> Vec<super::Layer> {
        self.fs_layers.into_iter().map(MainLayer::from).collect()
    }
}
