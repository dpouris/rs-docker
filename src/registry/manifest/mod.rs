pub mod v1;
pub mod v2;
use self::{
    v1::{FsLayer, ImageManifestV1},
    v2::{ImageManifestV2, Layer as V2Layer, ManifestDigestList},
};
use serde::{Deserialize, Serialize};

/// [Digest] is a type wrapper on [String] and is typically a
/// **\<encryption algorithm\>:\<hexencoded hash\>** [String] e.g `sha256:e4c58958181a5925816faa528ce959e487632f4cfd192f8132f71b32df2744b4`
pub type Digest = String;
/// MediaType contains the Content-Type header which is expected by the [v2] api to be present when requesting a resource from it e.g
/// `application/vnd.oci.image.manifest.v1+json` **or** `application/vnd.oci.image.config.v1+json`
pub type MediaType = String;
pub type Size = u32;

const VALID_PLATFORMS: [(&str, &str); 3] =
    [("linux", "amd64"), ("linux", "arm64"), ("linux", "arm")];

pub struct Layer {
    pub digest: Digest,
    pub media_type: Option<MediaType>,
}

pub trait GetLayers {
    fn get_layers(self) -> Vec<Layer>;
}

impl From<V2Layer> for Layer {
    fn from(value: V2Layer) -> Self {
        Self {
            digest: value.digest,
            media_type: Some(value.media_type),
        }
    }
}

impl From<FsLayer> for Layer {
    fn from(value: FsLayer) -> Self {
        Self {
            digest: value.blob_sum,
            media_type: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetManifestResponse {
    ManifestDigestList(ManifestDigestList),
    ImageManifestV1(ImageManifestV1),
    ImageManifestV2(ImageManifestV2),
}
