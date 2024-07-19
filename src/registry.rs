use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use bytes::Bytes;
use flate2::bufread::GzDecoder;
use reqwest::Client;

use self::manifest::{
    v1::ImageManifestV1,
    v2::{ImageManifestV2, ManifestDigestList},
    GetLayers, GetManifestResponse, MediaType,
};
use auth::*;
use manifest::Layer;
pub use repository::*;

mod auth;
mod manifest;
mod repository;

#[derive(Debug, Default)]
pub struct Registry<'r> {
    pub repository: Option<Repository<'r>>,
    access_token: String,
    client: Client,
}

const AUTH_URL: &str = "https://auth.docker.io";
const REGISTRY_URL: &str = "https://registry.hub.docker.com/v2/";

// TODO: on every method i need to construct a request, change that logic. We should have an instance that is alrady built and passed around that has the access_token embedded
impl<'r> Registry<'r> {
    pub async fn authenticate_repo(repo: Repository<'r>) -> Result<Registry> {
        let repo_auth = format!("repository:{}:pull", repo.name);
        let auth = AuthParams::new(None, None, &repo_auth);
        let mut reg = Self::authenticate(&auth).await?;
        reg.repository = Some(repo);
        Ok(reg)
    }

    #[allow(unused)]
    pub async fn authenticate_with_password(
        username: &'_ str,
        password: &'_ str,
    ) -> Result<Registry<'r>> {
        let auth = AuthParams::new(Some(username), Some(password), "registry");
        Self::authenticate(&auth).await
    }

    async fn authenticate(auth: &AuthParams<'_>) -> Result<Registry<'r>> {
        let mut url = reqwest::Url::from_str(AUTH_URL)?
            .join("token")
            .context("tried to construct url")?;
        let auth_params: String = auth
            .try_into()
            .context("tried to turn AuthParams into String")?;
        url.set_query(Some(&auth_params));
        let request = reqwest::Request::new(reqwest::Method::GET, url);
        let client = Client::new();
        let res = client
            .execute(request)
            .await
            .context("tried to authenticate with password")?;

        let AuthResponse { access_token, .. } = res
            .json()
            .await
            .context("tried parse authentication response body to json")?;

        Ok(Self {
            access_token,
            client,
            ..Default::default()
        })
    }

    pub async fn pull(&self) -> Result<()> {
        self.image_layers().await?;
        Ok(())
    }

    #[async_recursion]
    async fn get_manifest_layers(
        &self,
        name: &str,
        reference: &str,
        media_type: Option<MediaType>,
    ) -> Result<Vec<Layer>> {
        let manifest_path = format!("{}/manifests/{}", name, reference);
        let url = reqwest::Url::from_str(REGISTRY_URL)?
            .join(&manifest_path)
            .context("tried to construct manifest url")?;
        let req = self
            .client
            .request(reqwest::Method::GET, url)
            .bearer_auth(&self.access_token)
            .header("Accept", media_type.unwrap_or_default());

        let res = req.send().await.context("tried to get image manifest")?;
        match res.json().await? {
            GetManifestResponse::ManifestDigestList(manifests @ ManifestDigestList { .. }) => {
                match manifests.get_one() {
                    Some(image_manifest) => {
                        self.get_manifest_layers(
                            name,
                            &image_manifest.digest,
                            Some(image_manifest.media_type),
                        )
                        .await
                    }
                    None => Err(anyhow!("Tried to get image manifest but none was found")),
                }
            }
            GetManifestResponse::ImageManifestV2(image @ ImageManifestV2 { .. }) => {
                Ok(image.get_layers())
            }
            GetManifestResponse::ImageManifestV1(image @ ImageManifestV1 { .. }) => {
                Ok(image.get_layers())
            }
        }
    }

    async fn image_layers(&self) -> Result<()> {
        let mut layers = match &self.repository {
            Some(repo) => self.get_manifest_layers(&repo.name, repo.tag, None).await?,
            None => return Err(anyhow!("No image layers")),
        };

        let mut layer_blobs = Vec::with_capacity(layers.len());
        for (idx, layer) in layers.iter_mut().enumerate() {
            let blob = self
                .fetch_layer_blob(layer)
                .await
                .context(anyhow!("failed to fetch {idx} layer blob"))?;
            layer_blobs.push(blob);
        }
        for blob in layer_blobs {
            let mut archive = tar::Archive::new(GzDecoder::new(&blob[..]));
            archive.unpack("./foo")?;
        }

        Ok(())
    }

    async fn fetch_layer_blob(&self, layer: &mut Layer) -> Result<Bytes> {
        let blob_path = format!(
            "{}/blobs/{}",
            self.repository
                .as_ref()
                .expect("Haven't authenticated repository")
                .name,
            layer.digest
        );
        let url = reqwest::Url::from_str(REGISTRY_URL)?
            .join(&blob_path)
            .context("tried to construct layer blob url")?;
        let res = self
            .client
            .request(reqwest::Method::GET, url)
            .bearer_auth(&self.access_token)
            .header("Accept", &layer.media_type.take().unwrap_or_default())
            .send()
            .await
            .context("tried to fetch layer blob")?;

        res.bytes().await.map_err(|err| anyhow::format_err!(err))
    }

    // TODO: check the status of the api upon auth
    #[allow(unused)]
    async fn v2_status(&self) -> Result<ApiV2Status> {
        let url = reqwest::Url::from_str(REGISTRY_URL)?;
        let res = self
            .client
            .request(reqwest::Method::GET, url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("tried to get api status")?;

        match res.status() {
            /* should handle the possibility that if StatusCode::Ok there might be some content in
            the res.body that contains the allowed/existing routes that can be accessed through the api v2 */
            reqwest::StatusCode::OK => Ok(ApiV2Status::Supported),
            reqwest::StatusCode::NOT_FOUND => Ok(ApiV2Status::NotSupport),
            reqwest::StatusCode::UNAUTHORIZED => {
                let www_authenticate = res.headers().get("www-authenticate").map(|h| {
                    h.to_str()
                        .expect("www-authenticate should have content")
                        .to_owned()
                });
                Ok(ApiV2Status::Unauthorized(www_authenticate))
            }
            _ => Err(anyhow!("unexpected status code returned from /v2/")),
        }
    }

    // TODO: fetch image config from /v2/<name>/blobs/<manifest_digest>
    #[allow(unused)]
    pub async fn image_config(&self, _img: &str) -> Result<()> {
        unimplemented!()
    }

    // TODO: implement or remove this
    #[allow(unused)]
    fn extract_layer_blob(&self, blob: Bytes) -> Result<()> {
        unimplemented!()
    }
}
